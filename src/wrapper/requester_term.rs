use std::collections::HashMap;

use url::Url;

use crate::constants::{
    ALL_SCHEDULE, CHANGE_ENROLL, COURSE_DATA, CURR_SCHEDULE, DEFAULT_SCHEDULE_NAME, DEPT_LIST,
    ENROLL_ADD, ENROLL_DROP, ENROLL_EDIT, EVENT_ADD, EVENT_EDIT, EVENT_GET, EVENT_REMOVE, PLAN_ADD,
    PLAN_EDIT, PLAN_REMOVE, PLAN_REMOVE_ALL, PREREQS_INFO, REMOVE_SCHEDULE, RENAME_SCHEDULE,
    SEND_EMAIL, SUBJ_LIST, WAITLIST_ADD, WAITLIST_DROP, WAITLIST_EDIT,
};
use crate::raw_types::{
    RawDepartmentElement, RawEvent, RawPrerequisite, RawScheduledMeeting, RawSubjectElement,
    RawWebRegMeeting, RawWebRegSearchResultItem,
};
use crate::types::{
    Courses, Events, PrerequisiteInfo, Schedule, SearchResult, SearchResultItem,
    SectionIdNotFoundContext, WrapperError,
};
use crate::wrapper::input_types::{
    AddType, DayOfWeek, EnrollWaitAdd, EventAdd, ExplicitAddType, GradeOption, PlanAdd, SearchType,
};
use crate::wrapper::request_builder::WrapperTermTempRequest;
use crate::wrapper::ww_helper::{
    associate_term_helper, extract_text, process_get_text, process_post_response,
};
use crate::wrapper::ReqwestClientWrapper;
use crate::ww_parser::{
    build_search_course_url, parse_course_info, parse_enrollment_count, parse_get_events,
    parse_prerequisites, parse_schedule,
};
use crate::{types, util};

/// A structure that can be used to get raw data from WebReg, with minimal error handling.
///
/// Keep in mind that this structure only gives you some API access, as these APIs give you
/// interesting data. For full access, consider using `WrapperTermRequest`.
pub struct WrapperTermRawRequest<'a> {
    pub(crate) info: WrapperTermTempRequest<'a>,
}

impl<'a> WrapperTermRawRequest<'a> {
    /// Gets all prerequisites for a specified course for the term set by the wrapper.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_code`: The course code. For example, if you wanted to check `MATH 100B`, you
    /// would put `100B`.
    ///
    /// # Returns
    /// Prerequisite data as returned by WebReg.
    pub async fn get_prerequisites(
        &self,
        subject_code: impl AsRef<str>,
        course_code: impl AsRef<str>,
    ) -> types::Result<String> {
        let crsc_code = util::get_formatted_course_num(course_code.as_ref());
        let url = Url::parse_with_params(
            PREREQS_INFO,
            &[
                ("subjcode", subject_code.as_ref()),
                ("crsecode", crsc_code.as_str()),
                ("termcode", self.info.term),
                ("_", util::get_epoch_time().to_string().as_ref()),
            ],
        )?;

        extract_text(self.info.req_get(url).send().await).await
    }

    /// Gets your current schedule.
    ///
    /// # Parameters
    /// - `schedule_name`: The schedule that you want to get. If `None` is given, this will default
    /// to your main schedule.
    ///
    /// # Returns
    /// Schedule data as returned by WebReg.
    pub async fn get_schedule(&self, schedule_name: Option<&str>) -> types::Result<String> {
        let url = Url::parse_with_params(
            CURR_SCHEDULE,
            &[
                ("schedname", schedule_name.unwrap_or(DEFAULT_SCHEDULE_NAME)),
                ("final", ""),
                ("sectnum", ""),
                ("termcode", self.info.term),
                ("_", util::get_epoch_time().to_string().as_str()),
            ],
        )?;

        extract_text(self.info.req_get(url).send().await).await
    }

    /// Gets course information for a particular course.
    ///
    /// Note that WebReg provides this information in a way that makes it hard to use; in
    /// particular, WebReg separates each lecture, discussion, final exam, etc. from each other.
    /// This function attempts to figure out which lecture/discussion/final exam/etc. correspond
    /// to which section.
    ///
    /// Additionally, this implementation will not retrieve canceled sections.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_num`: The course number. For example, if you wanted to check `MATH 100B`, you
    /// would put `100B`.
    ///
    /// # Returns
    /// Course information, as returned by WebReg.
    pub async fn get_course_info(
        &self,
        subject_code: impl AsRef<str>,
        course_num: impl AsRef<str>,
    ) -> types::Result<String> {
        let crsc_code = util::get_formatted_course_num(course_num.as_ref());
        let url = Url::parse_with_params(
            COURSE_DATA,
            &[
                ("subjcode", subject_code.as_ref()),
                ("crsecode", crsc_code.as_str()),
                ("termcode", self.info.term),
                ("_", util::get_epoch_time().to_string().as_ref()),
            ],
        )?;

        extract_text(self.info.req_get(url).send().await).await
    }

    /// Gets a list of all departments that are offering courses for the given term.
    ///
    /// # Returns
    /// Department codes, as returned by WebReg.
    pub async fn get_department_codes(&self) -> types::Result<String> {
        extract_text(
            self.info
                .req_get(Url::parse_with_params(
                    DEPT_LIST,
                    &[
                        ("termcode", self.info.term),
                        ("_", util::get_epoch_time().to_string().as_str()),
                    ],
                )?)
                .send()
                .await,
        )
        .await
    }

    /// Gets a list of all subjects that have at least one course offered for the given term.
    ///
    /// # Returns
    /// Subject codes, as returned by WebReg.
    pub async fn get_subject_codes(&self) -> types::Result<String> {
        extract_text(
            self.info
                .req_get(Url::parse_with_params(
                    SUBJ_LIST,
                    &[
                        ("termcode", self.info.term),
                        ("_", util::get_epoch_time().to_string().as_str()),
                    ],
                )?)
                .send()
                .await,
        )
        .await
    }

    /// Gets all courses that are available. All this does is searches for all courses via Webreg's
    /// menu. Thus, only basic details are shown.
    ///
    /// # Parameters
    /// - `filter_by`: The request filter.
    ///
    /// # Returns
    /// Search results, as returned by WebReg.
    pub async fn search_courses(&self, filter_by: SearchType) -> types::Result<String> {
        extract_text(
            self.info
                .req_get(build_search_course_url(filter_by, self.info.term)?)
                .send()
                .await,
        )
        .await
    }

    /// Gets all event from your WebReg calendar.
    ///
    /// # Returns
    /// Information about any events you added, as returned by WebReg.
    pub async fn get_events(&self) -> types::Result<String> {
        let url = Url::parse_with_params(EVENT_GET, &[("termcode", self.info.term)]).unwrap();
        extract_text(self.info.req_get(url).send().await).await
    }

    /// Gets all of your schedules.
    ///
    /// # Returns
    /// Your schedule list, as returned by WebReg.
    pub async fn get_schedule_list(&self) -> types::Result<String> {
        let url = Url::parse_with_params(ALL_SCHEDULE, &[("termcode", self.info.term)])?;
        extract_text(self.info.req_get(url).send().await).await
    }

    /// Associates the term bound by this request to the cookies that are provided
    /// as part of this overridden request.
    ///
    /// # Returns
    /// A result, where nothing is returned if everything went well and an error is returned
    /// if something went wrong.
    pub async fn associate_term(&self) -> types::Result<()> {
        associate_term_helper(&self.info, self.info.term).await
    }
}

/// A structure that can be used to get data from WebReg and additionally parse it into
/// a Rust structure so that you can use the data easily without any additional parsing.
///
/// # Difference Between This Structure & `WrapperTermRawRequest`
/// Prefer this structure if you just want to be able to use the data from WebReg, without
/// any additional parsing. This structure handles a majority of the parsing for you.
///
/// Prefer `WrapperTermRawRequest` if you want more control over how WebReg data should be
/// parsed into a structure.
///
/// Keep in mind that this structure gives you the most access to various WebReg APIs that
/// involve `GET`ting data or `POST`ing data. The raw variant only gives you limited access
/// to some GET requests (the ones that return interesting data) and none of the POST requests
/// (since they don't give you anything useful).
///
/// In fact, `WrapperTermRequest` makes direct use of `WrapperTermRawRequest`.
pub struct WrapperTermRequest<'a> {
    pub(crate) raw: WrapperTermRawRequest<'a>,
}

impl<'a> WrapperTermRequest<'a> {
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
    /// use webweg::wrapper::WebRegWrapper;
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let prereqs = wrapper
    ///     .default_request()
    ///     .get_prerequisites("COGS", "108")
    ///     .await;
    ///
    /// if let Ok(prereq_info) = prereqs {
    ///     println!("{:?}", prereq_info.course_prerequisites);
    ///     println!("{:?}", prereq_info.exam_prerequisites);
    /// }
    /// # }
    /// ```
    pub async fn get_prerequisites(
        &self,
        subject_code: impl AsRef<str>,
        course_code: impl AsRef<str>,
    ) -> types::Result<PrerequisiteInfo> {
        parse_prerequisites(process_get_text::<Vec<RawPrerequisite>>(
            self.raw
                .get_prerequisites(subject_code, course_code)
                .await?,
        )?)
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
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// // Pass in "None" for the default schedule
    /// let default_schedule = wrapper.default_request().get_schedule(None).await;
    ///
    /// match default_schedule {
    ///     Ok(o) => o.iter().for_each(|sec| println!("{sec}")),
    ///     Err(e) => eprintln!("An error occurred! {e}"),
    /// }
    /// # }
    /// ```
    ///
    /// Getting the schedule with name "`Other Schedule`."
    /// ```rust,no_run
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// // Get all courses under the "Other Schedule" schedule
    /// let other_schedule = wrapper.default_request().get_schedule(Some("Other Schedule")).await;
    ///
    /// match other_schedule {
    ///     Ok(o) => o.iter().for_each(|sec| println!("{sec}")),
    ///     Err(e) => eprintln!("An error occurred! {e}"),
    /// }
    /// # }
    /// ```
    pub async fn get_schedule(&self, schedule_name: Option<&str>) -> types::Result<Schedule> {
        parse_schedule(process_get_text::<Vec<RawScheduledMeeting>>(
            self.raw.get_schedule(schedule_name).await?,
        )?)
    }

    /// Gets enrollment count for a particular course.
    ///
    /// Unlike the `get_course_info` function, this function only returns a vector of sections
    /// with the proper enrollment counts. Therefore, the `meetings` vector will always be
    /// empty as it is not relevant.
    ///
    /// Additionally, this function only returns one of some number of possible instructors.
    ///
    /// Also keep in mind that this implementation will not retrieve canceled sections.
    ///
    /// If you want full course information, use `get_course_info`. If you only care about the
    /// number of people enrolled in a section, this function is for you.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_num`: The course number. For example, if you wanted to check `MATH 100B`, you
    /// would put `100B`.
    ///
    /// # Returns
    /// Either a vector with all sections that match the given subject code & course code, or an
    /// error if one occurred.
    ///
    /// # Example
    /// Suppose we wanted to find all sections of COGS 108 for the sole purpose of seeing how
    /// many people are enrolled.
    /// ```rust,no_run
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let sec_count = wrapper
    ///     .default_request()
    ///     .get_enrollment_count("COGS", "108")
    ///     .await;
    ///
    /// match sec_count {
    ///     Ok(o) => o.iter().for_each(|sec| println!("{sec}")),
    ///     Err(e) => eprintln!("An error occurred! {e}"),
    /// }
    /// # }
    /// ```
    pub async fn get_enrollment_count(
        &self,
        subject_code: impl AsRef<str>,
        course_num: impl AsRef<str>,
    ) -> types::Result<Courses> {
        let course_dept_id = format!(
            "{} {}",
            subject_code.as_ref().trim(),
            course_num.as_ref().trim()
        )
        .to_uppercase();

        parse_enrollment_count(
            process_get_text::<Vec<RawWebRegMeeting>>(
                self.raw.get_course_info(subject_code, course_num).await?,
            )?,
            course_dept_id,
        )
    }

    /// Gets course information for a particular course.
    ///
    /// Note that WebReg provides this information in a way that makes it hard to use; in
    /// particular, WebReg separates each lecture, discussion, final exam, etc. from each other.
    /// This function attempts to figure out which lecture/discussion/final exam/etc. correspond
    /// to which section.
    ///
    /// Additonally, this implementation will not retrieve canceled sections.
    ///
    /// # Parameters
    /// - `subject_code`: The subject code. For example, if you wanted to check `MATH 100B`, you
    /// would put `MATH`.
    /// - `course_num`: The course number. For example, if you wanted to check `MATH 100B`, you
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
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let course_info = wrapper
    ///     .default_request()
    ///     .get_course_info("CSE", "105")
    ///     .await;
    ///
    /// match course_info {
    ///     Ok(o) => o.iter().for_each(|sec| println!("{sec}")),
    ///     Err(e) => eprintln!("An error occurred! {e}"),
    /// }
    /// # }
    /// ```
    pub async fn get_course_info(
        &self,
        subject_code: impl AsRef<str>,
        course_num: impl AsRef<str>,
    ) -> types::Result<Courses> {
        let course_dept_id = format!(
            "{} {}",
            subject_code.as_ref().trim(),
            course_num.as_ref().trim()
        )
        .to_uppercase();

        parse_course_info(
            process_get_text::<Vec<RawWebRegMeeting>>(
                self.raw.get_course_info(subject_code, course_num).await?,
            )?,
            course_dept_id,
        )
    }

    /// Gets a list of all departments that are offering courses for the given term.
    ///
    /// # Returns
    /// A vector of department codes.
    pub async fn get_department_codes(&self) -> types::Result<Vec<String>> {
        Ok(
            process_get_text::<Vec<RawDepartmentElement>>(self.raw.get_department_codes().await?)?
                .into_iter()
                .map(|x| x.dep_code.trim().to_string())
                .collect::<Vec<_>>(),
        )
    }

    /// Gets a list of all subjects that have at least one course offered for the given term.
    ///
    /// # Returns
    /// A vector of subject codes.
    pub async fn get_subject_codes(&self) -> types::Result<Vec<String>> {
        Ok(
            process_get_text::<Vec<RawSubjectElement>>(self.raw.get_subject_codes().await?)?
                .into_iter()
                .map(|x| x.subject_code.trim().to_string())
                .collect::<Vec<_>>(),
        )
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
    pub async fn search_courses(&self, filter_by: SearchType) -> types::Result<SearchResult> {
        Ok(process_get_text::<Vec<RawWebRegSearchResultItem>>(
            self.raw.search_courses(filter_by).await?,
        )?
        .into_iter()
        .map(|item| SearchResultItem {
            subj_code: item.subj_code.trim().to_owned(),
            course_code: item.course_code.trim().to_owned(),
            course_title: item.course_title.trim().to_owned(),
        })
        .collect())
    }

    /// Gets all event from your WebReg calendar.
    ///
    /// # Returns
    /// A vector of all events, or `None` if an error occurred.
    ///
    /// # Example
    /// Renaming the schedule "`Test Schedule`" to "`Another Schedule`."
    /// ```rust,no_run
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let events = wrapper
    ///     .default_request()
    ///     .get_events()
    ///     .await;
    /// match events {
    ///     Ok(events) => events.into_iter().for_each(|event| println!("{event}")),
    ///     Err(e) => eprintln!("Error! {e}"),
    /// };
    /// # }
    /// ```
    pub async fn get_events(&self) -> types::Result<Events> {
        parse_get_events(process_get_text::<Vec<RawEvent>>(
            self.raw.get_events().await?,
        )?)
    }

    /// Gets all of your schedules.
    ///
    /// # Returns
    /// Either a vector of strings representing the names of the schedules, or the error that
    /// occurred.
    pub async fn get_schedule_list(&self) -> types::Result<Vec<String>> {
        process_get_text::<Vec<String>>(self.raw.get_schedule_list().await?)
    }

    /// Sends an email to yourself using the same email that is used to confirm that you have
    /// enrolled or waitlisted in a particular class. In other words, this will send an email
    /// to you through the email `NoReplyRegistrar@ucsd.edu`.
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
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let email_res = wrapper
    ///     .default_request()
    ///     .send_email_to_self("hello, world.")
    ///     .await;
    ///
    /// match email_res {
    ///     Ok(_) => println!("Email sent successfully."),
    ///     Err(e) => eprintln!("Email could not be sent: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn send_email_to_self(&self, email_content: &str) -> types::Result<()> {
        let r = self
            .raw
            .info
            .req_post(SEND_EMAIL)
            .form(&[
                ("actionevent", email_content),
                ("termcode", self.raw.info.term),
            ])
            .send()
            .await?;

        let status = r.status();
        if !status.is_success() {
            let text = r.text().await.ok();
            return Err(WrapperError::BadStatusCode(status.as_u16(), text));
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
    /// Changing the section associated with section ID `235181` to letter grading option.
    /// ```rust,no_run
    /// use webweg::wrapper::input_types::GradeOption;
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let change_res = wrapper
    ///     .default_request()
    ///     .change_grading_option("235181", GradeOption::P)
    ///     .await;
    ///
    /// match change_res {
    ///     Ok(res) => println!("Grade option changed? {res}"),
    ///     Err(e) => eprintln!("Grade option error: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn change_grading_option(
        &self,
        section_id: &str,
        new_grade_opt: GradeOption,
    ) -> types::Result<bool> {
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
            None => {
                return Err(WrapperError::SectionIdNotFound(
                    section_id.into(),
                    SectionIdNotFoundContext::Schedule,
                ))
            }
        };

        let sec_id = poss_class.section_id.to_string();
        let units = poss_class.units.to_string();

        process_post_response(
            self.raw
                .info
                .req_post(CHANGE_ENROLL)
                .form(&[
                    ("section", sec_id.as_str()),
                    ("subjCode", ""),
                    ("crseCode", ""),
                    ("unit", units.as_str()),
                    ("grade", new_grade_opt),
                    // You don't actually need these
                    ("oldGrade", ""),
                    ("oldUnit", ""),
                    ("termcode", self.raw.info.term),
                ])
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
    /// Either an `Ok` if the request generally went through, or an `Err` if something went wrong
    /// with the request.
    ///
    /// # Example
    /// Here, we will add the course `CSE 100`, which has section ID `079911` and section code
    /// `A01`, to our plan.
    /// ```rust,no_run
    /// use webweg::wrapper::input_types::{GradeOption, PlanAdd};
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let plan_add_data = PlanAdd::builder()
    ///     .with_subject_code("CSE")
    ///     .with_course_code("100")
    ///     .with_section_id("079911")
    ///     .with_section_code("A01")
    ///     .with_grading_option(GradeOption::S)
    ///     .with_unit_count(4)
    ///     .try_build()
    ///     .unwrap();
    ///
    /// let plan_res = wrapper
    ///     .default_request()
    ///     .validate_add_to_plan(&plan_add_data)
    ///     .await;
    ///
    /// match plan_res {
    ///     Ok(res) => println!("Can plan? {res}"),
    ///     Err(e) => eprintln!("Unable to plan: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn validate_add_to_plan(&self, plan_options: &PlanAdd<'_>) -> types::Result<bool> {
        let crsc_code = util::get_formatted_course_num(plan_options.course_code.as_ref());
        process_post_response(
            self.raw
                .info
                .req_post(PLAN_EDIT)
                .form(&[
                    ("section", plan_options.section_id.as_ref()),
                    ("subjcode", plan_options.subject_code.as_ref()),
                    ("crsecode", crsc_code.as_str()),
                    ("termcode", self.raw.info.term),
                ])
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
    /// Here, we will add the course `POLI 145`, which has section ID `278941` and section code
    /// `A00`, to our plan.
    /// ```rust,no_run
    /// use webweg::wrapper::input_types::{GradeOption, PlanAdd};
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let plan_add_data = PlanAdd::builder()
    ///     .with_subject_code("CSE")
    ///     .with_course_code("100")
    ///     .with_section_id("079911")
    ///     .with_section_code("A01")
    ///     .with_grading_option(GradeOption::S)
    ///     .with_unit_count(4)
    ///     .try_build()
    ///     .unwrap();
    ///
    /// let plan_res = wrapper
    ///     .default_request()
    ///     .add_to_plan(plan_add_data, true)
    ///     .await;
    ///
    /// match plan_res {
    ///     Ok(res) => println!("Planned? {res}"),
    ///     Err(e) => eprintln!("Unable to plan: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn add_to_plan(
        &self,
        plan_options: PlanAdd<'_>,
        validate: bool,
    ) -> types::Result<bool> {
        let u = plan_options.unit_count.to_string();
        let crsc_code = util::get_formatted_course_num(plan_options.course_code.as_ref());

        if validate {
            // We need to call the edit endpoint first, or else we'll have issues where we don't
            // actually enroll in every component of the course.
            // Also, this can potentially return "false" due to you not being able to enroll in the
            // class, e.g. the class you're trying to plan is a major-restricted class.
            self.validate_add_to_plan(&plan_options)
                .await
                .unwrap_or(false);
        }

        process_post_response(
            self.raw
                .info
                .req_post(PLAN_ADD)
                .form(&[
                    ("subjcode", plan_options.subject_code.as_ref()),
                    ("crsecode", crsc_code.as_str()),
                    ("sectnum", plan_options.section_id.as_ref()),
                    ("sectcode", plan_options.section_code.as_ref()),
                    ("unit", u.as_str()),
                    (
                        "grade",
                        plan_options
                            .grading_option
                            .unwrap_or(GradeOption::L)
                            .as_str(),
                    ),
                    ("termcode", self.raw.info.term),
                    (
                        "schedname",
                        match plan_options.schedule_name {
                            Some(ref r) => r.as_ref(),
                            None => DEFAULT_SCHEDULE_NAME,
                        },
                    ),
                ])
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
    /// Here, we will remove the planned course with section ID `123456` from our default schedule.
    /// ```rust,no_run
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let unplan_res = wrapper
    ///     .default_request()
    ///     .remove_from_plan("123456", None)
    ///     .await;
    ///
    /// match unplan_res {
    ///     Ok(res) => println!("Unplanned? {res}"),
    ///     Err(e) => eprintln!("Unable to remove from plan: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn remove_from_plan(
        &self,
        section_id: impl AsRef<str>,
        schedule_name: Option<&str>,
    ) -> types::Result<bool> {
        process_post_response(
            self.raw
                .info
                .req_post(PLAN_REMOVE)
                .form(&[
                    ("sectnum", section_id.as_ref()),
                    ("termcode", self.raw.info.term),
                    ("schedname", schedule_name.unwrap_or(DEFAULT_SCHEDULE_NAME)),
                ])
                .send()
                .await,
        )
        .await
    }

    /// Validates that the section that you are trying to enroll in is valid.
    ///
    /// # Parameters
    /// - `add_type`: The add type. As a warning, specifying `DecideForMe` will incur extra
    /// requests (searching by section ID, then searching for course).
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
    /// use webweg::wrapper::input_types::{AddType, EnrollWaitAdd};
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let enroll_options = EnrollWaitAdd::builder()
    ///     .with_section_id("260737")
    ///     .try_build()
    ///     .unwrap();
    ///
    /// let add_res = wrapper
    ///     .default_request()
    ///     .validate_add_section(AddType::Enroll, &enroll_options)
    ///     .await;
    ///
    /// match add_res {
    ///     Ok(res) => println!("Can add? {res}"),
    ///     Err(e) => eprintln!("Unable to add: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn validate_add_section(
        &self,
        add_type: AddType,
        enroll_options: &EnrollWaitAdd<'_>,
    ) -> types::Result<bool> {
        let base_edit_url = match add_type {
            AddType::Enroll => ENROLL_EDIT,
            AddType::Waitlist => WAITLIST_EDIT,
            AddType::DecideForMe => match self
                .get_add_type(enroll_options.section_id.as_ref())
                .await?
            {
                ExplicitAddType::Enroll => ENROLL_EDIT,
                ExplicitAddType::Waitlist => WAITLIST_EDIT,
            },
        };

        process_post_response(
            self.raw
                .info
                .req_post(base_edit_url)
                .form(&[
                    // These are required
                    ("section", enroll_options.section_id.as_ref()),
                    ("termcode", self.raw.info.term),
                    // These are optional.
                    ("subjcode", ""),
                    ("crsecode", ""),
                ])
                .send()
                .await,
        )
        .await
    }

    /// Checks whether the user can enroll or waitlist into a section.
    ///
    /// Keep in mind that this implementation does _not_ check if you are able to enroll
    /// into a class, just that there are enough seats for you to enroll.
    ///
    /// # Parameters
    /// - `section_id`: The section ID to check.
    ///
    /// # Returns
    /// An enum value that can either be `Enroll` or `Waitlist` depending on whether
    /// the user can enroll into the specified section.
    pub async fn get_add_type(&self, section_id: &str) -> types::Result<ExplicitAddType> {
        let search_res = self
            .search_courses(SearchType::BySection(section_id.to_string()))
            .await?;

        if search_res.is_empty() {
            return Err(WrapperError::SectionIdNotFound(
                section_id.into(),
                SectionIdNotFoundContext::Catalog,
            ));
        }

        let subject_code = search_res[0].subj_code.trim();
        let course_code = search_res[0].course_code.trim();

        let section_info = self
            .get_enrollment_count(subject_code, course_code)
            .await?
            .into_iter()
            .find(|sec| sec.section_id == section_id);
        if let Some(info) = section_info {
            if info.has_seats() {
                Ok(ExplicitAddType::Enroll)
            } else {
                Ok(ExplicitAddType::Waitlist)
            }
        } else {
            // In theory, this should never hit.
            Err(WrapperError::SectionIdNotFound(
                section_id.into(),
                SectionIdNotFoundContext::Catalog,
            ))
        }
    }

    /// Enrolls in, or waitlists, a class.
    ///
    /// # Parameters
    /// - `add_type`: The add type (either `Enroll`, `Waitlist`, for `DecideForMe`). As a warning,
    /// `DecideForMe` will incur extra requests.
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
    /// Here, we will enroll in the course with section ID `260737`, and with the default grading
    /// option and unit count.
    /// ```rust,no_run
    /// use webweg::wrapper::input_types::{AddType, EnrollWaitAdd};
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let enroll_options = EnrollWaitAdd::builder()
    ///     .with_section_id("260737")
    ///     .try_build()
    ///     .unwrap();
    ///
    /// let add_res = wrapper
    ///     .default_request()
    ///     // Let the library decide if we should enroll or waitlist
    ///     .add_section(AddType::DecideForMe, enroll_options, true)
    ///     .await;
    ///
    /// match add_res {
    ///     Ok(res) => println!("Added? {res}"),
    ///     Err(e) => eprintln!("Unable to add: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn add_section(
        &self,
        add_type: AddType,
        enroll_options: EnrollWaitAdd<'_>,
        validate: bool,
    ) -> types::Result<bool> {
        let base_reg_url = match add_type {
            AddType::Enroll => ENROLL_ADD,
            AddType::Waitlist => WAITLIST_ADD,
            AddType::DecideForMe => match self
                .get_add_type(enroll_options.section_id.as_ref())
                .await?
            {
                ExplicitAddType::Enroll => ENROLL_ADD,
                ExplicitAddType::Waitlist => WAITLIST_ADD,
            },
        };
        let u = match enroll_options.unit_count {
            Some(r) => r.to_string(),
            None => "".to_string(),
        };

        if validate {
            self.validate_add_section(add_type, &enroll_options).await?;
        }

        process_post_response(
            self.raw
                .info
                .req_post(base_reg_url)
                .form(&[
                    // These are required
                    ("section", enroll_options.section_id.as_ref()),
                    ("termcode", self.raw.info.term),
                    // These are optional.
                    ("unit", u.as_str()),
                    (
                        "grade",
                        enroll_options
                            .grading_option
                            .unwrap_or(GradeOption::L)
                            .as_str(),
                    ),
                    ("crsecode", ""),
                    ("subjcode", ""),
                ])
                .send()
                .await,
        )
        .await?;

        // This will always return true
        process_post_response(
            self.raw
                .info
                .req_post(PLAN_REMOVE_ALL)
                .form(&[
                    ("sectnum", enroll_options.section_id.as_ref()),
                    ("termcode", self.raw.info.term),
                ])
                .send()
                .await,
        )
        .await
    }

    /// Drops a section.
    ///
    /// # Parameters
    /// - `prev_enroll_status`: Your enrollment status (either `Enroll` or `Waitlist` if you
    /// are enrolled or waitlisted in the section, respectively).
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
    /// use webweg::wrapper::input_types::ExplicitAddType;
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies go here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let drop_res = wrapper
    ///     .default_request()
    ///     .drop_section(ExplicitAddType::Enroll, "123456")
    ///     .await;
    ///
    /// match drop_res {
    ///     Ok(res) => println!("Dropped? {res}"),
    ///     Err(e) => eprintln!("Unable to drop: {e}"),
    /// }
    /// # }
    /// ```
    pub async fn drop_section(
        &self,
        prev_enroll_status: ExplicitAddType,
        section_id: impl AsRef<str>,
    ) -> types::Result<bool> {
        let base_reg_url = match prev_enroll_status {
            ExplicitAddType::Enroll => ENROLL_DROP,
            ExplicitAddType::Waitlist => WAITLIST_DROP,
        };

        process_post_response(
            self.raw
                .info
                .req_post(base_reg_url)
                .form(&[
                    // These parameters are optional
                    ("subjcode", ""),
                    ("crsecode", ""),
                    // But these are required
                    ("section", section_id.as_ref()),
                    ("termcode", self.raw.info.term),
                ])
                .send()
                .await,
        )
        .await
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
    /// Renaming the schedule "`Test Schedule`" to "`Another Schedule`." Keep in mind that you
    /// should be doing error handling here.
    ///
    /// ```rust,no_run
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let default_requester = wrapper.default_request();
    /// assert!(!default_requester
    ///     .get_schedule_list()
    ///     .await
    ///     .unwrap()
    ///     .contains(&"Another Schedule".to_string()));
    ///
    /// default_requester
    ///     .rename_schedule("Test Schedule", "Another Schedule")
    ///     .await
    ///     .expect("An error occurred.");
    ///
    /// assert!(default_requester
    ///     .get_schedule_list()
    ///     .await
    ///     .unwrap()
    ///     .contains(&"Another Schedule".to_string()));
    /// # }
    /// ```
    pub async fn rename_schedule(
        &self,
        old_name: impl AsRef<str>,
        new_name: impl AsRef<str>,
    ) -> types::Result<bool> {
        // Can't rename your default schedule.
        if old_name.as_ref() == DEFAULT_SCHEDULE_NAME {
            return Err(WrapperError::InputError(
                "old_name",
                "You cannot rename the default schedule",
            ));
        }

        process_post_response(
            self.raw
                .info
                .req_post(RENAME_SCHEDULE)
                .form(&[
                    ("termcode", self.raw.info.term),
                    ("oldschedname", old_name.as_ref()),
                    ("newschedname", new_name.as_ref()),
                ])
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
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let default_requester = wrapper.default_request();
    /// assert!(default_requester
    ///     .get_schedule_list()
    ///     .await
    ///     .unwrap()
    ///     .contains(&"Test Schedule".to_string()));
    /// default_requester
    ///     .remove_schedule("Test Schedule")
    ///     .await
    ///     .expect("An error occurred.");
    /// assert!(!default_requester
    ///     .get_schedule_list()
    ///     .await
    ///     .unwrap()
    ///     .contains(&"Test Schedule".to_string()));
    /// # }
    /// ```
    pub async fn remove_schedule(&self, schedule_name: impl AsRef<str>) -> types::Result<bool> {
        // Can't remove your default schedule.
        if schedule_name.as_ref() == DEFAULT_SCHEDULE_NAME {
            return Err(WrapperError::InputError(
                "schedule_name",
                "You cannot remove the default schedule.",
            ));
        }

        process_post_response(
            self.raw
                .info
                .req_post(REMOVE_SCHEDULE)
                .form(&[
                    ("termcode", self.raw.info.term),
                    ("schedname", schedule_name.as_ref()),
                ])
                .send()
                .await,
        )
        .await
    }

    /// Adds an event to your WebReg calendar, or edits an existing event.
    ///
    /// Keep in mind that if you edit an event, WebReg effectively just removes
    /// the old event and then adds the new event, so the timestamp will change.
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
    /// use webweg::wrapper::input_types::{DayOfWeek, EventAdd};
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let event_to_add = EventAdd::builder()
    ///     .with_name("Clown on Alex")
    ///     .with_location("B250")
    ///     .with_day(DayOfWeek::Monday)
    ///     .with_day(DayOfWeek::Friday)
    ///     .with_start_time(8, 30)
    ///     .with_end_time(10, 45)
    ///     .try_build()
    ///     .unwrap();
    ///
    /// // Adding an event
    /// let add_res = wrapper
    ///     .default_request()
    ///     .add_or_edit_event(event_to_add, None)
    ///     .await;
    /// match add_res {
    ///     Ok(o) => println!("Added event? {o}"),
    ///     Err(e) => println!("Error! {e}"),
    /// }
    ///
    /// let event_to_replace_with = EventAdd::builder()
    ///     .with_name("Clown on Kira and Ruby")
    ///     .with_location("CSE")
    ///     .with_day(DayOfWeek::Monday)
    ///     .with_day(DayOfWeek::Wednesday)
    ///     .with_start_time(8, 30)
    ///     .with_end_time(10, 45)
    ///     .try_build()
    ///     .unwrap();
    ///
    /// // Replace the event with the specified timestamp `2022-09-09 21:50:16.846885`
    /// // with another event.
    /// let replace_res = wrapper
    ///     .default_request()
    ///     .add_or_edit_event(event_to_replace_with, Some("2022-09-09 21:50:16.846885"))
    ///     .await;
    /// match replace_res {
    ///     Ok(o) => println!("Edited event? {o}"),
    ///     Err(e) => println!("Error! {e}"),
    /// };
    /// # }
    /// ```
    pub async fn add_or_edit_event(
        &self,
        event_info: EventAdd<'_>,
        event_timestamp: impl Into<Option<&str>>,
    ) -> types::Result<bool> {
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

        let mut start_time_full = start_time_full.to_string();
        let mut end_time_full = end_time_full.to_string();
        while start_time_full.len() < 4 {
            start_time_full.insert(0, '0');
        }

        while end_time_full.len() < 4 {
            end_time_full.insert(0, '0');
        }

        let mut form_data = HashMap::from([
            ("termcode", self.raw.info.term),
            ("aename", event_info.event_name.as_ref()),
            ("aestarttime", start_time_full.as_str()),
            ("aeendtime", end_time_full.as_str()),
            (
                "aelocation",
                match event_info.location {
                    None => "",
                    Some(ref s) => s.as_ref(),
                },
            ),
            ("aedays", day_str.as_str()),
        ]);

        let et = event_timestamp.into();
        if let Some(timestamp) = et {
            form_data.insert("aetimestamp", timestamp);
        }

        process_post_response(
            self.raw
                .info
                .req_post(match et {
                    Some(_) => EVENT_EDIT,
                    None => EVENT_ADD,
                })
                .form(&form_data)
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
    /// use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapperBuilder::new()
    ///     .with_cookies("Your cookies here.")
    ///     .with_default_term("FA23")
    ///     .try_build_wrapper()
    ///     .unwrap();
    ///
    /// let delete_res = wrapper
    ///     .default_request()
    ///     .remove_event("2022-09-09 21:50:16.846885")
    ///     .await;
    /// match delete_res {
    ///     Ok(o) => println!("Deleted? {o}"),
    ///     Err(e) => eprintln!("Error! {e}"),
    /// };
    /// # }
    /// ```
    pub async fn remove_event(&self, event_timestamp: impl AsRef<str>) -> types::Result<bool> {
        process_post_response(
            self.raw
                .info
                .req_post(EVENT_REMOVE)
                .form(&[
                    ("aetimestamp", event_timestamp.as_ref()),
                    ("termcode", self.raw.info.term),
                ])
                .send()
                .await,
        )
        .await
    }

    /// Associates the term bound by this request to the cookies that are provided
    /// as part of this overridden request.
    ///
    /// # Returns
    /// A result, where nothing is returned if everything went well and an error is returned
    /// if something went wrong.
    pub async fn associate_term(&self) -> types::Result<()> {
        associate_term_helper(&self.raw.info, self.raw.info.term).await
    }
}
