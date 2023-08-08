use std::time::SystemTime;

use crate::raw_types::RawWebRegMeeting;
use crate::types::MeetingDay;

/// Gets the meeting type (e.g. Lecture, Final Exam, Discussion, etc.) and the meeting time from
/// an arbitrary `WebRegMeeting`.
///
/// # Parameters
/// - `w_meeting`: The WebReg meeting to check.
///
/// # Returns
/// A tuple where:
/// - the first element is the meeting type
/// - the second element is/are the day(s) that this meeting occurs
#[inline]
pub fn parse_meeting_type_date(w_meeting: &RawWebRegMeeting) -> (&str, MeetingDay) {
    let special_meeting = w_meeting.special_meeting.trim();
    if !special_meeting.is_empty() && special_meeting != "TBA" {
        assert!(!w_meeting.section_start_date.is_empty());
        return (
            special_meeting,
            MeetingDay::OneTime(w_meeting.start_date.to_string()),
        );
    }

    let regular_meeting = w_meeting.meeting_type.trim();
    let day_code = w_meeting.day_code.trim();
    assert!(day_code.chars().all(|x| x.is_numeric()));

    if day_code.is_empty() {
        (regular_meeting, MeetingDay::None)
    } else {
        (
            regular_meeting,
            MeetingDay::Repeated(parse_day_code(day_code)),
        )
    }
}

/// Parses the days of the week from a day code string.
///
/// # Parameters
/// - `day_code_str`: The day code string. This should only contain integers between 0 and 6, both
/// inclusive.
///
/// # Returns
/// A string with the days of the week.
///
/// # Example
/// An input of `135` would return `["M", "W", "F"]`.
pub fn parse_day_code(day_code_str: &str) -> Vec<String> {
    let mut s = vec![];
    day_code_str.chars().for_each(|c| {
        if !c.is_numeric() {
            return;
        }

        match c {
            '0' => s.push("Su".to_string()),
            '1' => s.push("M".to_string()),
            '2' => s.push("Tu".to_string()),
            '3' => s.push("W".to_string()),
            '4' => s.push("Th".to_string()),
            '5' => s.push("F".to_string()),
            '6' => s.push("Sa".to_string()),
            _ => {}
        };
    });

    s
}

const DAYS: [&str; 7] = ["M", "Tu", "W", "Th", "F", "Sa", "Su"];

/// Parses a binary string representing the days that are active.
///
/// # Parameters
/// - `bin_str`: The binary string. Must be length 7. The first bit
/// represents Monday, the second bit represents Tuesday, and so on.
/// The `1` bit means that the day is active, and the `0` bit means
/// the day is inactive.
///
/// # Returns
/// A string with the days of the week.
///
/// # Example
/// An input of `1010101` would return `["M", "W", "F", "Su"]`.
pub fn parse_binary_days(bin_str: &str) -> Vec<String> {
    let mut days = vec![];
    if bin_str.len() == 7 {
        let day_vec = bin_str.chars().collect::<Vec<_>>();
        for (idx, day) in DAYS.iter().enumerate() {
            if day_vec[idx] == '1' {
                days.push(day.to_string());
            }
        }
    }

    days
}

const TERM_ARR: [(&str, (i64, i64)); 7] = [
    ("SP", (5200, 22)), // SP22
    ("S1", (5210, 22)), // S122
    ("S2", (5220, 22)), // S222
    ("S3", (5230, 22)), // S322
    ("SU", (5240, 22)), // SU22
    ("FA", (5250, 22)), // FA22
    ("WI", (5260, 23)), // WI23
];

/// Gets the term ID based on the term that was passed in.
///
/// # Parameters
/// - `term`: The term
///
/// # Returns
/// The term ID, if valid. If `0` is returned, then the input
/// is invalid.
pub fn get_term_seq_id(term: impl AsRef<str>) -> i64 {
    let term = term.as_ref();
    if term.len() != 4 {
        return 0;
    }

    let term_init = &term[..2];
    let (base_seq_id, base_year) = match TERM_ARR.iter().find(|(term, _)| *term == term_init) {
        Some((_, data)) => *data,
        None => return 0,
    };

    let quarter_yr = match term[2..].parse::<i64>() {
        Ok(o) => o,
        Err(_) => return 0,
    };

    // 70 is the difference between each term, apparently
    // For example, the seqid of FA22 and FA23 has a difference of 70
    70 * (quarter_yr - base_year) + base_seq_id
}

/// Gets the formatted course code so that it can be recognized by
/// WebReg's internal API.
///
/// # Parameters
/// - `course_code`: The course code, e.g. if you have the course
/// `CSE 110`, you would put `110`.
///
/// # Returns
/// The formatted course code for WebReg.
#[inline(always)]
pub fn get_formatted_course_num(course_code: &str) -> String {
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

/// Gets the current epoch time.
///
/// # Returns
/// The current time.
#[inline(always)]
pub(crate) fn get_epoch_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// Gets the instructor's names.
///
/// # Parameters
/// - `instructor_name`: The raw name.
///
/// # Returns
/// The parsed instructor's names, as a vector.
#[inline(always)]
pub(crate) fn get_instructor_names(instructor_name: &str) -> Vec<String> {
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
#[inline(always)]
pub(crate) fn get_all_instructors<I>(instructors: I) -> Vec<String>
where
    I: Iterator<Item = String>,
{
    let mut all_inst = instructors.collect::<Vec<_>>();
    all_inst.sort();
    all_inst.dedup();
    all_inst
}

/// Formats multiple course inputs into a string that WebReg can recognize
/// for its search queries.
///
/// # Parameters
/// - `query`: The vector of courses to format. Each element can either be a
/// full course code (e.g., `CSE 100`) or a partial course code (e.g., `CSE`
/// or `100`).
///
/// # Returns
/// The formatted string.
pub fn format_multiple_courses<T: AsRef<str>>(query: &[T]) -> String {
    // The way the string query is formatted is
    // - each course (or part of course) is separated by ';'
    // - each whitespace within the course is replaced with ':'
    query
        .iter()
        .map(|x| {
            x.as_ref()
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|x| !x.is_empty())
        // Essentially, for each course...
        .map(|course| {
            // For each course vector, let's see what we should do.
            // There are several cases to consider for each item; an item can either be one of:
            // 1. SubjCode
            // 2. Subj
            // 3. Code
            // 4. Subj, Code [handled implicitly by cases 2 and 3]
            //
            // For now, we'll go through each individual item in the vector and
            // process it
            course
                .iter()
                .map(|item| {
                    match item.chars().next() {
                        // Case 3
                        Some(c) if c.is_ascii_digit() => get_formatted_course_num(item),
                        // Case 1 or 2
                        Some(_) => {
                            // See if case 1 is what we're working with
                            if let Some(idx) = item.find(|c: char| c.is_ascii_digit()) {
                                let subj = &item[..idx];
                                let csrc = &item[idx..];
                                format!("{}:{}", subj, get_formatted_course_num(csrc))
                            } else {
                                item.to_string()
                            }
                        }
                        // This should never hit
                        _ => "".to_string(),
                    }
                })
                .collect::<Vec<_>>()
                .join(":")
        })
        .collect::<Vec<_>>()
        .join(";")
        .to_uppercase()
}
