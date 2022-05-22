use crate::webreg_clean_defn::MeetingDay;
use crate::webreg_raw_defn::RawWebRegMeeting;

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
    assert!(day_code.chars().into_iter().all(|x| x.is_numeric()));

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
/// - `dow_str`: The day code string. This should only contain integers between 0 and 6, both
/// inclusive.
///
/// # Returns
/// A string with the days of the week.
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
