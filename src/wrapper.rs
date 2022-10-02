use crate::raw_types::{
    RawCoursePrerequisite, RawEvent, RawPrerequisite, RawScheduledMeeting, RawWebRegMeeting,
    RawWebRegSearchResultItem,
};
use crate::types::{
    CoursePrerequisite, CourseSection, EnrollmentStatus, Event, Meeting, MeetingDay,
    PrerequisiteInfo, ScheduledSection,
};
use crate::util::{self, parse_binary_days};
use reqwest::header::{COOKIE, USER_AGENT};
use reqwest::{Client, Error, Response};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use thiserror::Error;
use url::Url;

// URLs for WebReg
const MY_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, \
like Gecko) Chrome/97.0.4692.71 Safari/537.36";

const DEFAULT_SCHEDULE_NAME: &str = "My Schedule";

// Random WebReg links
const WEBREG_BASE: &str = "https://act.ucsd.edu/webreg2";
const WEBREG_SEARCH: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-by-all?";
const WEBREG_SEARCH_SEC: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-by-sectionid?";
const ACC_NAME: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/get-current-name";
const COURSE_DATA: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-load-group-data?";
const CURR_SCHEDULE: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/get-class?";
const SEND_EMAIL: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/send-email";
const CHANGE_ENROLL: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/change-enroll";

const REMOVE_SCHEDULE: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/sched-remove";
const RENAME_SCHEDULE: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-rename";
const ALL_SCHEDULE: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/sched-get-schednames";

const PING_SERVER: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/ping-server";

const PLAN_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-add";
const PLAN_REMOVE: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-remove";
const PLAN_EDIT: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/edit-plan";
const PLAN_REMOVE_ALL: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-remove-all";

const ENROLL_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/add-enroll";
const ENROLL_EDIT: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/edit-enroll";
const ENROLL_DROP: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/drop-enroll";

const WAITLIST_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/add-wait";
const WAITLIST_EDIT: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/edit-wait";
const WAILIST_DROP: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/drop-wait";

const PREREQS_INFO: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/get-prerequisites?";

const EVENT_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-add";
const EVENT_EDIT: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-edit";
const EVENT_REMOVE: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-remove";
const EVENT_GET: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-get?";

#[derive(Error, Debug)]
pub enum WrapperError {
    /// Occurs if there was an error encountered by the reqwest library.
    #[error("request error occurred: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Occurs when there was an error with serde.
    #[error("serde error occurred: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Occurs when the wrapper encounters a bad status code
    #[error("unsuccessful status code: {0}")]
    BadStatusCode(u16),

    /// Occurs when an error from WebReg was returned.
    #[error("error from WebReg: {0}")]
    WebRegError(String),

    /// Occurs when the given input is not valid.
    #[error("invalid input for '{0}' provided: {1}")]
    InputError(&'static str, &'static str),

    /// The general error, given when the particular error doesn't
    /// fit into any of the other categories.
    #[error("error: {0}")]
    GeneralError(String),

    /// Occurs when there was an error parsing the URL.
    #[error("malformed url: {0}")]
    UrlParseError(#[from] url::ParseError),
}

/// The generic type is the return value. Otherwise, regardless of request type,
/// we're just returning the error string if there is an error.
pub type Result<T, E = WrapperError> = std::result::Result<T, E>;

/// A wrapper for [UCSD's WebReg](https://act.ucsd.edu/webreg2/start). For more information,
/// please see the README.
pub struct WebRegWrapper {
    cookies: String,
    client: Client,
    term: String,
}

impl WebRegWrapper {
    /// Creates a new instance of the `WebRegWrapper` with the specified `Client`, cookies, and
    /// term.
    ///
    /// After logging into WebReg, the cookies will be authenticated, but won't be associated
    /// with any specific term until you select a term.
    ///
    /// Once you select a term, your cookies will be "bound" to that term. This is especially
    /// important, since providing cookies to a non-matching terms (e.g., you provided cookies
    /// associated with SP22, but provided the term FA22) will result in cryptic error messages.
    ///
    /// You are expected to provide a
    /// [`reqwest::Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html). This
    /// can be as simple as the default client (`Client::new()`), or can be customized to suit
    /// your needs (e.g., by adding a timeout).
    ///
    /// # Parameters
    /// - `client`: The `reqwest` client.
    /// - `cookies`: The cookies from your session of WebReg.
    /// - `term`: The term *corresponding* to your `cookies`.
    ///
    /// # Returns
    /// The new instance of the `WebRegWrapper`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// let client = Client::new();
    /// let wrapper = WebRegWrapper::new(client, "my cookies".to_string(), "FA22");
    /// ```
    pub fn new(client: Client, cookies: String, term: impl Into<String>) -> Self {
        Self {
            cookies,
            client,
            term: term.into(),
        }
    }

    /// Sets the cookies to the new, specified cookies.
    ///
    /// This might be useful if you want to use the existing wrapper but need to change the
    /// cookies.
    ///
    /// # Parameters
    /// - `new_cookies`: The new cookies.
    pub fn set_cookies(&mut self, new_cookies: String) {
        self.cookies = new_cookies;
    }

    /// Checks if the current WebReg instance is valid. Specifically, this will check if you
    /// are logged in.
    ///
    /// # Returns
    /// `true` if the instance is valid and `false` otherwise.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// assert!(wrapper.is_valid().await);
    /// # }
    /// ```
    pub async fn is_valid(&self) -> bool {
        let res = self
            .client
            .get(WEBREG_BASE)
            .header(COOKIE, &self.cookies)
            .header(USER_AGENT, MY_USER_AGENT)
            .send()
            .await;

        match res {
            Err(_) => false,
            Ok(r) => match &r.text().await {
                Ok(res) => self.internal_is_valid(res),
                Err(_) => false,
            },
        }
    }

    /// Gets the name of the owner associated with this account.
    ///
    /// # Returns
    /// The name of the person, or an empty string if the cookies that were given were invalid.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// assert_eq!("Your name here", wrapper.get_account_name().await.unwrap());
    /// # }
    /// ```
    pub async fn get_account_name(&self) -> self::Result<String> {
        let res = self
            .client
            .get(ACC_NAME)
            .header(COOKIE, &self.cookies)
            .header(USER_AGENT, MY_USER_AGENT)
            .send()
            .await?;

        let name = res.text().await?;
        if self.internal_is_valid(&name) {
            Ok(name)
        } else {
            Ok("".into())
        }
    }

    /// Gets all prerequisites for a specified course for the term set by the wrapper.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_code`: The course code. For example, if you wanted to check `MATH 100B`, you
    /// would put `100B`.
    ///
    /// # Returns
    /// All prerequisites for the specified course. This is a structure that has two fields: one
    /// for all exam prerequisites, and one for all course prerequisites.
    ///
    ///
    /// ### Course Prerequisites
    ///
    /// This is a vector of vector of prerequisites, where each vector contains one or
    /// more prerequisites. Any prerequisites in the same vector means that you only need
    /// one of those prerequisites to fulfill that requirement.
    ///
    /// For example, if this value was `[[a, b], [c, d, e], [f]], then this means
    /// that you need
    /// - one of 'a' or 'b', *and*
    /// - one of 'c', 'd', or 'e', *and*
    /// - f.
    ///
    ///
    /// ### Exam Prerequisites
    /// Exam prerequisites will satisfy all of the requirements defined by course prerequisites.
    /// In other words, if you satisfy one of the exam prerequisites, you automatically satisfy
    /// all of the course prerequisites.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // Get all prerequisites
    /// let prereqs = wrapper.get_prereqs("MATH", "20C").await;
    /// match prereqs {
    ///     Ok(o) => {
    ///         println!("Exam prerequisites: {:?}", o.exam_prerequisites);
    ///         println!("Course prerequisites: {:?}", o.course_prerequisites);
    ///     },
    ///     Err(e) => eprintln!("Error when getting prerequisites! {}", e)
    /// };
    /// # }
    /// ```
    pub async fn get_prereqs(
        &self,
        subject_code: impl AsRef<str>,
        course_code: impl AsRef<str>,
    ) -> self::Result<PrerequisiteInfo> {
        let url = Url::parse_with_params(
            PREREQS_INFO,
            &[
                ("subjcode", subject_code.as_ref()),
                ("crsecode", course_code.as_ref()),
                ("termcode", self.term.as_str()),
                ("_", self.get_epoch_time().to_string().as_str()),
            ],
        )?;

        let res = self
            ._process_get_result::<Vec<RawPrerequisite>>(
                self.client
                    .get(url)
                    .header(COOKIE, &self.cookies)
                    .header(USER_AGENT, MY_USER_AGENT)
                    .send()
                    .await,
            )
            .await?;

        let mut all_reqs = PrerequisiteInfo {
            course_prerequisites: vec![],
            exam_prerequisites: vec![],
        };

        if res.is_empty() {
            return Ok(all_reqs);
        }

        let mut req_map: HashMap<&str, Vec<&RawCoursePrerequisite>> = HashMap::new();
        for r in &res {
            match r {
                RawPrerequisite::Course(c) => {
                    req_map.entry(&c.prereq_seq_id).or_insert(vec![]).push(c)
                }
                RawPrerequisite::Test(t) => all_reqs
                    .exam_prerequisites
                    .push(t.test_title.trim().to_string()),
            }
        }

        for (_, reqs) in req_map {
            let mut cleaned_reqs: Vec<CoursePrerequisite> = vec![];
            for req in reqs {
                cleaned_reqs.push(CoursePrerequisite {
                    subj_course_id: format!(
                        "{} {}",
                        req.subject_code.trim(),
                        req.course_code.trim()
                    ),
                    course_title: req.course_title.trim().to_string(),
                });
            }

            all_reqs.course_prerequisites.push(cleaned_reqs);
        }

        Ok(all_reqs)
    }

    /// Gets your current schedule.
    ///
    /// # Parameters
    /// - `schedule_name`: The schedule that you want to get. If `None` is given, this will default
    /// to your main schedule.
    ///
    /// # Returns
    /// Either a vector of sections that appear in your schedule, or an error message if something
    /// went wrong.
    ///
    /// # Examples
    ///
    /// Getting the default schedule.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // Pass in "None" for the default.
    /// let schedule = wrapper.get_schedule(None).await;
    /// match schedule {
    ///     Ok(s) => s.iter().for_each(|sec| println!("{}", sec.to_string())),
    ///     Err(e) => eprintln!("An error occurred! {}", e)
    /// }
    ///
    /// # }
    /// ```
    ///
    /// Getting the schedule with name "`Other Schedule`."
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // Pass in "None" for the default.
    /// let schedule = wrapper.get_schedule(Some("Other Schedule")).await;
    /// match schedule {
    ///     Ok(s) => s.iter().for_each(|sec| println!("{}", sec.to_string())),
    ///     Err(e) => eprintln!("An error occurred! {}", e)
    /// }
    /// # }
    /// ```
    pub async fn get_schedule(
        &self,
        schedule_name: Option<&str>,
    ) -> self::Result<Vec<ScheduledSection>> {
        let url = Url::parse_with_params(
            CURR_SCHEDULE,
            &[
                ("schedname", schedule_name.unwrap_or(DEFAULT_SCHEDULE_NAME)),
                ("final", ""),
                ("sectnum", ""),
                ("termcode", self.term.as_str()),
                ("_", self.get_epoch_time().to_string().as_str()),
            ],
        )?;

        let res = self
            ._process_get_result::<Vec<RawScheduledMeeting>>(
                self.client
                    .get(url)
                    .header(COOKIE, &self.cookies)
                    .header(USER_AGENT, MY_USER_AGENT)
                    .send()
                    .await,
            )
            .await?;

        if res.is_empty() {
            return Ok(vec![]);
        }

        // First, we separate the raw meetings based on whether it belongs to a special section
        // (a section whose section code is all numerical digits, e.g. section 001) OR a general
        // section.
        let mut base_group_secs: HashMap<&str, Vec<&RawScheduledMeeting>> = HashMap::new();
        let mut special_classes: HashMap<&str, Vec<&RawScheduledMeeting>> = HashMap::new();
        for s_meeting in &res {
            if s_meeting.enrolled_count == Some(0) && s_meeting.section_capacity == Some(0) {
                continue;
            }

            if s_meeting.sect_code.as_bytes()[0].is_ascii_digit() {
                special_classes
                    .entry(s_meeting.course_title.trim())
                    .or_insert_with(Vec::new)
                    .push(s_meeting);

                continue;
            }

            base_group_secs
                .entry(s_meeting.course_title.trim())
                .or_insert_with(Vec::new)
                .push(s_meeting);
        }

        let mut schedule: Vec<ScheduledSection> = vec![];

        // We next begin processing the general sections. Each key/value pair represents a course
        // section. We do not care about the key; the value is a vector of meetings, which we will
        // clean up.
        //
        // Every meeting is separated. For example, if we have a MWF meeting, then there will
        // be three meeting objects -- one for M, one for W, and one for F.
        for (_, sch_meetings) in base_group_secs {
            // First, let's get all instructors associated with this course section.
            let instructors = self.get_all_instructors(
                sch_meetings
                    .iter()
                    .flat_map(|x| self.get_instructor_names(&x.person_full_name)),
            );

            // Here, we want to find the main meetings. We note that the main meetings are the
            // ones which have a section code ending with 00 AND doesn't have a special meeting
            // associated with it (e.g., it's not a final exam or midterm).
            let all_main = sch_meetings
                .iter()
                .filter(|x| {
                    x.sect_code.ends_with("00")
                        && x.special_meeting.replace("TBA", "").trim().is_empty()
                })
                .collect::<Vec<_>>();

            // This should never be empty, since every section must have a main meeting.
            assert!(
                !all_main.is_empty()
                    && all_main
                        .iter()
                        .all(|x| x.meeting_type == all_main[0].meeting_type)
            );

            // We now parse the main meetings.
            let mut all_meetings: Vec<Meeting> = vec![];
            for main in all_main {
                all_meetings.push(Meeting {
                    meeting_type: main.meeting_type.to_string(),
                    meeting_days: if main.day_code.trim().is_empty() {
                        MeetingDay::None
                    } else {
                        MeetingDay::Repeated(util::parse_day_code(main.day_code.trim()))
                    },
                    start_min: main.start_time_min,
                    start_hr: main.start_time_hr,
                    end_min: main.end_time_min,
                    end_hr: main.end_time_hr,
                    building: main.bldg_code.trim().to_string(),
                    room: main.room_code.trim().to_string(),
                    instructors: self.get_instructor_names(&main.person_full_name),
                });
            }

            // Parse the remaining meetings.
            // Here, we want to parse any midterm and exam meetings.
            sch_meetings
                .iter()
                .filter(|x| {
                    x.sect_code.ends_with("00")
                        && !x.special_meeting.replace("TBA", "").trim().is_empty()
                })
                .map(|x| Meeting {
                    meeting_type: x.meeting_type.to_string(),
                    meeting_days: MeetingDay::OneTime(x.start_date.to_string()),
                    start_min: x.start_time_min,
                    start_hr: x.start_time_hr,
                    end_min: x.end_time_min,
                    end_hr: x.end_time_hr,
                    building: x.bldg_code.trim().to_string(),
                    room: x.room_code.trim().to_string(),
                    instructors: self.get_instructor_names(&x.person_full_name),
                })
                .for_each(|meeting| all_meetings.push(meeting));

            // Finally, we parse the general meetings.
            sch_meetings
                .iter()
                .filter(|x| !x.sect_code.ends_with("00"))
                .map(|x| Meeting {
                    meeting_type: x.meeting_type.to_string(),
                    meeting_days: MeetingDay::Repeated(util::parse_day_code(&x.day_code)),
                    start_min: x.start_time_min,
                    start_hr: x.start_time_hr,
                    end_min: x.end_time_min,
                    end_hr: x.end_time_hr,
                    building: x.bldg_code.trim().to_string(),
                    room: x.room_code.trim().to_string(),
                    instructors: self.get_instructor_names(&x.person_full_name),
                })
                .for_each(|meeting| all_meetings.push(meeting));

            // At this point, we now want to look for data like section capacity, number of
            // students on the waitlist, and so on.
            let wl_count = match sch_meetings.iter().find(|x| x.count_on_waitlist.is_some()) {
                Some(r) => match r.count_on_waitlist {
                    Some(o) => o,
                    None => {
                        return Err(WrapperError::GeneralError(
                            "no waitlist property found".into(),
                        ))
                    }
                },
                None => 0,
            };

            let pos_on_wl = if sch_meetings[0].enroll_status == "WT" {
                match sch_meetings
                    .iter()
                    .find(|x| x.waitlist_pos.chars().all(|y| y.is_numeric()))
                {
                    Some(r) => match r.waitlist_pos.parse::<i64>() {
                        Ok(o) => o,
                        Err(_) => {
                            return Err(WrapperError::GeneralError("waitlist not parsable.".into()))
                        }
                    },
                    None => 0,
                }
            } else {
                0
            };

            let enrolled_count = match sch_meetings.iter().find(|x| x.enrolled_count.is_some()) {
                Some(r) => r.enrolled_count.unwrap(),
                None => -1,
            };

            let section_capacity = match sch_meetings.iter().find(|x| x.section_capacity.is_some())
            {
                Some(r) => r.section_capacity.unwrap(),
                None => -1,
            };

            schedule.push(ScheduledSection {
                section_id: sch_meetings[0].section_id.to_string(),
                all_instructors: instructors.clone(),
                subject_code: sch_meetings[0].subj_code.trim().to_string(),
                course_code: sch_meetings[0].course_code.trim().to_string(),
                course_title: sch_meetings[0].course_title.trim().to_string(),
                section_code: match sch_meetings.iter().find(|x| !x.sect_code.ends_with("00")) {
                    Some(r) => r.sect_code.to_string(),
                    None => sch_meetings[0].sect_code.to_string(),
                },
                section_capacity,
                enrolled_count,
                available_seats: max(section_capacity - enrolled_count, 0),
                grade_option: sch_meetings[0].grade_option.trim().to_string(),
                units: sch_meetings[0].sect_credit_hrs,
                enrolled_status: match sch_meetings[0].enroll_status.as_str() {
                    "EN" => EnrollmentStatus::Enrolled,
                    "WT" => EnrollmentStatus::Waitlist(pos_on_wl),
                    "PL" => EnrollmentStatus::Planned,
                    _ => EnrollmentStatus::Unknown,
                },
                waitlist_ct: wl_count,
                meetings: all_meetings,
            });
        }

        // Now, we look into parsing the special sections. This is trivial to parse.
        for (_, sch_meetings) in special_classes {
            let day_code = sch_meetings
                .iter()
                .map(|x| x.day_code.trim())
                .collect::<Vec<_>>()
                .join("");

            let parsed_day_code = if day_code.is_empty() {
                MeetingDay::None
            } else {
                MeetingDay::Repeated(util::parse_day_code(&day_code))
            };

            let section_capacity = sch_meetings[0].section_capacity.unwrap_or(-1);
            let enrolled_count = sch_meetings[0].enrolled_count.unwrap_or(-1);

            schedule.push(ScheduledSection {
                section_id: sch_meetings[0].section_id.to_string(),
                all_instructors: self.get_all_instructors(
                    sch_meetings
                        .iter()
                        .flat_map(|x| self.get_instructor_names(&x.person_full_name)),
                ),
                subject_code: sch_meetings[0].subj_code.trim().to_string(),
                course_code: sch_meetings[0].course_code.trim().to_string(),
                course_title: sch_meetings[0].course_title.trim().to_string(),
                section_code: sch_meetings[0].sect_code.to_string(),
                section_capacity,
                enrolled_count,
                available_seats: max(section_capacity - enrolled_count, 0),
                grade_option: sch_meetings[0].grade_option.trim().to_string(),
                units: sch_meetings[0].sect_credit_hrs,
                enrolled_status: match sch_meetings[0].enroll_status.as_str() {
                    "EN" => EnrollmentStatus::Enrolled,
                    "WT" => EnrollmentStatus::Waitlist(-1),
                    "PL" => EnrollmentStatus::Planned,
                    _ => EnrollmentStatus::Unknown,
                },
                waitlist_ct: -1,
                meetings: vec![Meeting {
                    meeting_type: sch_meetings[0].meeting_type.to_string(),
                    meeting_days: parsed_day_code,
                    start_min: sch_meetings[0].start_time_min,
                    start_hr: sch_meetings[0].start_time_hr,
                    end_min: sch_meetings[0].end_time_min,
                    end_hr: sch_meetings[0].start_time_hr,
                    building: sch_meetings[0].bldg_code.trim().to_string(),
                    room: sch_meetings[0].room_code.trim().to_string(),
                    instructors: self.get_instructor_names(&sch_meetings[0].person_full_name),
                }],
            });
        }

        Ok(schedule)
    }

    /// Gets enrollment count for a particular course.
    ///
    /// Unlike the `get_course_info` function, this function only returns a vector of sections
    /// with the proper enrollment counts. Therefore, the `meetings` vector will always be
    /// empty as it is not relevant.
    ///
    /// Additionally, this function only returns one of some number of possible instructors.
    ///
    /// If you want full course information, use `get_course_info`. If you only care about the
    /// number of people enrolled in a section, this function is for you.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_code`: The course code. For example, if you wanted to check `MATH 100B`, you
    /// would put `100B`.
    ///
    /// # Returns
    /// Either a vector with all sections that match the given subject code & course code, or an
    /// error if one occurred.
    ///
    /// # Example
    /// Suppose we wanted to find all sections of CSE 101 for the sole purpose of seeing how
    /// many people are enrolled.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let sections = wrapper.get_enrollment_count("CSE", "101").await;
    /// match sections {
    ///     Ok(s) => s.iter().for_each(|sec| println!("{}", sec.to_string())),
    ///     Err(e) => eprintln!("An error occurred! {}", e)
    /// }
    /// # }
    /// ```
    pub async fn get_enrollment_count(
        &self,
        subject_code: impl AsRef<str>,
        course_code: impl AsRef<str>,
    ) -> self::Result<Vec<CourseSection>> {
        let crsc_code = self.get_formatted_course_code(course_code.as_ref());
        let url = Url::parse_with_params(
            COURSE_DATA,
            &[
                ("subjcode", subject_code.as_ref()),
                ("crsecode", crsc_code.as_ref()),
                ("termcode", self.term.as_str()),
                ("_", self.get_epoch_time().to_string().as_ref()),
            ],
        )?;

        let meetings = self
            ._process_get_result::<Vec<RawWebRegMeeting>>(
                self.client
                    .get(url)
                    .header(COOKIE, &self.cookies)
                    .header(USER_AGENT, MY_USER_AGENT)
                    .send()
                    .await,
            )
            .await?;

        // First, remove any duplicate meetings. For example, some courses may only have sections
        // with one lecture and one final exam meeting. Call this section section A00, so that
        // the lecture and final exam are both tagged as section code A00. Then, WebReg will
        // show both of these in the resulting JSON; additionally, they will both appear to be
        // enrollable (i.e., the `display_type` is `AC`).
        //
        // Note that if we are dealing with both a lecture and final exam meeting, then
        // both meeting structures will contain the same exact data (for our purposes);
        // this means that information like enrolled count, waitlist count, and so on will
        // be reflected across both structures accurately, so there's no need to search
        // for one particular meeting.
        let mut meetings_to_parse = vec![];
        let mut seen: HashSet<&str> = HashSet::new();
        for meeting in &meetings {
            if !seen.insert(meeting.sect_code.as_str()) {
                continue;
            }

            meetings_to_parse.push(meeting);
        }

        Ok(meetings_to_parse
            .into_iter()
            // Only want available sections, AC = displayed
            .filter(|x| x.display_type == "AC")
            .map(|x| CourseSection {
                subj_course_id: format!(
                    "{} {}",
                    subject_code.as_ref().trim(),
                    course_code.as_ref().trim()
                )
                .to_uppercase(),
                section_id: x.section_id.trim().to_string(),
                section_code: x.sect_code.trim().to_string(),
                all_instructors: self.get_instructor_names(&x.person_full_name),
                available_seats: max(x.avail_seat, 0),
                enrolled_ct: x.enrolled_count,
                total_seats: x.section_capacity,
                waitlist_ct: x.count_on_waitlist,
                meetings: vec![],
                needs_waitlist: x.needs_waitlist == "Y",
            })
            .collect())
    }

    /// Gets course information for a particular course.
    ///
    /// Note that WebReg provides this information in a way that makes it hard to use; in
    /// particular, WebReg separates each lecture, discussion, final exam, etc. from each other.
    /// This function attempts to figure out which lecture/discussion/final exam/etc. correspond
    /// to which section.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_code`: The course code. For example, if you wanted to check `MATH 100B`, you
    /// would put `100B`.
    ///
    /// # Returns
    /// A result containing either:
    /// - A vector with all possible sections that match the given subject code & course code.
    /// - Or the error that occurred.
    ///
    /// # Example
    /// Let's suppose we wanted to find all sections of CSE 105. This is how we would do this.
    /// Note that this will contain a lot of information.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let sections = wrapper.get_course_info("CSE", "105").await;
    /// match sections {
    ///     Ok(s) => s.iter().for_each(|sec| println!("{}", sec.to_string())),
    ///     Err(e) => eprintln!("An error occurred! {}", e)
    /// }
    /// # }
    /// ```
    pub async fn get_course_info(
        &self,
        subject_code: impl AsRef<str>,
        course_code: impl AsRef<str>,
    ) -> self::Result<Vec<CourseSection>> {
        let crsc_code = self.get_formatted_course_code(course_code.as_ref());
        let url = Url::parse_with_params(
            COURSE_DATA,
            &[
                ("subjcode", subject_code.as_ref()),
                ("crsecode", crsc_code.as_ref()),
                ("termcode", self.term.as_str()),
                ("_", self.get_epoch_time().to_string().as_ref()),
            ],
        )?;

        let parsed = self
            ._process_get_result::<Vec<RawWebRegMeeting>>(
                self.client
                    .get(url)
                    .header(COOKIE, &self.cookies)
                    .header(USER_AGENT, MY_USER_AGENT)
                    .send()
                    .await,
            )
            .await?;

        let course_dept_id = format!(
            "{} {}",
            subject_code.as_ref().trim(),
            course_code.as_ref().trim()
        )
        .to_uppercase();

        let mut sections: Vec<CourseSection> = vec![];
        let mut unprocessed_meetings: Vec<RawWebRegMeeting> = vec![];

        // First, let's determine which meetings only have numerical section codes (e.g., 001).
        // Generally, sections with numerical section codes will have ONE meeting, so if we find
        // any meetings here with numerical section code, then we can just call that a section
        // and easily process it.
        for meeting in parsed {
            // If the meeting is canceled, then we do not need to check anything else.
            // Likewise, if the section code doesn't exist, then we can't process it.
            if meeting.display_type == "CA" || meeting.sect_code.trim().is_empty() {
                continue;
            }

            // Next, we check to see if the meeting is a special meeting. To do so, we can just
            // check to make sure the first character in the section code is a digit (e.g. *0*01)
            if meeting.sect_code.as_bytes()[0].is_ascii_digit() {
                let (m_type, m_days) = util::parse_meeting_type_date(&meeting);
                sections.push(CourseSection {
                    subj_course_id: course_dept_id.clone(),
                    section_id: meeting.section_id.trim().to_string(),
                    section_code: meeting.sect_code.trim().to_string(),
                    all_instructors: self.get_instructor_names(&meeting.person_full_name),
                    // Because it turns out that you can have negative available seats.
                    available_seats: max(meeting.avail_seat, 0),
                    enrolled_ct: meeting.enrolled_count,
                    total_seats: meeting.section_capacity,
                    waitlist_ct: meeting.count_on_waitlist,
                    needs_waitlist: meeting.needs_waitlist == "Y",
                    meetings: vec![Meeting {
                        start_hr: meeting.start_time_hr,
                        start_min: meeting.start_time_min,
                        end_hr: meeting.end_time_hr,
                        end_min: meeting.end_time_min,
                        meeting_type: m_type.to_string(),
                        meeting_days: m_days,
                        building: meeting.bldg_code.trim().to_string(),
                        room: meeting.room_code.trim().to_string(),
                        instructors: self.get_instructor_names(&meeting.person_full_name),
                    }],
                });

                continue;
            }

            // If this wasn't a special meeting, we can process it later.
            unprocessed_meetings.push(meeting);
        }

        // If there is nothing left to process, then we're done!
        if unprocessed_meetings.is_empty() {
            return Ok(sections);
        }

        // Otherwise, we need to deal with non-special meetings. Remember that these are all
        // scattered (e.g. one meeting may represent one discussion, another meeting may represent
        // a midterm for a completely different section, etc.)
        //
        // We create a map to categorize each meeting by their meeting code. The key, then, will
        // be the section code family (e.g., for section A01, its family will be 'A') and the
        // value will be the corresponding meetings.
        let mut map: HashMap<char, GroupedSection<RawWebRegMeeting>> = HashMap::new();
        for meeting in &unprocessed_meetings {
            // Get the section family, which *should* exist (i.e., no panic should occur here).
            let sec_fam = meeting
                .sect_code
                .chars()
                .next()
                .expect("Non-existent section code.");

            let entry = map.entry(sec_fam).or_insert(GroupedSection {
                child_meetings: vec![],
                general_meetings: vec![],
            });

            // If the meeting's code ends with '00' then it is automatically a general meeting.
            // This includes lectures, final exams, and other similar meetings.
            // Note that if a section ONLY has a lecture and final exam, both lecture and
            // final exam meeting will show up as "enrollable" (i.e., the `display_type` is `AC`),
            // so we want to catch those meetings here first instead of in the match statement
            // below.
            if meeting.sect_code.ends_with("00") {
                entry.general_meetings.push(meeting);
                continue;
            }

            // Otherwise, we can check everything else.
            match meeting.display_type.as_str() {
                // AC = Enrollable (usually discussion sections).
                "AC" => entry.child_meetings.push(meeting),

                // NC = Cannot be enrolled in (usually lectures, final exams).
                //
                // The reason why we have this is because some courses, like CSE 8A, will have
                // labs and discussions. Here, students can enroll in labs (often with section
                // codes like A50, A51, and so on). However, the discussions are not enrollable and
                // so they will have the `NC` display type. However, unlike lectures, final exams,
                // and related, these discussion sections will have section codes like A01, A02,
                // and so on.
                "NC" => entry.general_meetings.push(meeting),
                _ => continue,
            };
        }

        // Sort the keys so that section A is first, then section B, and so on.
        let mut keys: Vec<_> = map.keys().collect();
        keys.sort();

        // Now that we have all of the meetings, categorizing should be easier.
        for key in keys {
            // We're making a bold assumption that the `general_meetings` vector will never be
            // empty. However, we note from various courses that sections will *always* either
            // have at least ONE meeting with
            // - section code X00 (where X is a letter), or         handled here.
            // - 0YY (where Y is a digit)                           handled above.
            // In other words, it's *very* unlikely that we'll see a section where there's no
            // meeting that meets the above patterns, so we have little to hopefully worry about.
            let entry = &map[key];
            if entry.general_meetings.is_empty() {
                // This should never hit, but sanity check nonetheless.
                dbg!(&course_dept_id);
                continue;
            }

            // First, get the base instructors. These are all of the instructors for the lectures.
            // Note that, for a majority of courses, there will only be one instructor. However,
            // some courses may have two or more instructors.
            let base_instructors = self.get_all_instructors(
                entry
                    .general_meetings
                    .iter()
                    .flat_map(|x| self.get_instructor_names(&x.person_full_name)),
            );

            // Define a closure that takes in a slice `from` (which is a slice of all meetings that
            // we want to read in) and a vector `to` (which is where we want to write these
            // meetings to).
            let process_meetings = |from: &[&RawWebRegMeeting], to: &mut Vec<Meeting>| {
                for meeting in from {
                    let (m_m_type, m_days) = util::parse_meeting_type_date(meeting);

                    to.push(Meeting {
                        meeting_type: m_m_type.to_string(),
                        meeting_days: m_days,
                        building: meeting.bldg_code.trim().to_string(),
                        room: meeting.room_code.trim().to_string(),
                        start_hr: meeting.start_time_hr,
                        start_min: meeting.start_time_min,
                        end_hr: meeting.end_time_hr,
                        end_min: meeting.end_time_min,
                        // These are instructors specifically assigned to this meeting. For most
                        // cases, these will be the same instructors assigned to the lecture
                        // meetings.
                        instructors: self.get_instructor_names(&meeting.person_full_name),
                    });
                }
            };

            // If there are no child meetings, then this means we only have lecture + exams.
            if entry.child_meetings.is_empty() {
                // Note that the general meetings vector will contain a lecture (and maybe a
                // final exam) meeting. If it contains both a lecture and final exam meeting, then
                // both meeting structures will contain the same exact data (for our purposes);
                // this means that information like enrolled count, waitlist count, and so on will
                // be reflected across both structures accurately, so there's no need to search
                // for one particular meeting.
                let mut section = CourseSection {
                    subj_course_id: course_dept_id.clone(),
                    section_id: entry.general_meetings[0].section_id.clone(),
                    section_code: entry.general_meetings[0].sect_code.clone(),
                    all_instructors: self
                        .get_instructor_names(&entry.general_meetings[0].person_full_name),
                    available_seats: max(entry.general_meetings[0].avail_seat, 0),
                    enrolled_ct: entry.general_meetings[0].enrolled_count,
                    total_seats: entry.general_meetings[0].section_capacity,
                    waitlist_ct: entry.general_meetings[0].count_on_waitlist,
                    meetings: vec![],
                    needs_waitlist: entry.general_meetings[0].needs_waitlist == "Y",
                };

                // Then, iterate through the rest of the general meetings.
                process_meetings(&entry.general_meetings, &mut section.meetings);
                // Finally, add it to the sections.
                sections.push(section);
                continue;
            }

            // Otherwise, we essentially repeat the same process above. The only difference is that
            // we clone 'section' for each child meeting.
            for c_meeting in &entry.child_meetings {
                let mut instructors = base_instructors.clone();
                instructors.append(&mut self.get_instructor_names(&c_meeting.person_full_name));
                instructors.sort();
                instructors.dedup();

                // Process the general section info.
                let mut section = CourseSection {
                    subj_course_id: course_dept_id.clone(),
                    section_id: c_meeting.section_id.clone(),
                    section_code: c_meeting.sect_code.clone(),
                    all_instructors: instructors,
                    available_seats: max(c_meeting.avail_seat, 0),
                    enrolled_ct: c_meeting.enrolled_count,
                    total_seats: c_meeting.section_capacity,
                    waitlist_ct: c_meeting.count_on_waitlist,
                    meetings: vec![],
                    needs_waitlist: c_meeting.needs_waitlist == "Y",
                };

                // Iterate through the general and child meetings.
                process_meetings(&entry.general_meetings, &mut section.meetings);
                process_meetings(&[c_meeting], &mut section.meetings);
                // Finally, add it to the sections as usual.
                sections.push(section);
            }
        }

        Ok(sections)
    }

    /// Gets all courses that are available. This searches for all courses via Webreg's menu, but
    /// then also searches each course found for specific details. This essentially calls the two
    /// functions [`WebRegWrapper::search_courses`] and [`WebRegWrapper::get_course_info`].
    ///
    /// # Parameters
    /// - `filter_by`: The request filter.
    ///
    /// # Returns
    /// Either a vector consisting of all courses that are available, with detailed information,
    /// or the error that was encountered.
    ///
    /// # Warning
    /// This function call will make *many* API requests. Thus, searching for many classes is not
    /// recommended as you may get rate-limited.
    ///
    /// # Example
    /// Searching for a section by section ID; in this example, we're looking for one section
    /// with the section ID `1234567`. If this is successful, then we will get a vector back
    /// with *one* element (since section IDs are unique).
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{SearchType, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let res = wrapper.search_courses_detailed(SearchType::BySection("1234567")).await;
    ///
    /// // We should only find one section at most.
    /// if let Ok(results) = res {
    ///     assert!(results.len() <= 1);
    /// }
    /// # }
    /// ```
    ///
    /// Searching for multiple sections by an array (slice) of IDs. If there are `N` section IDs
    /// provided, then you should get *at most* `N` results back. In this example, we're looking
    /// for any sections with IDs `1234567` or `115123` or `2135`.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{SearchType, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let res = wrapper
    ///     .search_courses_detailed(SearchType::ByMultipleSections(&[
    ///         "1234567", "115123", "2135",
    ///     ]))
    ///     .await;
    ///
    /// // We should only find 3 sections at most.
    /// if let Ok(results) = res {
    ///     assert!(results.len() <= 1);
    /// }
    /// # }
    /// ```
    ///
    /// Searching by many different options. In this example, we're searching for courses that are:
    /// - either upper-division or lower-division,
    /// - offered in the CSE or MATH department,
    /// - on Monday, Wednesday, and Friday,
    /// - starts no earlier than 10:00am, *and*
    /// - ends no later than 5:30pm.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{
    ///     CourseLevelFilter, DayOfWeek, SearchRequestBuilder, SearchType, WebRegWrapper
    /// };
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let res = wrapper
    ///     .search_courses_detailed(SearchType::Advanced(
    ///         &SearchRequestBuilder::new()
    ///             .filter_courses_by(CourseLevelFilter::LowerDivision)
    ///             .filter_courses_by(CourseLevelFilter::UpperDivision)
    ///             .add_department("CSE")
    ///             .add_department("MATH")
    ///             .apply_day(DayOfWeek::Monday)
    ///             .apply_day(DayOfWeek::Wednesday)
    ///             .apply_day(DayOfWeek::Friday)
    ///             .set_start_time(10, 0)
    ///             .set_end_time(12 + 5, 30)
    ///     ))
    ///     .await;
    ///
    /// // Who knows how many sections we'll find?
    /// if let Ok(results) = res {
    ///     assert!(results.len() <= 1);
    /// }
    /// # }
    /// ```
    pub async fn search_courses_detailed(
        &self,
        filter_by: SearchType<'_>,
    ) -> self::Result<Vec<CourseSection>> {
        let get_zero_trim = |s: &[u8]| -> (usize, usize) {
            let start = s.iter().position(|p| *p != b'0').unwrap_or(0);
            let end = s.iter().rposition(|p| *p != b'0').unwrap_or(0);
            // "0001000" -> (3, 4)  | "0001000"[3..4] = "1"
            // "0000" -> (0, 0)     | "0000"[0..0] = ""
            // "00100100" -> (2, 6) | "00100100"[2..6] = "1001"
            (
                start,
                if start == end && start == 0 {
                    0
                } else {
                    end + 1
                },
            )
        };

        let mut ids_to_filter = vec![];
        match filter_by {
            SearchType::BySection(s) => {
                let (start, end) = get_zero_trim(s.as_bytes());
                ids_to_filter.push(&s[start..end]);
            }
            SearchType::ByMultipleSections(s) => {
                s.iter().for_each(|t| {
                    let (start, end) = get_zero_trim(t.as_bytes());
                    ids_to_filter.push(&t[start..end]);
                });
            }
            SearchType::Advanced(_) => {}
        };

        let search_res = self.search_courses(filter_by).await?;

        let mut vec: Vec<CourseSection> = vec![];
        for r in search_res {
            let req_res = self
                .get_course_info(r.subj_code.trim(), r.course_code.trim())
                .await;
            match req_res {
                Ok(r) => r.into_iter().for_each(|x| {
                    if !ids_to_filter.is_empty() {
                        let (start, end) = get_zero_trim(x.section_id.as_bytes());
                        if !ids_to_filter.contains(&&x.section_id.as_str()[start..end]) {
                            return;
                        }
                    }
                    vec.push(x);
                }),
                Err(_) => break,
            };
        }

        Ok(vec)
    }

    /// Gets all courses that are available. All this does is searches for all courses via Webreg's
    /// menu. Thus, only basic details are shown.
    ///
    /// # Parameters
    /// - `filter_by`: The request filter.
    ///
    /// # Returns
    /// A vector consisting of all courses that are available. Note that the data that is returned
    /// is directly from WebReg's API, so care will need to be taken to clean the resulting data.
    ///
    /// # Example
    /// Please see [`WebWegWrapper::search_courses_detailed`] for examples.
    pub async fn search_courses(
        &self,
        filter_by: SearchType<'_>,
    ) -> self::Result<Vec<RawWebRegSearchResultItem>> {
        let url = match filter_by {
            SearchType::BySection(section) => Url::parse_with_params(
                WEBREG_SEARCH_SEC,
                &[("sectionid", section), ("termcode", self.term.as_str())],
            )?,
            SearchType::ByMultipleSections(sections) => Url::parse_with_params(
                WEBREG_SEARCH_SEC,
                &[
                    ("sectionid", sections.join(":").as_str()),
                    ("termcode", self.term.as_str()),
                ],
            )?,
            SearchType::Advanced(request_filter) => {
                let subject_code = if request_filter.subjects.is_empty() {
                    "".to_string()
                } else {
                    request_filter.subjects.join(":")
                };

                let course_code = if request_filter.courses.is_empty() {
                    "".to_string()
                } else {
                    // This can probably be made significantly more efficient
                    request_filter
                        .courses
                        .iter()
                        .map(|x| x.split_whitespace().collect::<Vec<_>>())
                        .map(|course| {
                            course
                                .into_iter()
                                .map(|x| self.get_formatted_course_code(x))
                                .collect::<Vec<_>>()
                                .join(":")
                        })
                        .collect::<Vec<_>>()
                        .join(";")
                        .to_uppercase()
                };

                let department = if request_filter.departments.is_empty() {
                    "".to_string()
                } else {
                    request_filter.departments.join(":")
                };

                let professor = match request_filter.instructor {
                    Some(r) => r.to_uppercase(),
                    None => "".to_string(),
                };

                let title = match request_filter.title {
                    Some(r) => r.to_uppercase(),
                    None => "".to_string(),
                };

                let levels = if request_filter.level_filter == 0 {
                    "".to_string()
                } else {
                    // Needs to be exactly 12 digits
                    let mut s = format!("{:b}", request_filter.level_filter);
                    while s.len() < 12 {
                        s.insert(0, '0');
                    }

                    s
                };

                let days = if request_filter.days == 0 {
                    "".to_string()
                } else {
                    // Needs to be exactly 7 digits
                    let mut s = format!("{:b}", request_filter.days);
                    while s.len() < 7 {
                        s.insert(0, '0');
                    }

                    s
                };

                let time_str = {
                    if request_filter.start_time.is_none() && request_filter.end_time.is_none() {
                        "".to_string()
                    } else {
                        let start_time = match request_filter.start_time {
                            Some((h, m)) => format!("{:0>2}{:0>2}", h, m),
                            None => "".to_string(),
                        };

                        let end_time = match request_filter.end_time {
                            Some((h, m)) => format!("{:0>2}{:0>2}", h, m),
                            None => "".to_string(),
                        };

                        format!("{}:{}", start_time, end_time)
                    }
                };

                Url::parse_with_params(
                    WEBREG_SEARCH,
                    &[
                        ("subjcode", subject_code.as_str()),
                        ("crsecode", course_code.as_str()),
                        ("department", department.as_str()),
                        ("professor", professor.as_str()),
                        ("title", title.as_str()),
                        ("levels", levels.as_str()),
                        ("days", days.as_str()),
                        ("timestr", time_str.as_str()),
                        (
                            "opensection",
                            if request_filter.only_open {
                                "true"
                            } else {
                                "false"
                            },
                        ),
                        ("isbasic", "true"),
                        ("basicsearchvalue", ""),
                        ("termcode", self.term.as_str()),
                        ("_", self.get_epoch_time().to_string().as_str()),
                    ],
                )?
            }
        };

        self._process_get_result::<Vec<RawWebRegSearchResultItem>>(
            self.client
                .get(url)
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Sends an email to yourself using the same email that is used to confirm that you have
    /// enrolled or waitlisted in a particular class. In other words, this will send an email
    /// to you through the email NoReplyRegistrar@ucsd.edu.
    ///
    /// It is strongly recommended that this function not be abused.
    ///
    /// # Parameters
    /// - `email_content`: The email to send.
    ///
    /// # Returns
    /// `true` if the email was sent successfully and `false` otherwise.
    ///
    /// # Example
    /// This will send an email to yourself with the content specified as the string shown below.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let res = wrapper
    ///     .send_email_to_self("Hello, world! This will be sent to you via email.")
    ///     .await;
    ///
    /// match res {
    ///     Ok(_) => println!("Sent successfully."),
    ///     Err(e) => eprintln!("Error! {}", e)
    /// };
    /// # }
    /// ```
    pub async fn send_email_to_self(&self, email_content: &str) -> self::Result<()> {
        let r = self
            .client
            .post(SEND_EMAIL)
            .form(&[
                ("actionevent", email_content),
                ("termcode", self.term.as_str()),
            ])
            .header(COOKIE, &self.cookies)
            .header(USER_AGENT, MY_USER_AGENT)
            .send()
            .await?;

        if !r.status().is_success() {
            return Err(WrapperError::BadStatusCode(r.status().as_u16()));
        }

        let t = r.text().await?;
        if t.contains("\"YES\"") {
            Ok(())
        } else {
            Err(WrapperError::WebRegError(t))
        }
    }

    /// Changes the grading option for the class corresponding to the section ID.
    ///
    /// # Parameters
    /// - `section_id`: The section ID corresponding to the class that you want to change
    /// the grading option for.
    /// - `new_grade_opt`: The new grading option. This must either be `L` (letter),
    /// `P` (pass/no pass), or `S` (satisfactory/unsatisfactory), and is enforced via an enum.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Changing the section associated with section ID `12345` to letter grading option.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{GradeOption, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let res = wrapper.change_grading_option("12345", GradeOption::L).await;
    ///
    /// match res {
    ///     Ok(_) => println!("Success!"),
    ///     Err(e) => eprintln!("Something went wrong: {}", e)
    /// }
    /// # }
    /// ```
    pub async fn change_grading_option(
        &self,
        section_id: &str,
        new_grade_opt: GradeOption,
    ) -> self::Result<bool> {
        let new_grade_opt = match new_grade_opt {
            GradeOption::L => "L",
            GradeOption::S => "S",
            GradeOption::P => "P",
        };

        // "Slice" any zeros off of the left-most side of the string. We need to do this
        // because, when comparing section IDs in the schedule, WebReg gives us the
        // section IDs as integers; however, for the rest of the API, it's given as a
        // string.
        //
        // Essentially, this means that, while most of WebReg's API will take `"079911"` as
        // an input and as an output (e.g. see `get_course_info`), the schedule API will
        // specifically return an integer `79911`. The `get_schedule` function will simply
        // convert this integer to a string, e.g. `79911` -> `"79911"` and return that along
        // with the other parsed info for each scheduled section.
        //
        // So, we need to slice off any 0s from the input parameter `section_id` to account
        // for this.
        let mut left_idx = 0;
        for c in section_id.chars() {
            if c != '0' {
                break;
            }

            left_idx += 1;
            continue;
        }

        let poss_class = self
            .get_schedule(None as Option<&str>)
            .await?
            .into_iter()
            .find(|x| x.section_id == section_id[left_idx..]);

        // don't care about previous poss_class
        let poss_class = match poss_class {
            Some(s) => s,
            None => return Err(WrapperError::GeneralError("Class not found.".into())),
        };

        let sec_id = poss_class.section_id.to_string();
        let units = poss_class.units.to_string();

        self._process_post_response(
            self.client
                .post(CHANGE_ENROLL)
                .form(&[
                    ("section", sec_id.as_str()),
                    ("subjCode", ""),
                    ("crseCode", ""),
                    ("unit", units.as_str()),
                    ("grade", new_grade_opt),
                    // You don't actually need these
                    ("oldGrade", ""),
                    ("oldUnit", ""),
                    ("termcode", self.term.as_str()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Validates that adding a course to your plan will cause no issue.
    ///
    /// # Parameters
    /// - `plan_options`: Information for the course that you want to plan.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// an issue appears.
    ///
    /// # Example
    /// Here, we will add the course `CSE 100`, which has section ID `079911` and section code
    /// `A01`, to our plan.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{GradeOption, PlanAdd, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    ///
    /// let res = wrapper.validate_add_to_plan(&PlanAdd {
    ///     subject_code: "CSE",
    ///     course_code: "100",
    ///     section_id: "079911",
    ///     section_code: "A01",
    ///     // Using S/U grading.
    ///     grading_option: Some(GradeOption::S),
    ///     // Put in default schedule
    ///     schedule_name: None,
    ///     unit_count: 4
    /// }).await;
    ///
    /// match res {
    ///     Ok(o) => println!("{}", if o { "Successful, planning is good" } else { "Unsuccessful" }),
    ///     Err(e) => eprintln!("{}", e),
    /// };
    /// # }
    /// ```
    pub async fn validate_add_to_plan(&self, plan_options: &PlanAdd<'_>) -> self::Result<bool> {
        let crsc_code = self.get_formatted_course_code(plan_options.course_code);
        self._process_post_response(
            self.client
                .post(PLAN_EDIT)
                .form(&[
                    ("section", plan_options.section_id),
                    ("subjcode", plan_options.subject_code),
                    ("crsecode", crsc_code.as_str()),
                    ("termcode", self.term.as_str()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Allows you to plan a course.
    ///
    /// # Parameters
    /// - `plan_options`: Information for the course that you want to plan.
    /// - `validate`: Whether to validate your planning of this course beforehand.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Warning
    /// Setting the `validate` parameter to `false` can cause issues. For example, when this is
    /// `false`, you will be able to plan courses with more units than allowed (e.g. 42 units), set
    /// the rading option to one that you are not allowed to use (e.g. S/U as an undergraduate),
    /// and only enroll in specific components of a section (e.g. just the discussion section).
    /// Some of these options can visually break WebReg (e.g. Remove/Enroll button will not appear).
    ///
    /// # Example
    /// Here, we will add the course `CSE 100`, which has section ID `079911` and section code
    /// `A01`, to our plan.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{GradeOption, PlanAdd, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    ///
    /// let res = wrapper.add_to_plan(PlanAdd {
    ///     subject_code: "CSE",
    ///     course_code: "100",
    ///     section_id: "079911",
    ///     section_code: "A01",
    ///     // Using S/U grading.
    ///     grading_option: Some(GradeOption::S),
    ///     // Put in default schedule
    ///     schedule_name: None,
    ///     unit_count: 4
    /// }, true).await;
    ///
    /// match res {
    ///     Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    ///     Err(e) => eprintln!("{}", e),
    /// };
    /// # }
    /// ```
    pub async fn add_to_plan(
        &self,
        plan_options: PlanAdd<'_>,
        validate: bool,
    ) -> self::Result<bool> {
        let u = plan_options.unit_count.to_string();
        let crsc_code = self.get_formatted_course_code(plan_options.course_code);

        if validate {
            // We need to call the edit endpoint first, or else we'll have issues where we don't
            // actually enroll in every component of the course.
            // Also, this can potentially return "false" due to you not being able to enroll in the
            // class, e.g. the class you're trying to plan is a major-restricted class.
            self.validate_add_to_plan(&plan_options)
                .await
                .unwrap_or(false);
        }

        self._process_post_response(
            self.client
                .post(PLAN_ADD)
                .form(&[
                    ("subjcode", plan_options.subject_code),
                    ("crsecode", crsc_code.as_str()),
                    ("sectnum", plan_options.section_id),
                    ("sectcode", plan_options.section_code),
                    ("unit", u.as_str()),
                    (
                        "grade",
                        match plan_options.grading_option {
                            Some(r) => match r {
                                GradeOption::L => "L",
                                GradeOption::S => "S",
                                GradeOption::P => "P",
                            },
                            _ => "L",
                        },
                    ),
                    ("termcode", self.term.as_str()),
                    (
                        "schedname",
                        match plan_options.schedule_name {
                            Some(r) => r,
                            None => DEFAULT_SCHEDULE_NAME,
                        },
                    ),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Allows you to unplan a course.
    ///
    /// # Parameters
    /// - `section_id`: The section ID.
    /// - `schedule_name`: The schedule name where the course should be unplanned from.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Here, we will remove the planned course with section ID `079911` from our default schedule.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{GradeOption, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let res = wrapper.remove_from_plan("079911", None).await;
    /// match res {
    ///     Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    ///     Err(e) => eprintln!("{}", e),
    /// };
    /// # }
    /// ```
    pub async fn remove_from_plan(
        &self,
        section_id: impl AsRef<str>,
        schedule_name: Option<&str>,
    ) -> self::Result<bool> {
        self._process_post_response(
            self.client
                .post(PLAN_REMOVE)
                .form(&[
                    ("sectnum", section_id.as_ref()),
                    ("termcode", self.term.as_str()),
                    ("schedname", schedule_name.unwrap_or(DEFAULT_SCHEDULE_NAME)),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Validates that the section that you are trying to enroll in is valid.
    ///
    /// # Parameters
    /// - `is_enroll`: Whether you are enrolling.
    /// - `enroll_options`: The enrollment options. Note that the section ID is the only thing
    /// that matters here. A reference, thus, is expected since you will probably be reusing
    /// the structure when calling the `add_section` function.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// there is an issue when trying to enroll.
    ///
    /// # Example
    /// Here, we will enroll in the course with section ID `078616`, and with the default grading
    /// option and unit count.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{EnrollWaitAdd, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    ///
    /// let enroll_options = EnrollWaitAdd {
    ///     section_id: "078616",
    ///     // Use default grade option
    ///     grading_option: None,
    ///     // Use default unit count
    ///     unit_count: None,
    /// };
    ///
    /// let add_res = wrapper
    ///     .validate_add_section(
    ///         // Use true here since we want to enroll (not waitlist). Note that this might
    ///         // result in an `Err` being returned if you can't enroll.
    ///         true,
    ///         &enroll_options,
    ///     )
    ///     .await;
    ///
    /// match add_res {
    ///     Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    ///     Err(e) => eprintln!("{}", e),
    /// };
    /// # }
    /// ```
    pub async fn validate_add_section(
        &self,
        is_enroll: bool,
        enroll_options: &EnrollWaitAdd<'_>,
    ) -> self::Result<bool> {
        let base_edit_url = if is_enroll {
            ENROLL_EDIT
        } else {
            WAITLIST_EDIT
        };

        self._process_post_response(
            self.client
                .post(base_edit_url)
                .form(&[
                    // These are required
                    ("section", enroll_options.section_id),
                    ("termcode", self.term.as_str()),
                    // These are optional.
                    ("subjcode", ""),
                    ("crsecode", ""),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Enrolls in, or waitlists, a class.
    ///
    /// # Parameters
    /// - `is_enroll`: Whether you want to enroll. This should be `true` if you want to enroll
    /// in this section and `false` if you want to waitlist.
    /// - `enroll_options`: Information for the course that you want to enroll in.
    /// - `validate`: Whether to validate your enrollment of this course beforehand. Note that
    /// validation is required, so this should be `true`. This should only be `false` if you
    /// called `validate_add_section` before. If you attempt to call `add_section` without
    /// validation, then you will get an error.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Here, we will enroll in the course with section ID `078616`, and with the default grading
    /// option and unit count.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{EnrollWaitAdd, WebRegWrapper};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    ///
    /// let add_res = wrapper
    ///     .add_section(
    ///         // Use true here since we want to enroll (not waitlist). Note that this might
    ///         // result in an `Err` being returned if you can't enroll.
    ///         true,
    ///         EnrollWaitAdd {
    ///             section_id: "078616",
    ///             // Use default grade option
    ///             grading_option: None,
    ///             // Use default unit count
    ///             unit_count: None,
    ///         },
    ///         true,
    ///     )
    ///     .await;
    ///
    /// match add_res {
    ///     Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    ///     Err(e) => eprintln!("{}", e),
    /// };
    /// # }
    /// ```
    pub async fn add_section(
        &self,
        is_enroll: bool,
        enroll_options: EnrollWaitAdd<'_>,
        validate: bool,
    ) -> self::Result<bool> {
        let base_reg_url = if is_enroll { ENROLL_ADD } else { WAITLIST_ADD };
        let u = match enroll_options.unit_count {
            Some(r) => r.to_string(),
            None => "".to_string(),
        };

        if validate {
            self.validate_add_section(is_enroll, &enroll_options)
                .await?;
        }

        self._process_post_response(
            self.client
                .post(base_reg_url)
                .form(&[
                    // These are required
                    ("section", enroll_options.section_id),
                    ("termcode", self.term.as_str()),
                    // These are optional.
                    ("unit", u.as_str()),
                    (
                        "grade",
                        match enroll_options.grading_option {
                            Some(r) => match r {
                                GradeOption::L => "L",
                                GradeOption::S => "S",
                                GradeOption::P => "P",
                            },
                            _ => "",
                        },
                    ),
                    ("crsecode", ""),
                    ("subjcode", ""),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await?;

        // This will always return true
        self._process_post_response(
            self.client
                .post(PLAN_REMOVE_ALL)
                .form(&[
                    ("sectnum", enroll_options.section_id),
                    ("termcode", self.term.as_str()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Drops a section.
    ///
    /// # Parameters
    /// - `was_enrolled`: Whether you were originally enrolled in the section. This would
    /// be `true` if you were enrolled and `false` if waitlisted.
    /// - `section_id`: The section ID corresponding to the section that you want to drop.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Remarks
    /// It is a good idea to make a call to get your current schedule before you
    /// make a request here. That way, you know which classes can be dropped.
    ///
    /// # Example
    /// Here, we assume that we are enrolled in a course with section ID `078616`, and want to
    /// drop it.
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    ///
    /// // Use `true` here since we were originally enrolled (not waitlisted).
    /// let drop_res = wrapper.drop_section(true, "078616").await;
    /// match drop_res {
    ///     Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    ///     Err(e) => eprintln!("{}", e),
    /// };
    /// # }
    /// ```
    pub async fn drop_section(
        &self,
        was_enrolled: bool,
        section_id: impl AsRef<str>,
    ) -> self::Result<bool> {
        let base_reg_url = if was_enrolled {
            ENROLL_DROP
        } else {
            WAILIST_DROP
        };

        self._process_post_response(
            self.client
                .post(base_reg_url)
                .form(&[
                    // These parameters are optional
                    ("subjcode", ""),
                    ("crsecode", ""),
                    // But these are required
                    ("section", section_id.as_ref()),
                    ("termcode", self.term.as_str()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Pings the WebReg server. Presumably, this is the endpoint that is used to ensure that
    /// your (authenticated) session is still valid. In other words, if this isn't called, I
    /// assume that you will be logged out, rendering your cookies invalid.
    ///
    /// # Returns
    /// `true` if the ping was successful and `false` otherwise.
    pub async fn ping_server(&self) -> bool {
        let res = self
            .client
            .get(format!("{}?_={}", PING_SERVER, self.get_epoch_time()))
            .header(COOKIE, &self.cookies)
            .header(USER_AGENT, MY_USER_AGENT)
            .send()
            .await;

        match res {
            Err(_) => false,
            Ok(r) => {
                let text = r.text().await.unwrap_or_else(|_| {
                    json!({
                        "SESSION_OK": false
                    })
                    .to_string()
                });

                let json: Value = serde_json::from_str(&text).unwrap_or_default();
                // Use of unwrap here is safe since we know that there is a boolean value beforehand
                json["SESSION_OK"].is_boolean() && json["SESSION_OK"].as_bool().unwrap()
            }
        }
    }

    /// Renames a schedule to the specified name. You cannot rename the default
    /// `My Schedule` schedule.
    ///
    /// # Parameter
    /// - `old_name`: The name of the old schedule.
    /// - `new_name`: The name that you want to change the old name to.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Renaming the schedule "`Test Schedule`" to "`Another Schedule`."
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // You should do error handling here, but I won't
    /// assert!(!wrapper.get_schedule_list().await.unwrap().contains(&"Another Schedule".to_string()));
    /// wrapper.rename_schedule("Test Schedule", "Another Schedule").await;
    /// assert!(wrapper.get_schedule_list().await.unwrap().contains(&"Another Schedule".to_string()));
    /// # }
    /// ```
    pub async fn rename_schedule(
        &self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> self::Result<bool> {
        // Can't rename your default schedule.
        if old_name.as_ref() == DEFAULT_SCHEDULE_NAME {
            return Err(WrapperError::InputError(
                "old_name",
                "You cannot rename the default schedule",
            ));
        }

        self._process_post_response(
            self.client
                .post(RENAME_SCHEDULE)
                .form(&[
                    ("termcode", self.term.as_str()),
                    ("oldschedname", old_name.as_ref()),
                    ("newschedname", new_name.as_ref()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Removes a schedule. You cannot delete the default `My Schedule` one.
    ///
    /// # Parameter
    /// - `schedule_name`: The name of the schedule to delete.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Delete the schedule "`Test Schedule`."
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // You should do error handling here, but I won't
    /// assert!(wrapper.get_schedule_list().await.unwrap().contains(&"Test Schedule".to_string()));
    /// wrapper.remove_schedule("Test Schedule").await;
    /// assert!(!wrapper.get_schedule_list().await.unwrap().contains(&"Test Schedule".to_string()));
    /// # }
    /// ```
    pub async fn remove_schedule(&self, schedule_name: impl AsRef<str>) -> self::Result<bool> {
        // Can't remove your default schedule.
        if schedule_name.as_ref() == DEFAULT_SCHEDULE_NAME {
            return Err(WrapperError::InputError(
                "schedule_name",
                "You cannot remove the default schedule.",
            ));
        }

        self._process_post_response(
            self.client
                .post(REMOVE_SCHEDULE)
                .form(&[
                    ("termcode", self.term.as_str()),
                    ("schedname", schedule_name.as_ref()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Adds an event to your WebReg calendar, or edits an existing event.
    ///
    /// # Parameter
    /// - `event_info`: The details of the event.
    /// - `event_timestamp`: The timestamp corresponding to the event that you want to
    /// edit. If this is `None`, then this function will add the event. If this is `Some`,
    /// then this function will edit an existing event.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Renaming the schedule "`Test Schedule`" to "`Another Schedule`."
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::{DayOfWeek, WebRegWrapper};
    /// use webweg::wrapper::EventAdd;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// let event = EventAdd {
    ///     event_name: "Clown on AYU",
    ///     location: Some("B250"),
    ///     event_days: vec![DayOfWeek::Monday, DayOfWeek::Friday],
    ///     start_hr: 5,
    ///     start_min: 30,
    ///     end_hr: 10,
    ///     end_min: 45,
    /// };
    ///
    /// // Adding an event
    /// wrapper.add_or_edit_event(event, None).await;
    ///
    /// // Editing an event (commenting this out since we moved `event` in the previous line)
    /// // wrapper.add_or_edit_event(event, Some("2022-09-09 21:50:16.846885")).await;
    /// # }
    /// ```
    pub async fn add_or_edit_event(
        &self,
        event_info: EventAdd<'_>,
        event_timestamp: impl Into<Option<&str>>,
    ) -> self::Result<bool> {
        let start_time_full = event_info.start_hr * 100 + event_info.start_min;
        let end_time_full = event_info.end_hr * 100 + event_info.end_min;
        if start_time_full >= end_time_full {
            return Err(WrapperError::InputError(
                "time",
                "Start time must be less than end time.",
            ));
        }

        if event_info.start_hr < 7 || event_info.start_hr > 12 + 10 {
            return Err(WrapperError::InputError(
                "event_info.start_hr",
                "Start hour must be between 7 and 22 (7am and 10pm)",
            ));
        }

        if event_info.start_hr == 12 + 10 && event_info.start_min != 0 {
            return Err(WrapperError::InputError(
                "event_info.start",
                "You cannot exceed 10pm.",
            ));
        }

        if event_info.event_days.is_empty() {
            return Err(WrapperError::InputError(
                "event_info.event_days",
                "Must specify one day.",
            ));
        }

        let mut days: [bool; 7] = [false; 7];
        for d in event_info.event_days {
            let idx = match d {
                DayOfWeek::Monday => 0,
                DayOfWeek::Tuesday => 1,
                DayOfWeek::Wednesday => 2,
                DayOfWeek::Thursday => 3,
                DayOfWeek::Friday => 4,
                DayOfWeek::Saturday => 5,
                DayOfWeek::Sunday => 6,
            };

            days[idx] = true;
        }

        let mut day_str = String::new();
        for d in days {
            day_str.push(if d { '1' } else { '0' });
        }

        assert_eq!(7, day_str.len());

        let mut start_time_full = start_time_full.to_string();
        let mut end_time_full = end_time_full.to_string();
        while start_time_full.len() < 4 {
            start_time_full.insert(0, '0');
        }

        while end_time_full.len() < 4 {
            end_time_full.insert(0, '0');
        }

        let mut form_data = HashMap::from([
            ("termcode", self.term.as_str()),
            ("aename", event_info.event_name),
            ("aestarttime", start_time_full.as_str()),
            ("aeendtime", end_time_full.as_str()),
            ("aelocation", event_info.location.unwrap_or("")),
            ("aedays", day_str.as_str()),
        ]);

        let et = event_timestamp.into();
        if let Some(timestamp) = et {
            form_data.insert("aetimestamp", timestamp);
        }

        self._process_post_response(
            self.client
                .post(match et {
                    Some(_) => EVENT_EDIT,
                    None => EVENT_ADD,
                })
                .form(&form_data)
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Removes an event from your WebReg calendar.
    ///
    /// # Parameter
    /// - `event_timestamp`: The timestamp corresponding to the event that you want to
    /// remove.
    ///
    /// # Returns
    /// `true` if the process succeeded, or a string containing the error message from WebReg if
    /// something wrong happened.
    ///
    /// # Example
    /// Renaming the schedule "`Test Schedule`" to "`Another Schedule`."
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // Removing an event
    /// wrapper.remove_event("2022-09-09 21:50:16.846885").await;
    /// # }
    /// ```
    pub async fn remove_event(&self, event_timestamp: impl AsRef<str>) -> self::Result<bool> {
        self._process_post_response(
            self.client
                .post(EVENT_REMOVE)
                .form(&[
                    ("aetimestamp", event_timestamp.as_ref()),
                    ("termcode", self.term.as_str()),
                ])
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Gets all event from your WebReg calendar.
    ///
    /// # Returns
    /// A vector of all events, or `None` if an error occurred.
    ///
    /// # Example
    /// Renaming the schedule "`Test Schedule`" to "`Another Schedule`."
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// // Get all my events
    /// let all_events = wrapper.get_events().await;
    /// # }
    /// ```
    pub async fn get_events(&self) -> self::Result<Vec<Event>> {
        let url = Url::parse_with_params(EVENT_GET, &[("termcode", self.term.as_str())]).unwrap();
        let raw_events = self
            ._process_get_result::<Vec<RawEvent>>(
                self.client
                    .get(url)
                    .header(COOKIE, &self.cookies)
                    .header(USER_AGENT, MY_USER_AGENT)
                    .send()
                    .await,
            )
            .await?;

        let mut res = vec![];
        for event in raw_events {
            let start_chars = event.start_time.chars().collect::<Vec<_>>();
            let start_hr = start_chars[0].to_digit(10).unwrap_or_default()
                * start_chars[1].to_digit(10).unwrap_or_default();
            let start_min = start_chars[2].to_digit(10).unwrap_or_default()
                * start_chars[3].to_digit(10).unwrap_or_default();
            let end_chars = event.end_time.chars().collect::<Vec<_>>();
            let end_hr = end_chars[0].to_digit(10).unwrap_or_default()
                * end_chars[1].to_digit(10).unwrap_or_default();
            let end_min = end_chars[2].to_digit(10).unwrap_or_default()
                * end_chars[3].to_digit(10).unwrap_or_default();

            res.push(Event {
                location: event.location,
                start_hr: start_hr as i16,
                start_min: start_min as i16,
                end_hr: end_hr as i16,
                end_min: end_min as i16,
                name: event.description,
                days: parse_binary_days(&event.days),
                timestamp: event.time_stamp,
            });
        }

        Ok(res)
    }

    /// Gets all of your schedules.
    ///
    /// # Returns
    /// Either a vector of strings representing the names of the schedules, or the error that
    /// occurred.
    pub async fn get_schedule_list(&self) -> self::Result<Vec<String>> {
        let url = Url::parse_with_params(ALL_SCHEDULE, &[("termcode", self.term.as_str())])?;

        self._process_get_result::<Vec<String>>(
            self.client
                .get(url)
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
    }

    /// Processes a GET response from the resulting JSON, if any.
    ///
    /// # Parameters
    /// - `res`: The initial response.
    ///
    /// # Returns
    /// The result of processing the response.
    async fn _process_get_result<T: DeserializeOwned>(
        &self,
        res: Result<Response, Error>,
    ) -> self::Result<T> {
        let r = res?;
        if !r.status().is_success() {
            return Err(WrapperError::BadStatusCode(r.status().as_u16()));
        }

        let text = r.text().await?;
        match serde_json::from_str::<T>(&text) {
            Ok(o) => Ok(o),
            Err(e) => Err(WrapperError::SerdeError(e)),
        }
    }

    /// Processes a POST response from the resulting JSON, if any.
    ///
    /// # Parameters
    /// - `res`: The initial response.
    ///
    /// # Returns
    /// Either one of:
    /// - `true` or `false`, depending on what WebReg returns.
    /// - or some error message if an error occurred.
    async fn _process_post_response(&self, res: Result<Response, Error>) -> self::Result<bool> {
        let r = res?;
        if !r.status().is_success() {
            return Err(WrapperError::BadStatusCode(r.status().as_u16()));
        }

        let text = r.text().await?;
        let json: Value = serde_json::from_str(&text).unwrap();
        if json["OPS"].is_string() && json["OPS"].as_str().unwrap() == "SUCCESS" {
            return Ok(true);
        }

        // Purely to handle an error
        let mut parsed_str = String::new();
        let mut is_in_brace = false;
        json["REASON"]
            .as_str()
            .unwrap_or("")
            .trim()
            .chars()
            .for_each(|c| {
                if c == '<' {
                    is_in_brace = true;
                    return;
                }

                if c == '>' {
                    is_in_brace = false;
                    return;
                }

                if is_in_brace {
                    return;
                }

                parsed_str.push(c);
            });

        Err(WrapperError::WebRegError(parsed_str))
    }

    /// Gets the current term.
    ///
    /// # Returns
    /// The current term.
    pub fn get_term(&self) -> &str {
        self.term.as_str()
    }

    /// Checks if the output string represents a valid session.
    ///
    /// # Parameters
    /// - `s`: The string.
    ///
    /// # Returns
    /// `true` if the string doesn't contain signs that we have an invalid session.
    #[inline(always)]
    fn internal_is_valid(&self, s: &str) -> bool {
        !s.contains("Skip to main content")
    }

    /// Gets the current epoch time.
    ///
    /// # Returns
    /// The current time.
    fn get_epoch_time(&self) -> u128 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    /// Gets the formatted course code so that it can be recognized by WebReg's internal API.
    ///
    /// # Parameters
    /// - `course_code`: The course code, e.g. if you have the course `CSE 110`, you would put
    /// `110`.
    ///
    /// # Returns
    /// The formatted course code for WebReg.
    #[inline(always)]
    fn get_formatted_course_code(&self, course_code: &str) -> String {
        // If the course code only has 1 digit (excluding any letters), then we need to prepend 2
        // spaces to the course code.
        //
        // If the course code has 2 digits (excluding any letters), then we need to prepend 1
        // space to the course code.
        //
        // Otherwise, don't need to prepend any spaces to the course code.
        //
        // For now, assume that no digits will ever appear *after* the letters. Weird thing is that
        // WebReg uses '+' to offset the course code but spaces are accepted.
        match course_code.chars().filter(|x| x.is_ascii_digit()).count() {
            1 => format!("  {}", course_code),
            2 => format!(" {}", course_code),
            _ => course_code.to_string(),
        }
    }

    /// Gets the instructor's names.
    ///
    /// # Parameters
    /// - `instructor_name`: The raw name.
    ///
    /// # Returns
    /// The parsed instructor's names, as a vector.
    fn get_instructor_names(&self, instructor_name: &str) -> Vec<String> {
        // The instructor string is in the form
        // name1    ;pid1:name2      ;pid2:...:nameN      ;pidN
        instructor_name
            .split(':')
            .map(|x| {
                if x.contains(';') {
                    x.split_once(';').unwrap().0.trim().to_string()
                } else {
                    x.trim().to_string()
                }
            })
            .collect()
    }

    /// Removes duplicate names from the list of instructors that are given.
    ///
    /// # Parameters
    /// - `instructors`: An iterator of instructors, potentially with duplicates.
    ///
    /// # Returns
    /// A vector of instructors, with no duplicates.
    fn get_all_instructors<I>(&self, instructors: I) -> Vec<String>
    where
        I: Iterator<Item = String>,
    {
        let mut all_inst = instructors.collect::<Vec<_>>();
        all_inst.sort();
        all_inst.dedup();
        all_inst
    }
}

// Helper structure for organizing meetings. Only used once for now.
#[derive(Debug)]
struct GroupedSection<'a, T> {
    /// All general meetings. These include meetings that are consistent across *all* sections.
    /// For example, lectures and final exams.
    general_meetings: Vec<&'a T>,

    /// All unique meetings. These are generally meetings that are unique the one section.
    /// For example, discussions.
    child_meetings: Vec<&'a T>,
}

/// Use this struct to add more information regarding the section that you want to enroll/waitlist
/// in.
pub struct EnrollWaitAdd<'a> {
    /// The section ID. For example, `0123123`.
    pub section_id: &'a str,
    /// The grading option. Can either be L, P, or S.
    /// If None is specified, this uses the default option.
    pub grading_option: Option<GradeOption>,
    /// The number of units. If none is specified, this
    /// uses the default unit count.
    pub unit_count: Option<u8>,
}

impl<'a> EnrollWaitAdd<'a> {
    /// Creates a new `EnrollWaitAdd` structure with the specified `section_id` and default grading
    /// option and unit count.
    ///
    /// # Parameters
    /// - `section_id`: The section ID.
    ///
    /// # Returns
    /// The structure.
    pub fn new(section_id: &'a str) -> Self {
        Self {
            section_id,
            grading_option: None,
            unit_count: None,
        }
    }
}

// This trait may be helpful later.
impl<'a> AsRef<EnrollWaitAdd<'a>> for EnrollWaitAdd<'a> {
    fn as_ref(&self) -> &EnrollWaitAdd<'a> {
        self
    }
}

/// Use this struct to add more information regarding the course that you want to plan.
pub struct PlanAdd<'a> {
    /// The subject code. For example, `CSE`.
    pub subject_code: &'a str,
    /// The course code. For example, `12`.
    pub course_code: &'a str,
    /// The section ID. For example, `0123123`.
    pub section_id: &'a str,
    /// The section code. For example `A00`.
    pub section_code: &'a str,
    /// The grading option.
    pub grading_option: Option<GradeOption>,
    /// The schedule name.
    pub schedule_name: Option<&'a str>,
    /// The number of units.
    pub unit_count: u8,
}

/// A struct that represents an event to be added.
pub struct EventAdd<'a> {
    /// The name of the event. This is required.
    pub event_name: &'a str,
    /// The location of the event. This is optional.
    pub location: Option<&'a str>,
    /// The days that this event will be held.
    pub event_days: Vec<DayOfWeek>,
    /// The hour start time. For example, if the event starts at
    /// 3:50 PM, use `15` (since `12 + 3 = 15`).
    pub start_hr: i16,
    /// The minute start time. For example, if the event starts at
    /// 3:50 PM, use `50`.
    pub start_min: i16,
    /// The hour end time. For example, if the event ends at 3:50 PM,
    /// use `15` (since `12 + 3 = 15`).
    pub end_hr: i16,
    /// The minute end time. For example, if the event ends at 3:50 PM,
    /// use `50`.
    pub end_min: i16,
}

/// Used to construct search requests for the `search_courses` function.
#[derive(Clone)]
pub struct SearchRequestBuilder<'a> {
    subjects: Vec<&'a str>,
    courses: Vec<&'a str>,
    departments: Vec<&'a str>,
    instructor: Option<&'a str>,
    title: Option<&'a str>,
    level_filter: u32,
    days: u32,
    start_time: Option<(u32, u32)>,
    end_time: Option<(u32, u32)>,
    only_open: bool,
}

impl<'a> SearchRequestBuilder<'a> {
    /// Creates a new instance of the `SearchRequestBuilder`, which is used to search for specific
    /// courses.
    ///
    /// # Returns
    /// The empty `SearchRequestBuilder`.
    pub fn new() -> Self {
        Self {
            subjects: vec![],
            courses: vec![],
            departments: vec![],
            instructor: None,
            title: None,
            level_filter: 0,
            days: 0,
            start_time: None,
            end_time: None,
            only_open: false,
        }
    }

    /// Adds a subject to this search request. Valid search requests are uppercase and at most
    /// 4 characters long. Some examples include `MATH` or `CSE`.
    ///
    /// # Parameters
    /// - `subject`: The subject.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn add_subject(mut self, subject: &'a str) -> Self {
        if subject != subject.to_uppercase() || subject.len() > 4 {
            return self;
        }

        self.subjects.push(subject);
        self
    }

    /// Adds a course (either a subject code, course code, or both) to the search request. Some
    /// examples include `20E`, `math 20d`, `101`, `CSE`.
    ///
    /// # Parameters
    /// - `course`: The course.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn add_course(mut self, course: &'a str) -> Self {
        self.courses.push(course);
        self
    }

    /// Adds a department to the search request. Valid search requests are uppercase and at most 4
    /// characters long. Some examples include `MATH` or `CSE`.
    ///
    /// # Parameters
    /// - `department`: The department.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn add_department(mut self, department: &'a str) -> Self {
        if department != department.to_uppercase() || department.len() > 4 {
            return self;
        }

        self.departments.push(department);
        self
    }

    /// Sets the instructor to the specified instructor.
    ///
    /// # Parameters
    /// - `instructor`: The instructor. This should be formatted in `Last Name, First Name` form.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_instructor(mut self, instructor: &'a str) -> Self {
        self.instructor = Some(instructor);
        self
    }

    /// Sets the course title to the specified title. Some examples could be `differential equ`,
    /// `data structures`, `algorithms`, and so on.
    ///
    /// # Parameters
    /// - `title`: The title of the course.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Restrict search results to to the specified filter. This can be applied multiple times.
    ///
    /// # Parameters
    /// - `filter`: The filter.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn filter_courses_by(mut self, filter: CourseLevelFilter) -> Self {
        self.level_filter |= match filter {
            CourseLevelFilter::LowerDivision => 1 << 11,
            CourseLevelFilter::FreshmenSeminar => 1 << 10,
            CourseLevelFilter::LowerDivisionIndepStudy => 1 << 9,
            CourseLevelFilter::UpperDivision => 1 << 8,
            CourseLevelFilter::Apprenticeship => 1 << 7,
            CourseLevelFilter::UpperDivisionIndepStudy => 1 << 6,
            CourseLevelFilter::Graduate => 1 << 5,
            CourseLevelFilter::GraduateIndepStudy => 1 << 4,
            CourseLevelFilter::GraduateResearch => 1 << 3,
            CourseLevelFilter::Lvl300 => 1 << 2,
            CourseLevelFilter::Lvl400 => 1 << 1,
            CourseLevelFilter::Lvl500 => 1 << 0,
        };

        self
    }

    /// Only shows courses based on the specified day(s).
    ///
    /// # Parameters
    /// - `day`: The day.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn apply_day(mut self, day: DayOfWeek) -> Self {
        let day = match day {
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
            DayOfWeek::Sunday => 7,
        };

        self.days |= 1 << (7 - day);
        self
    }

    /// Sets the start time to the specified time.
    ///
    /// # Parameters
    /// - `hour`: The hour. This should be between 0 and 23, inclusive.
    /// - `min`: The minute. This should be between 0 and 59, inclusive.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_start_time(mut self, hour: u32, min: u32) -> Self {
        if hour > 23 || min > 59 {
            return self;
        }

        self.start_time = Some((hour, min));
        self
    }

    /// Sets the end time to the specified time.
    ///
    /// # Parameters
    /// - `hour`: The hour. This should be between 0 and 23, inclusive.
    /// - `min`: The minute. This should be between 0 and 59, inclusive.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_end_time(mut self, hour: u32, min: u32) -> Self {
        if hour > 23 || min > 59 {
            return self;
        }

        self.end_time = Some((hour, min));
        self
    }

    /// Whether to only show sections with open seats.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn only_allow_open(mut self) -> Self {
        self.only_open = true;
        self
    }
}

impl<'a> Default for SearchRequestBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// The day of week enum, which designates what days you want
/// to filter specific sections by.
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// The course level filter enum, which can be used to filter
/// specific sections by.
pub enum CourseLevelFilter {
    /// Level 1-99 courses.
    LowerDivision,
    /// Level 87, 90 courses.
    FreshmenSeminar,
    /// Level 99 courses.
    LowerDivisionIndepStudy,
    /// Level 100-198 courses
    UpperDivision,
    /// Level 195 courses
    Apprenticeship,
    /// Level 199 courses
    UpperDivisionIndepStudy,
    /// Level 200-297 courses
    Graduate,
    /// Level 298 courses
    GraduateIndepStudy,
    /// Level 299 courses
    GraduateResearch,
    /// Level 300+ courses
    Lvl300,
    /// Level 400+ courses
    Lvl400,
    /// Level 500+ courses
    Lvl500,
}

/// Lets you choose how you want to search for a course.
pub enum SearchType<'a> {
    /// Searches for a course by section ID.
    BySection(&'a str),

    /// Searches for a course by more than one section ID.
    ByMultipleSections(&'a [&'a str]),

    /// Searches for a (set of) course(s) by multiple specifications.
    Advanced(&'a SearchRequestBuilder<'a>),
}

/// The possible grading options.
pub enum GradeOption {
    /// S/U grading (Satisfactory/Unsatisfactory) option.
    S,

    /// P/NP grading (Pass/No Pass) option.
    P,

    /// Letter grading option.
    L,
}
