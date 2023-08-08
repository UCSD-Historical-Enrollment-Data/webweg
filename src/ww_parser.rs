use std::cmp::max;
use std::collections::{HashMap, HashSet};

use url::Url;

use crate::constants::*;
use crate::raw_types::{
    RawCoursePrerequisite, RawEvent, RawPrerequisite, RawScheduledMeeting, RawWebRegMeeting,
};
use crate::types::{
    CoursePrerequisite, CourseSection, Courses, EnrollmentStatus, Event, Events, Meeting,
    MeetingDay, PrerequisiteInfo, Schedule, ScheduledSection, TimeType, WrapperError,
};
use crate::util::parse_binary_days;
use crate::wrapper::input_types::SearchType;
use crate::{types, util};

/// Processes the vector containing raw prerequisites information.
///
/// # Parameters
/// - `res`: The vector of raw prerequisite objects.
///
/// # Returns
/// Either the [arsed prerequisite information or an error.
pub fn parse_prerequisites(res: Vec<RawPrerequisite>) -> types::Result<PrerequisiteInfo> {
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
            RawPrerequisite::Course(c) => req_map.entry(&c.prereq_seq_id).or_insert(vec![]).push(c),
            RawPrerequisite::Test(t) => all_reqs
                .exam_prerequisites
                .push(t.test_title.trim().to_string()),
        }
    }

    for (_, reqs) in req_map {
        let mut cleaned_reqs: Vec<CoursePrerequisite> = vec![];
        for req in reqs {
            cleaned_reqs.push(CoursePrerequisite {
                subj_course_id: format!("{} {}", req.subject_code.trim(), req.course_code.trim()),
                course_title: req.course_title.trim().to_string(),
            });
        }

        all_reqs.course_prerequisites.push(cleaned_reqs);
    }

    Ok(all_reqs)
}

/// Processes the vector containing the raw scheduled meeting objects.
///
/// # Parameters
/// - `res`: The vector of raw scheduled meeting objects.
///
/// # Returns
/// Either the parsed schedule information or an error.
pub fn parse_schedule(res: Vec<RawScheduledMeeting>) -> types::Result<Schedule> {
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

    let mut schedule: Schedule = vec![];

    // We next begin processing the general sections. Each key/value pair represents a course
    // section. We do not care about the key; the value is a vector of meetings, which we will
    // clean up.
    //
    // Every meeting is separated. For example, if we have a MWF meeting, then there will
    // be three meeting objects -- one for M, one for W, and one for F.
    for (_, sch_meetings) in base_group_secs {
        // First, let's get all instructors associated with this course section.
        let instructors = util::get_all_instructors(
            sch_meetings
                .iter()
                .flat_map(|x| util::get_instructor_names(&x.person_full_name)),
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
                start_min: TimeType::try_from(main.start_time_min)
                    .map_err(|_| WrapperError::BadTimeError)?,
                start_hr: TimeType::try_from(main.start_time_hr)
                    .map_err(|_| WrapperError::BadTimeError)?,
                end_min: TimeType::try_from(main.end_time_min)
                    .map_err(|_| WrapperError::BadTimeError)?,
                end_hr: TimeType::try_from(main.end_time_hr)
                    .map_err(|_| WrapperError::BadTimeError)?,
                building: main.bldg_code.trim().to_string(),
                room: main.room_code.trim().to_string(),
                instructors: util::get_instructor_names(&main.person_full_name),
            });
        }

        // Parse the remaining meetings.
        // Here, we want to parse any midterm and exam meetings.
        for meeting in sch_meetings
            .iter()
            .filter(|x| {
                x.sect_code.ends_with("00")
                    && !x.special_meeting.replace("TBA", "").trim().is_empty()
            })
            .map(|x| -> types::Result<Meeting> {
                Ok(Meeting {
                    meeting_type: x.meeting_type.to_string(),
                    meeting_days: MeetingDay::OneTime(x.start_date.to_string()),
                    start_min: TimeType::try_from(x.start_time_min)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    start_hr: TimeType::try_from(x.start_time_hr)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    end_min: TimeType::try_from(x.end_time_min)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    end_hr: TimeType::try_from(x.end_time_hr)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    building: x.bldg_code.trim().to_string(),
                    room: x.room_code.trim().to_string(),
                    instructors: util::get_instructor_names(&x.person_full_name),
                })
            })
        {
            all_meetings.push(meeting?);
        }

        // Finally, we parse the general meetings.
        for meeting in sch_meetings
            .iter()
            .filter(|x| !x.sect_code.ends_with("00"))
            .map(|x| -> types::Result<Meeting> {
                Ok(Meeting {
                    meeting_type: x.meeting_type.to_string(),
                    meeting_days: MeetingDay::Repeated(util::parse_day_code(&x.day_code)),
                    start_min: TimeType::try_from(x.start_time_min)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    start_hr: TimeType::try_from(x.start_time_hr)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    end_min: TimeType::try_from(x.end_time_min)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    end_hr: TimeType::try_from(x.end_time_hr)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    building: x.bldg_code.trim().to_string(),
                    room: x.room_code.trim().to_string(),
                    instructors: util::get_instructor_names(&x.person_full_name),
                })
            })
        {
            all_meetings.push(meeting?);
        }

        // Find the main meeting (the one that you can enroll in). This meeting object has
        // information like how many people are enrolled, capacity, etc. (the others will not).
        let main_meeting = sch_meetings
            .iter()
            .find(|m| m.enrolled_count.is_some() && m.section_capacity.is_some());

        match main_meeting {
            None => {
                // If we cannot find the meeting, then assume the schedule is deformed and return.
                return if sch_meetings.is_empty() {
                    Err(WrapperError::WrapperParsingError(format!(
                        "{} {} is deformed",
                        sch_meetings[0].sect_code, sch_meetings[0].course_code
                    )))
                } else {
                    Err(WrapperError::WrapperParsingError(
                        "schedule is deformed".to_owned(),
                    ))
                };
            }
            Some(data) => {
                // At this point, we now want to look for data like section capacity, number of
                // students on the waitlist, and so on. `data` is the main section that should
                // have all this data.
                let enrolled_count = data.enrolled_count.unwrap_or(-1);
                let section_capacity = data.section_capacity.unwrap_or(-1);

                schedule.push(ScheduledSection {
                    section_id: data.section_id.to_string(),
                    all_instructors: instructors.clone(),
                    subject_code: data.subj_code.trim().to_string(),
                    course_code: data.course_code.trim().to_string(),
                    course_title: data.course_title.trim().to_string(),
                    section_code: match sch_meetings.iter().find(|x| !x.sect_code.ends_with("00")) {
                        Some(r) => r.sect_code.to_string(),
                        None => data.sect_code.to_string(),
                    },
                    section_capacity,
                    enrolled_count,
                    available_seats: max(section_capacity - enrolled_count, 0),
                    grade_option: data.grade_option.to_string(),
                    units: data.sect_credit_hrs.trunc() as i64,
                    enrolled_status: match data.enroll_status.as_str() {
                        STATUS_ENROLL => EnrollmentStatus::Enrolled,
                        STATUS_WAITLIST => EnrollmentStatus::Waitlist {
                            waitlist_pos: data.waitlist_pos.parse().unwrap_or(-1),
                        },
                        STATUS_PLANNED => EnrollmentStatus::Planned,
                        _ => EnrollmentStatus::Unknown,
                    },
                    waitlist_ct: data.count_on_waitlist.unwrap_or(0),
                    meetings: all_meetings,
                });
            }
        }
    }

    // Now, we look into parsing the special sections. This is trivial to parse.
    // Note: we're making the assumption that these sections have one meeting.
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
            all_instructors: util::get_all_instructors(
                sch_meetings
                    .iter()
                    .flat_map(|x| util::get_instructor_names(&x.person_full_name)),
            ),
            subject_code: sch_meetings[0].subj_code.trim().to_string(),
            course_code: sch_meetings[0].course_code.trim().to_string(),
            course_title: sch_meetings[0].course_title.trim().to_string(),
            section_code: sch_meetings[0].sect_code.to_string(),
            section_capacity,
            enrolled_count,
            available_seats: max(section_capacity - enrolled_count, 0),
            grade_option: sch_meetings[0].grade_option.trim().to_string(),
            units: sch_meetings[0].sect_credit_hrs.trunc() as i64,
            enrolled_status: match sch_meetings[0].enroll_status.as_str() {
                STATUS_ENROLL => EnrollmentStatus::Enrolled,
                STATUS_WAITLIST => EnrollmentStatus::Waitlist {
                    waitlist_pos: sch_meetings[0].waitlist_pos.parse().unwrap_or(-1),
                },
                STATUS_PLANNED => EnrollmentStatus::Planned,
                _ => EnrollmentStatus::Unknown,
            },
            waitlist_ct: sch_meetings[0].count_on_waitlist.unwrap_or(0),
            meetings: vec![Meeting {
                meeting_type: sch_meetings[0].meeting_type.to_string(),
                meeting_days: parsed_day_code,
                start_min: TimeType::try_from(sch_meetings[0].start_time_min)
                    .map_err(|_| WrapperError::BadTimeError)?,
                start_hr: TimeType::try_from(sch_meetings[0].start_time_hr)
                    .map_err(|_| WrapperError::BadTimeError)?,
                end_min: TimeType::try_from(sch_meetings[0].end_time_min)
                    .map_err(|_| WrapperError::BadTimeError)?,
                end_hr: TimeType::try_from(sch_meetings[0].start_time_hr)
                    .map_err(|_| WrapperError::BadTimeError)?,
                building: sch_meetings[0].bldg_code.trim().to_string(),
                room: sch_meetings[0].room_code.trim().to_string(),
                instructors: util::get_instructor_names(&sch_meetings[0].person_full_name),
            }],
        });
    }

    Ok(schedule)
}

/// Processes the vector containing raw meeting information into enrollment
/// count data.
///
/// # Parameters
/// - `meetings`: The vector of meetings.
/// - `subj_num`: The subject course number (e.g., `CSE 100`).
///
/// # Returns
/// Either the parsed enrollment count information or an error. Note that this
/// function will return a vector of course sections, but most information are
/// not filled out.
pub fn parse_enrollment_count(
    meetings: Vec<RawWebRegMeeting>,
    subj_num: String,
) -> types::Result<Courses> {
    if meetings.is_empty() {
        return Ok(vec![]);
    }

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
            is_visible: x.is_visible(),
            subj_course_id: subj_num.to_owned(),
            section_id: x.section_id.trim().to_string(),
            section_code: x.sect_code.trim().to_string(),
            all_instructors: util::get_instructor_names(&x.person_full_name),
            available_seats: max(x.avail_seat, 0),
            enrolled_ct: x.enrolled_count,
            total_seats: x.section_capacity,
            waitlist_ct: x.count_on_waitlist,
            meetings: vec![],
        })
        .collect())
}

pub enum CourseInfoType {
    Full,
    Count,
}

/// Runs either the `parse_course_info` or `parse_enrollment_count`
/// function depending on what data is requested.
///
/// # Parameters
/// - `meetings`: The vector of meetings.
/// - `subj_num`: The subject course number (e.g., `CSE 100`).
/// - `data_type`: The data type (either "Full" for all data, or "Count"
///                for partial).
///
/// # Returns
/// Either the parsed course information or an error.
pub fn parse_course_info_or_enrollment_ct(
    parsed: Vec<RawWebRegMeeting>,
    subj_num: String,
    data_type: CourseInfoType,
) -> types::Result<Courses> {
    match data_type {
        CourseInfoType::Full => parse_course_info(parsed, subj_num),
        CourseInfoType::Count => parse_enrollment_count(parsed, subj_num),
    }
}

/// Processes the vector containing raw meeting information into parsed
/// course information.
///
/// # Parameters
/// - `meetings`: The vector of meetings.
/// - `subj_num`: The subject course number (e.g., `CSE 100`).
///
/// # Returns
/// Either the parsed course information or an error.
pub fn parse_course_info(
    parsed: Vec<RawWebRegMeeting>,
    subj_num: String,
) -> types::Result<Courses> {
    let mut sections: Courses = vec![];
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
                is_visible: meeting.is_visible(),
                subj_course_id: subj_num.to_owned(),
                section_id: meeting.section_id.trim().to_string(),
                section_code: meeting.sect_code.trim().to_string(),
                all_instructors: util::get_instructor_names(&meeting.person_full_name),
                // Because it turns out that you can have negative available seats.
                available_seats: max(meeting.avail_seat, 0),
                enrolled_ct: meeting.enrolled_count,
                total_seats: meeting.section_capacity,
                waitlist_ct: meeting.count_on_waitlist,
                meetings: vec![Meeting {
                    start_hr: TimeType::try_from(meeting.start_time_hr)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    start_min: TimeType::try_from(meeting.start_time_min)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    end_hr: TimeType::try_from(meeting.end_time_hr)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    end_min: TimeType::try_from(meeting.end_time_min)
                        .map_err(|_| WrapperError::BadTimeError)?,
                    meeting_type: m_type.to_string(),
                    meeting_days: m_days,
                    building: meeting.bldg_code.trim().to_string(),
                    room: meeting.room_code.trim().to_string(),
                    instructors: util::get_instructor_names(&meeting.person_full_name),
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

    struct GroupedSection<'a, T> {
        /// All general meetings. These include meetings that are consistent across *all* sections.
        /// For example, lectures and final exams.
        pub general_meetings: Vec<&'a T>,

        /// All unique meetings. These are generally meetings that are unique the one section.
        /// For example, discussions.
        pub child_meetings: Vec<&'a T>,
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
        let sec_fam = meeting.sect_code.chars().next().ok_or_else(|| {
            WrapperError::WrapperParsingError("Non-existent section code.".into())
        })?;

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
            dbg!(&subj_num);
            continue;
        }

        // First, get the base instructors. These are all of the instructors for the lectures.
        // Note that, for a majority of courses, there will only be one instructor. However,
        // some courses may have two or more instructors.
        let base_instructors = util::get_all_instructors(
            entry
                .general_meetings
                .iter()
                .flat_map(|x| util::get_instructor_names(&x.person_full_name)),
        );

        // Define a closure that takes in a slice `from` (which is a slice of all meetings that
        // we want to read in) and a vector `to` (which is where we want to write these
        // meetings to).
        let process_meetings =
            |from: &[&RawWebRegMeeting], to: &mut Vec<Meeting>| -> types::Result<()> {
                for meeting in from {
                    let (m_m_type, m_days) = util::parse_meeting_type_date(meeting);

                    to.push(Meeting {
                        meeting_type: m_m_type.to_string(),
                        meeting_days: m_days,
                        building: meeting.bldg_code.trim().to_string(),
                        room: meeting.room_code.trim().to_string(),
                        start_hr: TimeType::try_from(meeting.start_time_hr)
                            .map_err(|_| WrapperError::BadTimeError)?,
                        start_min: TimeType::try_from(meeting.start_time_min)
                            .map_err(|_| WrapperError::BadTimeError)?,
                        end_hr: TimeType::try_from(meeting.end_time_hr)
                            .map_err(|_| WrapperError::BadTimeError)?,
                        end_min: TimeType::try_from(meeting.end_time_min)
                            .map_err(|_| WrapperError::BadTimeError)?,
                        // These are instructors specifically assigned to this meeting. For most
                        // cases, these will be the same instructors assigned to the lecture
                        // meetings.
                        instructors: util::get_instructor_names(&meeting.person_full_name),
                    });
                }

                Ok(())
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
                is_visible: entry.general_meetings[0].is_visible(),
                subj_course_id: subj_num.to_owned(),
                section_id: entry.general_meetings[0].section_id.to_owned(),
                section_code: entry.general_meetings[0].sect_code.to_owned(),
                all_instructors: util::get_instructor_names(
                    &entry.general_meetings[0].person_full_name,
                ),
                available_seats: max(entry.general_meetings[0].avail_seat, 0),
                enrolled_ct: entry.general_meetings[0].enrolled_count,
                total_seats: entry.general_meetings[0].section_capacity,
                waitlist_ct: entry.general_meetings[0].count_on_waitlist,
                meetings: vec![],
            };

            // Then, iterate through the rest of the general meetings.
            process_meetings(&entry.general_meetings, &mut section.meetings)?;
            // Finally, add it to the sections.
            sections.push(section);
            continue;
        }

        // Otherwise, we essentially repeat the same process above. The only difference is that
        // we clone 'section' for each child meeting.
        for c_meeting in &entry.child_meetings {
            let mut instructors = base_instructors.clone();
            instructors.append(&mut util::get_instructor_names(&c_meeting.person_full_name));
            instructors.sort();
            instructors.dedup();

            // Process the general section info.
            let mut section = CourseSection {
                is_visible: c_meeting.is_visible(),
                subj_course_id: subj_num.to_owned(),
                section_id: c_meeting.section_id.to_owned(),
                section_code: c_meeting.sect_code.to_owned(),
                all_instructors: instructors,
                available_seats: max(c_meeting.avail_seat, 0),
                enrolled_ct: c_meeting.enrolled_count,
                total_seats: c_meeting.section_capacity,
                waitlist_ct: c_meeting.count_on_waitlist,
                meetings: vec![],
            };

            // Iterate through the general and child meetings.
            process_meetings(&entry.general_meetings, &mut section.meetings)?;
            process_meetings(&[c_meeting], &mut section.meetings)?;
            // Finally, add it to the sections as usual.
            sections.push(section);
        }
    }

    Ok(sections)
}

/// Builds the URL that can be used to search courses on WebReg.
///
/// # Parameters
/// - `filter_by`: The search type.
/// - `term`: The term.
///
/// # Returns
/// The resulting URL that can be used to search for courses.
pub(crate) fn build_search_course_url(filter_by: SearchType, term: &str) -> types::Result<Url> {
    Ok(match filter_by {
        SearchType::BySection(section) => Url::parse_with_params(
            WEBREG_SEARCH_SEC,
            &[("sectionid", section), ("termcode", term.into())],
        )?,
        SearchType::ByMultipleSections(sections) => Url::parse_with_params(
            WEBREG_SEARCH_SEC,
            &[
                ("sectionid", sections.join(":").as_str()),
                ("termcode", term),
            ],
        )?,
        SearchType::Advanced(request_filter) => {
            let subject_code = if request_filter.subjects.is_empty() {
                "".to_string()
            } else {
                // Subjects are separated by ':'
                request_filter.subjects.join(":")
            };

            let course_code = if request_filter.courses.is_empty() {
                "".to_string()
            } else {
                util::format_multiple_courses(&request_filter.courses)
            };

            let department = if request_filter.departments.is_empty() {
                "".to_string()
            } else {
                // Departments are separated by ':'
                request_filter.departments.join(":")
            };

            let professor = match &request_filter.instructor {
                Some(r) => r.to_uppercase(),
                None => "".to_string(),
            };

            let title = match &request_filter.title {
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
                    ("termcode", term),
                    ("_", util::get_epoch_time().to_string().as_str()),
                ],
            )?
        }
    })
}

/// Parses events from the vector of raw events.
///
/// # Parameters
/// - `raw_events`: The raw events.
///
/// # Returns
/// The parsed events.
pub(crate) fn parse_get_events(raw_events: Vec<RawEvent>) -> types::Result<Events> {
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
            start_hr,
            start_min,
            end_hr,
            end_min,
            name: event.description,
            days: parse_binary_days(&event.days),
            timestamp: event.time_stamp,
        });
    }

    Ok(res)
}
