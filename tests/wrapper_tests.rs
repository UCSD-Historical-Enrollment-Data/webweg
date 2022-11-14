extern crate core;

use reqwest::Client;
use std::collections::HashSet;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use webweg::types::{CourseSection, Meeting};
use webweg::wrapper::WebRegWrapper;

const TERM: &str = "FA22";

/// Gets the wrapper for testing.
///
/// # Returns
/// The new wrapper.
pub fn get_wrapper() -> WebRegWrapper {
    let file = Path::new("cookie.txt");
    if !file.exists() {
        panic!("File 'cookie.txt' does not exist.");
    }

    let cookie = fs::read_to_string(file).unwrap_or_else(|_| "".to_string());
    WebRegWrapper::new(Client::new(), cookie, TERM)
}

/// Gets all of a specific type of meeting from the sections.
///
/// # Parameters
/// - `sections`: The sections.
/// - `meeting_type`: The meeting type to filter by.
///
/// # Returns
/// The meetings.
pub fn get_all_meetings<'a>(
    sections: &'a [CourseSection],
    meeting_type: &'static str,
) -> Vec<&'a Meeting> {
    sections
        .iter()
        .flat_map(|x| {
            x.meetings
                .iter()
                .filter(|x| x.meeting_type == meeting_type)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

/// Checks if two slices are equal.
///
/// # Parameters
/// - `s1`: The first slice.
/// - `s2`: The second slice.
///
/// # Returns
/// Whether the slices are equal.
pub fn check_list_eq<T>(s1: &[T], s2: &[T]) -> bool
where
    T: Eq + Hash,
{
    let a: HashSet<_> = s1.iter().collect();
    let b: HashSet<_> = s2.iter().collect();
    a == b
}

#[cfg(test)]
mod test_course_info {
    use webweg::types::MeetingDay;

    use crate::{check_list_eq, get_all_meetings, get_wrapper};

    #[tokio::test]
    pub async fn test_cse_101() {
        let wrapper = get_wrapper();
        let mut cse_101 = wrapper
            .get_course_info("CSE", "101")
            .await
            .expect("CSE 101 is offered in this term.");
        let instructors = ["Jones, Miles E".to_string(), "Saha, Barna".to_string()];
        // There is only one section of CSE 101
        assert_eq!(1, cse_101.len());
        let cse_101 = cse_101.pop().unwrap();
        assert_eq!("A01", cse_101.section_code);
        assert!(check_list_eq(&instructors, &cse_101.all_instructors));
        assert_eq!("090035", cse_101.section_id);

        // Test meetings now
        let lecture_meeting = cse_101
            .meetings
            .iter()
            .find(|x| x.meeting_type == "LE")
            .expect("No lecture found, but a lecture should exist.");
        assert_eq!(
            MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
            lecture_meeting.meeting_days
        );
        assert_eq!(12 + 3, lecture_meeting.start_hr);
        assert_eq!(30, lecture_meeting.start_min);
        assert_eq!(12 + 4, lecture_meeting.end_hr);
        assert_eq!(50, lecture_meeting.end_min);
        assert_eq!("GH", lecture_meeting.building);
        assert_eq!("242", lecture_meeting.room);
        assert!(check_list_eq(&instructors, &lecture_meeting.instructors));

        // Test discussion
        let discussion_meeting = cse_101
            .meetings
            .iter()
            .find(|x| x.meeting_type == "DI")
            .expect("No discussion found, but a discussion should exist.");
        assert_eq!(
            MeetingDay::Repeated(vec!["W".into()]),
            discussion_meeting.meeting_days
        );
        assert_eq!(12 + 4, discussion_meeting.start_hr);
        assert_eq!(0, discussion_meeting.start_min);
        assert_eq!(12 + 4, discussion_meeting.end_hr);
        assert_eq!(50, discussion_meeting.end_min);
        assert_eq!("WLH", discussion_meeting.building);
        assert_eq!("2001", discussion_meeting.room);
        assert!(check_list_eq(&instructors, &discussion_meeting.instructors));

        // Test final
        let final_meeting = cse_101
            .meetings
            .iter()
            .find(|x| x.meeting_type == "FI")
            .expect("No final found, but a final should exist.");
        assert_eq!(
            MeetingDay::OneTime("2022-12-05".into()),
            final_meeting.meeting_days
        );
        assert_eq!(12 + 3, final_meeting.start_hr);
        assert_eq!(0, final_meeting.start_min);
        assert_eq!(12 + 5, final_meeting.end_hr);
        assert_eq!(59, final_meeting.end_min);
        assert!(check_list_eq(&instructors, &final_meeting.instructors));
    }

    #[tokio::test]
    pub async fn test_cogs_118b() {
        let wrapper = get_wrapper();
        let cogs_118b = wrapper
            .get_course_info("COGS", "118B")
            .await
            .expect("COGS 118B is offered in this term.");

        let instructors = ["De Sa, Virginia"].as_slice();
        assert_eq!(6, cogs_118b.len());
        assert!(cogs_118b.iter().all(|x| x.all_instructors == instructors));
        assert!(cogs_118b.iter().all(|x| x.total_seats == 35));
        assert!(cogs_118b
            .iter()
            .any(|x| x.section_id == "087930" && x.section_code == "A01"));
        assert!(cogs_118b
            .iter()
            .any(|x| x.section_id == "087931" && x.section_code == "A02"));
        assert!(cogs_118b
            .iter()
            .any(|x| x.section_id == "087932" && x.section_code == "A03"));
        assert!(cogs_118b
            .iter()
            .any(|x| x.section_id == "087933" && x.section_code == "A04"));
        assert!(cogs_118b
            .iter()
            .any(|x| x.section_id == "087934" && x.section_code == "A05"));
        assert!(cogs_118b
            .iter()
            .any(|x| x.section_id == "087935" && x.section_code == "A06"));

        // Test lectures
        let lectures = get_all_meetings(&cogs_118b, "LE");
        // There should be 6 meetings. All lectures should have same instructors,
        // building, meeting time, etc.
        assert_eq!(6, lectures.len());
        assert!(lectures
            .iter()
            .all(|x| x.building == "MOS" && x.room == "0113"));
        assert!(lectures.iter().all(|x| x.start_hr == 8));
        assert!(lectures.iter().all(|x| x.end_hr == 9));
        assert!(lectures.iter().all(|x| x.start_min == 0));
        assert!(lectures.iter().all(|x| x.end_min == 20));
        assert!(lectures
            .iter()
            .all(|x| x.meeting_days == MeetingDay::Repeated(vec!["Tu".into(), "Th".into()])));
        assert!(lectures.iter().all(|x| x.instructors == instructors));

        // Same idea with finals
        let finals = get_all_meetings(&cogs_118b, "FI");
        assert!(finals
            .iter()
            .all(|x| x.meeting_days == MeetingDay::OneTime("2022-12-06".into())));
        assert!(finals.iter().all(|x| x.start_hr == 8));
        assert!(finals.iter().all(|x| x.start_min == 0));
        assert!(finals.iter().all(|x| x.end_hr == 10));
        assert!(finals.iter().all(|x| x.end_min == 59));
        assert!(finals.iter().all(|x| x.instructors == instructors));

        // Test discussions. Note that this will be
        // slightly more annoying to test.
        let discussions = get_all_meetings(&cogs_118b, "DI");
        assert!(discussions.iter().all(|x| x.instructors == instructors));
        assert!(discussions
            .iter()
            .all(|x| x.meeting_days == MeetingDay::Repeated(vec!["F".into()])));
        assert!(discussions
            .iter()
            .all(|x| x.building == "CSB" && x.room == "115"));

        let test_discussion = |s_hr: i16, s_min: i16, e_hr: i16, e_min: i16| {
            discussions.iter().any(|x| {
                x.start_hr == s_hr && x.start_min == s_min && x.end_hr == e_hr && x.end_min == e_min
            })
        };

        assert!(test_discussion(9, 0, 9, 50));
        assert!(test_discussion(10, 0, 10, 50));
        assert!(test_discussion(11, 0, 11, 50));
        assert!(test_discussion(12, 0, 12, 50));
        assert!(test_discussion(12 + 1, 0, 12 + 1, 50));
        assert!(test_discussion(12 + 2, 0, 12 + 2, 50));
    }

    #[tokio::test]
    pub async fn test_cse_110() {
        let wrapper = get_wrapper();
        let cse_110 = wrapper
            .get_course_info("CSE", "110")
            .await
            .expect("CSE 110 is offered in this term.");

        let instructors = ["Powell, Thomas Allan"].as_slice();
        assert_eq!(8, cse_110.len());

        assert!(cse_110.iter().all(|x| x.all_instructors == instructors));
        assert!(cse_110
            .iter()
            .all(|x| x.total_seats == 50 || x.total_seats == 45));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090047" && x.section_code == "A50"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090048" && x.section_code == "A51"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090049" && x.section_code == "A52"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090050" && x.section_code == "A53"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090051" && x.section_code == "A54"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090052" && x.section_code == "A55"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090053" && x.section_code == "A56"));
        assert!(cse_110
            .iter()
            .any(|x| x.section_id == "090054" && x.section_code == "A57"));
        assert!(cse_110.iter().all(|x| x.meetings.len() == 4));

        // If we have a LE, LA, DI, and FI, then I assume its content is
        // accurate
        assert!(cse_110
            .iter()
            .all(|x| x.meetings.iter().any(|x| x.meeting_type == "LE")
                && x.meetings.iter().any(|x| x.meeting_type == "LA")
                && x.meetings.iter().any(|x| x.meeting_type == "DI")
                && x.meetings.iter().any(|x| x.meeting_type == "FI")));
    }

    #[tokio::test]
    pub async fn test_hidden_sections() {
        let wrapper = get_wrapper();
        let mus_19r = wrapper
            .get_course_info("MUS", "19R")
            .await
            .expect("MUS 19R should be a thing here.");

        // For MUS 19R, section A should all be visible, but section B is all invisible
        assert!(mus_19r
            .iter()
            .filter(|x| &x.section_code[..1] == "A")
            .all(|x| x.is_visible));
        assert!(mus_19r
            .iter()
            .filter(|x| &x.section_code[..1] == "B")
            .all(|x| !x.is_visible));

        // For EDS 124AR, section A+B should all be visible, but section C+D is all invisible
        let eds_124ar = wrapper
            .get_course_info("EDS", "124AR")
            .await
            .expect("EDS 124AR should be a thing here.");

        assert!(eds_124ar
            .iter()
            .filter(|x| ["A", "B"].contains(&&x.section_code[..1]))
            .all(|x| x.is_visible));
        assert!(eds_124ar
            .iter()
            .filter(|x| ["C", "D"].contains(&&x.section_code[..1]))
            .all(|x| !x.is_visible));

        // For EDS 124BR, section A should all be visible, but section B is all invisible
        let eds_124ar = wrapper
            .get_course_info("EDS", "124BR")
            .await
            .expect("EDS 124BR should be a thing here.");

        assert!(eds_124ar
            .iter()
            .filter(|x| &x.section_code[..1] == "A")
            .all(|x| x.is_visible));
        assert!(eds_124ar
            .iter()
            .filter(|x| &x.section_code[..1] == "B")
            .all(|x| !x.is_visible));
    }
}

#[cfg(test)]
mod test_search {
    use std::collections::HashSet;

    use webweg::wrapper::{
        CourseLevelFilter, DayOfWeek, PlanAdd, SearchRequestBuilder, SearchType,
    };

    use crate::get_wrapper;

    #[tokio::test]
    pub async fn test_search_basic() {
        let wrapper = get_wrapper();
        let wcwp_10a_search = wrapper
            .search_courses_detailed(SearchType::BySection("086598"))
            .await
            .expect("This section should exist.");
        assert_eq!(1, wcwp_10a_search.len());
        assert_eq!("086598", wcwp_10a_search[0].section_id);
        assert_eq!("011", wcwp_10a_search[0].section_code);
        assert_eq!("WCWP 10A", wcwp_10a_search[0].subj_course_id);
    }

    #[tokio::test]
    pub async fn test_search_basic_with_name() {
        // Note that CSE 87 (and other seminar classes) have
        // non-digits in the section ID. However, this should not
        // change anything.
        let wrapper = get_wrapper();
        let cse_87_search = wrapper
            .search_courses_detailed(SearchType::BySection("094193"))
            .await
            .expect("This section should exist.");
        assert_eq!(1, cse_87_search.len());
        assert_eq!("094193", cse_87_search[0].section_id);
        assert_eq!("A00", cse_87_search[0].section_code);
        assert_eq!("CSE 87", cse_87_search[0].subj_course_id);
    }

    #[tokio::test]
    pub async fn test_search_mult_sections() {
        let wrapper = get_wrapper();
        let mult_sec = wrapper
            .search_courses_detailed(SearchType::ByMultipleSections(&[
                "089605", "089564", "090936",
            ]))
            .await
            .unwrap();
        assert!(mult_sec.iter().any(|x| x.section_code == "B01"
            && x.section_id == "089605"
            && x.subj_course_id == "MATH 170A"));
        assert!(mult_sec.iter().any(|x| x.section_code == "B02"
            && x.section_id == "089564"
            && x.subj_course_id == "MATH 140A"));
        assert!(mult_sec.iter().any(|x| x.section_code == "A00"
            && x.section_id == "090936"
            && x.subj_course_id == "POLI 100G"));
    }

    #[tokio::test]
    pub async fn test_adv_search() {
        let wrapper = get_wrapper();
        let adv_search = wrapper
            .search_courses_detailed(SearchType::Advanced(
                &SearchRequestBuilder::new()
                    .add_department("CSE")
                    .filter_courses_by(CourseLevelFilter::LowerDivision)
                    .filter_courses_by(CourseLevelFilter::UpperDivision)
                    .apply_day(DayOfWeek::Thursday)
                    .apply_day(DayOfWeek::Tuesday)
                    .set_start_time(11, 0)
                    .set_end_time(12 + 2, 0),
            ))
            .await
            .expect("Something went wrong")
            .into_iter()
            .map(|x| x.subj_course_id)
            .collect::<HashSet<_>>();
        assert!(adv_search.contains("CSE 8A"));
        assert!(adv_search.contains("CSE 30"));
        assert!(adv_search.contains("CSE 106"));
        assert!(adv_search.contains("CSE 142"));
        assert_eq!(5, adv_search.len());
    }

    #[tokio::test]
    pub async fn test_adv_search_mult_departments() {
        let wrapper = get_wrapper();
        let res = wrapper
            .search_courses(SearchType::Advanced(
                &SearchRequestBuilder::new()
                    .filter_courses_by(CourseLevelFilter::LowerDivision)
                    .filter_courses_by(CourseLevelFilter::UpperDivision)
                    .add_department("CSE")
                    .add_department("MATH")
                    .apply_day(DayOfWeek::Monday)
                    .apply_day(DayOfWeek::Wednesday)
                    .apply_day(DayOfWeek::Friday)
                    .set_start_time(10, 0)
                    .set_end_time(12 + 5, 30),
            ))
            .await
            .unwrap();
        assert_eq!(54, res.len());
    }

    #[tokio::test]
    pub async fn test_adv_search_instructor() {
        let wrapper = get_wrapper();
        let res = wrapper
            .search_courses(SearchType::Advanced(
                &SearchRequestBuilder::new().set_instructor("kedlaya"),
            ))
            .await
            .unwrap();
        assert_eq!(3, res.len());
    }

    #[tokio::test]
    pub async fn test_adv_search_title() {
        let wrapper = get_wrapper();
        let res = wrapper
            .search_courses(SearchType::Advanced(
                &SearchRequestBuilder::new().set_title("politics"),
            ))
            .await
            .unwrap();
        assert_eq!(22, res.len());
    }

    // Random test function -- used to debug by printing.
    #[tokio::test]
    async fn test_random() {
        let wrapper = get_wrapper();
        let res = wrapper
            .add_to_plan(
                PlanAdd {
                    subject_code: "MUS",
                    course_code: "19R",
                    section_id: "090571",
                    section_code: "B10",
                    grading_option: None,
                    schedule_name: None,
                    unit_count: 4,
                },
                true,
            )
            .await;
        println!("{:?}", res);
    }
}

#[cfg(test)]
mod util_tests {
    use webweg::util;

    #[test]
    fn test_parse_day_code_simple() {
        assert_eq!(["Su", "M", "W"].as_slice(), &util::parse_day_code("013"));
    }

    #[test]
    fn test_parse_day_code_all() {
        assert_eq!(
            ["Su", "M", "Tu", "W", "Th", "F", "Sa"].as_slice(),
            &util::parse_day_code("0123456")
        );
    }

    #[test]
    fn test_parse_day_code_none() {
        assert!(util::parse_day_code("").is_empty());
    }

    #[test]
    fn test_parse_day_code_out_bounds() {
        assert_eq!(
            ["Su", "F", "M", "Tu"].as_slice(),
            &util::parse_day_code("051928")
        );
    }

    #[test]
    fn test_parse_binary_days_simple() {
        assert_eq!(
            ["M", "W", "F", "Su"].as_slice(),
            &util::parse_binary_days("1010101")
        );
    }

    #[test]
    fn test_parse_binary_days_all() {
        assert_eq!(
            ["M", "Tu", "W", "Th", "F", "Sa", "Su"].as_slice(),
            &util::parse_binary_days("1111111")
        );
    }

    #[test]
    fn test_parse_binary_days_none() {
        assert!(util::parse_binary_days("0000000").is_empty());
    }

    #[test]
    fn test_term_seq_id_base() {
        assert_eq!(5200, util::get_term_seq_id("SP22"));
        assert_eq!(5210, util::get_term_seq_id("S122"));
        assert_eq!(5220, util::get_term_seq_id("S222"));
        assert_eq!(5230, util::get_term_seq_id("S322"));
        assert_eq!(5250, util::get_term_seq_id("FA22"));
        assert_eq!(5260, util::get_term_seq_id("WI23"));
    }

    #[test]
    fn test_term_seq_id_one_year() {
        assert_eq!(5270, util::get_term_seq_id("SP23"));
        assert_eq!(5340, util::get_term_seq_id("SP24"));
        assert_eq!(5330, util::get_term_seq_id("WI24"));
        assert_eq!(5320, util::get_term_seq_id("FA23"));
        assert_eq!(5300, util::get_term_seq_id("S323"));
        assert_eq!(5290, util::get_term_seq_id("S223"));
        assert_eq!(5280, util::get_term_seq_id("S123"));
        // Try using an older term, too
        assert_eq!(5190, util::get_term_seq_id("WI22"));
    }

    #[test]
    fn test_term_seq_id_invalid() {
        // Invalid term
        assert_eq!(0, util::get_term_seq_id("XX24"));
        // Invalid year
        assert_eq!(0, util::get_term_seq_id("WI2T"));
    }

    #[test]
    fn test_format_course_code() {
        assert_eq!("  8B", util::get_formatted_course_num("8B"));
        assert_eq!("  1", util::get_formatted_course_num("1"));
        assert_eq!(" 15L", util::get_formatted_course_num("15L"));
        assert_eq!(" 12", util::get_formatted_course_num("12"));
        assert_eq!("158R", util::get_formatted_course_num("158R"));
        assert_eq!("101", util::get_formatted_course_num("101"));
        assert_eq!("MATH", util::get_formatted_course_num("MATH"));
    }

    #[test]
    fn test_format_multiple_courses_full() {
        assert_eq!(
            "CSE:  8B;CSE: 95;MATH:100A",
            util::format_multiple_courses(&["CSE 8B", "CSE 95", "MATH 100A"])
        );
        assert_eq!(
            "CSE:101;MATH:170A;MATH: 20D;MATH:187A;CSE: 11;POLI:102D;POLI:112A;COGS:  9",
            util::format_multiple_courses(&[
                "CSE 101",
                "MATH 170A",
                "math 20d",
                "MATH 187A",
                "cse 11",
                "POLI 102D",
                "poli 112a",
                "cogs 9"
            ])
        );
        assert_eq!(
            "CSE:101;MATH: 20D;MATH:187A;CSE: 11;POLI:102D;POLI:112A;COGS:  9",
            util::format_multiple_courses(&[
                "CSE 101",
                "math20d",
                "MATH187A",
                "cse 11",
                "POLI102D",
                "poli 112a",
                "cogs9"
            ])
        )
    }

    #[test]
    fn test_format_multiple_courses_subj() {
        assert_eq!(
            "CSE;CSE;MATH",
            util::format_multiple_courses(&["CSE", "CSE", "MATH"])
        );
        assert_eq!(
            "COGS;CSE;MATH;POLI;HIST",
            util::format_multiple_courses(&["cogs", "CSE", "Math", "Poli", "hist"])
        );
    }

    #[test]
    fn test_format_multiple_courses_num() {
        assert_eq!(
            "105;101; 30;108;  8A;  5",
            util::format_multiple_courses(&["105", "101", "30", "108", "8A", "5"])
        );
        assert_eq!(
            " 95;  1;  8B;190A;101; 15L;105;171; 30",
            util::format_multiple_courses(&[
                "95", "1", "8B", "190A", "101", "15L", "105", "171", "30"
            ])
        );
    }

    #[test]
    fn test_format_multiple_courses_mixed() {
        assert_eq!(
            "CSE:101;105;COGS: 10;  8",
            util::format_multiple_courses(&["cse 101", "105", "cogs 10", "8"])
        );
        assert_eq!(
            "MATH: 20;CSE: 95;COGS:100;MATH: 10",
            util::format_multiple_courses(&["math 20", "cse95", "cogs100", "math10"])
        )
    }
}
