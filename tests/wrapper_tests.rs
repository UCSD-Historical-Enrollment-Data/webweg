extern crate core;

use reqwest::Client;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time;
use webweg::webreg_clean_defn::MeetingDay;
use webweg::webreg_wrapper::{
    CourseLevelFilter, DayOfWeek, GradeOption, SearchRequestBuilder, SearchType, WebRegWrapper,
};

const TERM: &str = "SP22";

/// Gets the cookies needed to access WebReg.
fn get_cookie_str() -> String {
    use std::fs;
    use std::path::Path;

    let file = Path::new("cookie.txt");
    if !file.exists() {
        panic!("File 'cookie.txt does not exist.");
    }

    fs::read_to_string(file).unwrap_or_else(|_| "".to_string())
}

/// This function tests the `get_course_info()` method. Note that only one test is done for now.
///
/// This also implicitly tests different input types (e.g. instead of `MATH 10B`, you might do
/// `math 10b`.
#[tokio::test]
async fn test_get_course_info() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    let math_155a = wrapper.get_course_info("math", "155a").await;

    // Math 200C is a valid section.
    assert!(math_155a.is_ok());

    let math_155a = math_155a.unwrap();
    // There are 2 sections of Math 155A: Section A01 and Section A02
    assert_eq!(3, math_155a.len());
    assert_eq!(
        vec!["A01".to_string(), "A02".to_string(), "A03".to_string()],
        math_155a
            .iter()
            .map(|x| x.section_code.as_str())
            .collect::<Vec<_>>()
    );
    // Each section has 36 seats.
    assert_eq!(36, math_155a[0].total_seats);
    assert_eq!(36, math_155a[1].total_seats);
    // The professor teaching it is Sam, Steven V.
    assert_eq!(vec!["Buss, Samuel R".to_string()], math_155a[0].all_instructors);
    // There are five meetings -- a lecture, discussion, final, and 2 review sessions
    assert_eq!(5, math_155a[0].meetings.len());

    // Test the second section.
    let lecture = math_155a[1]
        .meetings
        .iter()
        .find(|x| x.meeting_type == "LE");
    let discussion = math_155a[1]
        .meetings
        .iter()
        .find(|x| x.meeting_type == "DI");
    let fin = math_155a[1]
        .meetings
        .iter()
        .find(|x| x.meeting_type == "FI");
    assert!(lecture.is_some());
    assert!(fin.is_some());
    assert!(discussion.is_some());

    // Test lecture: TuTh from 3:30p-4:50p at CENTR 115
    let lecture = lecture.unwrap();
    let lec_time = match lecture.meeting_days {
        MeetingDay::Repeated(ref r) => r,
        MeetingDay::OneTime(_) => panic!("This is not a one-time meeting."),
        MeetingDay::None => panic!("There should be a meeting."),
    };
    assert_eq!(&vec!["Tu".to_string(), "Th".to_string()], lec_time);
    assert_eq!(12 + 3, lecture.start_hr);
    assert_eq!(30, lecture.start_min);
    assert_eq!(12 + 4, lecture.end_hr);
    assert_eq!(50, lecture.end_min);
    assert_eq!("CENTR", lecture.building);
    assert_eq!("115", lecture.room);

    // Test discussion: W from 7:00p-7:50p at APM 5402
    let discussion = discussion.unwrap();
    let dis_time = match discussion.meeting_days {
        MeetingDay::Repeated(ref r) => r,
        MeetingDay::OneTime(_) => panic!("This is not a one-time meeting."),
        MeetingDay::None => panic!("There should be a meeting."),
    };
    assert_eq!(&vec!["W".to_string()], dis_time);
    assert_eq!(12 + 7, discussion.start_hr);
    assert_eq!(0, discussion.start_min);
    assert_eq!(12 + 7, discussion.end_hr);
    assert_eq!(50, discussion.end_min);
    assert_eq!("APM", discussion.building);
    assert_eq!("5402", discussion.room);

    // Test final: 06/06/2022 3:00p-5:59p
    let fin = fin.unwrap();
    let fin_time = match fin.meeting_days {
        MeetingDay::Repeated(_) => panic!("This is not a repeated meeting."),
        MeetingDay::OneTime(ref r) => r,
        MeetingDay::None => panic!("There should be a meeting."),
    };

    assert_eq!("2022-06-06", fin_time);
    assert_eq!(12 + 3, fin.start_hr);
    assert_eq!(0, fin.start_min);
    assert_eq!(12 + 5, fin.end_hr);
    assert_eq!(59, fin.end_min);
}

/// This function tests the `get_course_info()` function, but specifically for courses which have
/// multiple instructors, e.g. "Instructor Name" and "Staff"
#[tokio::test]
async fn test_instructor() {
    // Literally hours after I made this test, WebReg removed "Staff" from the courses I selected.
    // Nice.
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    let cse_130 = wrapper.get_course_info("cse", "130").await;
    assert!(cse_130.is_ok());
    let cse_130 = cse_130.unwrap();
    assert_eq!(1, cse_130.len());
    assert_eq!(
        vec!["Polikarpova, Nadezhda".to_string()],
        cse_130[0].all_instructors
    );

    let cse_100 = wrapper.get_course_info("cse", "100").await;
    assert!(cse_100.is_ok());
    let cse_100 = cse_100.unwrap();
    assert_eq!(3, cse_100.len());
    // Test both sections of 100
    assert_eq!(vec!["Sahoo, Debashis".to_string()], cse_100[0].all_instructors);
    assert_eq!(vec!["Cao, Yingjun".to_string()], cse_100[1].all_instructors);
    assert_eq!(vec!["Cao, Yingjun".to_string()], cse_100[2].all_instructors);
}

/// This function tests the `search_courses_detailed()` method with one section.
#[tokio::test]
async fn test_search_one_sec() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    // Search for 1 section: Math 184 (078615)
    let math_184 = wrapper
        .search_courses_detailed(SearchType::BySection("078615"))
        .await;
    // This section exists without error
    assert!(math_184.is_ok());
    let math_184 = math_184.unwrap();
    // There should only be one section since we only searched for one.
    assert_eq!(1, math_184.len());
    // The section ID is trivial
    assert_eq!("078615", math_184[0].section_id);
    // As is the subject + course number
    assert_eq!("MATH 184", math_184[0].subj_course_id);
    // The instructor is Kane, Daniel Mertz
    assert_eq!(
        vec!["Kane, Daniel Mertz".to_string()],
        math_184[0].all_instructors
    );
    // This is section A02
    assert_eq!("A02", math_184[0].section_code);

    // Search for 1 section: Math 184 (78615) [note no leading 0]
    let math_184 = wrapper
        .search_courses_detailed(SearchType::BySection("78615"))
        .await;
    // This section exists
    assert!(math_184.is_ok());
    let math_184 = math_184.unwrap();
    // There should only be one section since we only searched for one.
    assert_eq!(1, math_184.len());
    // The section ID is trivial
    assert_eq!("078615", math_184[0].section_id);
}

/// This function tests the `search_courses_detailed()` method with multiple sections.
#[tokio::test]
async fn test_search_mult_sec() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    // Search for 3 sections:
    // - CSE 110 (077443) taught by Politz, Joseph Gibbs
    // - Math 180A (085554) taught by Kolesnik, Brett T
    // - LIGN 101 (074271) taught by Styler, William Francis
    let mult_sec = wrapper
        .search_courses_detailed(SearchType::ByMultipleSections(&[
            "077443", "085554", "074271",
        ]))
        .await;
    // No error should have occurred
    assert!(mult_sec.is_ok());
    let mult_sec = mult_sec.unwrap();
    let cse_110 = mult_sec.iter().find(|x| x.section_id == "077443");
    let math_180a = mult_sec.iter().find(|x| x.section_id == "085554");
    let lign_101 = mult_sec.iter().find(|x| x.section_id == "074271");
    // Each of these should exist
    assert!(cse_110.is_some());
    assert!(math_180a.is_some());
    assert!(lign_101.is_some());
    let cse_110 = cse_110.unwrap();
    let math_180a = math_180a.unwrap();
    let lign_101 = lign_101.unwrap();
    // Start with CSE 110.
    assert_eq!("CSE 110", cse_110.subj_course_id);
    assert_eq!(
        vec!["Politz, Joseph Gibbs".to_string()],
        cse_110.all_instructors
    );
    assert_eq!("A51", cse_110.section_code);

    // Next is Math 180A
    assert_eq!("MATH 180A", math_180a.subj_course_id);
    assert_eq!(vec!["Kolesnik, Brett T".to_string()], math_180a.all_instructors);
    assert_eq!("A06", math_180a.section_code);

    // Last is LIGN 101
    assert_eq!("LIGN 101", lign_101.subj_course_id);
    assert_eq!(
        vec!["Styler, William Francis".to_string()],
        lign_101.all_instructors
    );
    assert_eq!("A01", lign_101.section_code);
}

/// This function tests the `search_courses_detailed()` method with advanced search features.
#[tokio::test]
async fn test_adv_search() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    // Filter all courses by:
    // - Must be in subject 'CSE'
    // - Must be lower- and upper-division courses
    // - Must only be on M, W, and Fs
    // - Must start no earlier than 10:00am and end no later than 6:00pm
    // There should be 15 distinct results. Note that this will make a lot of requests!
    let search_res = wrapper
        .search_courses_detailed(SearchType::Advanced(
            &SearchRequestBuilder::new()
                .add_subject("CSE")
                .filter_courses_by(CourseLevelFilter::LowerDivision)
                .filter_courses_by(CourseLevelFilter::UpperDivision)
                .apply_days(DayOfWeek::Monday)
                .apply_days(DayOfWeek::Wednesday)
                .apply_days(DayOfWeek::Friday)
                .set_start_time(10, 0)
                .set_end_time(12 + 6, 0),
        ))
        .await;

    // Should have no errors
    assert!(search_res.is_ok());
    let search_res = search_res.unwrap();
    // Collect into a set since we will have multiple of the same sections
    let all_courses = search_res
        .iter()
        .map(|x| x.subj_course_id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(15, all_courses.len());
    assert!(all_courses.iter().all(|x| x.starts_with("CSE")));
}

/// This function tests very basic schedule stuff. Due to how much can change with the schedule,
/// very minimal testing will be done; instead, numerous things will be printed, of which the user
/// will need to manually check for themselves.
#[tokio::test]
async fn test_get_schedule() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    let schedule = wrapper.get_schedule(None).await;
    // If you don't have a schedule, that's kind of a problem.
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap();

    // Test instructor display
    for s in &schedule {
        assert_eq!(
            HashSet::<&String>::from_iter(s.all_instructors.iter()).len(),
            s.all_instructors.len()
        );
    }

    for s in schedule {
        println!("{}", s.to_string());
    }
}

/// This function tests changing the grading options. Note that it's not easy to
/// programmatically validate that the functions work as expected.
#[tokio::test]
#[ignore = "Don't need to spam WebReg here."]
async fn test_change_grade_options() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    // Try to change grading option to P for a class that we're enrolled in.
    let test1 = wrapper.change_grading_option("84932", GradeOption::P).await;
    assert!(test1.is_ok());
    time::sleep(Duration::from_secs(2)).await;

    // Same class again, but with one zero.
    let test2 = wrapper
        .change_grading_option("00084932", GradeOption::L)
        .await;
    assert!(test2.is_ok());
    time::sleep(Duration::from_secs(2)).await;

    // Try a different class that we aren't enrolled in.
    let test3 = wrapper
        .change_grading_option("079911", GradeOption::P)
        .await;
    assert!(test3.is_err());
    time::sleep(Duration::from_secs(2)).await;

    // Same class as test 1 & 2, but this time we're trying to change to S/U grading (which is
    // not allowed).
    let test4 = wrapper
        .change_grading_option("084932", GradeOption::S)
        .await;
    assert!(test4.is_err());
    time::sleep(Duration::from_secs(2)).await;

    // Same class as test 1 & 2, just changing back to letter.
    let test5 = wrapper
        .change_grading_option("084932", GradeOption::L)
        .await;
    assert!(test5.is_ok());
}

/// This function tests sections where there may be multiple instructors assigned to a non-lecture
/// time.
#[tokio::test]
async fn test_other_instructors() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);

    let cse_30 = wrapper.get_course_info("cse", "30").await;
    assert!(cse_30.is_ok());
    let cse_30 = cse_30.unwrap();
    assert_eq!(2, cse_30.len());
    let cse_30_inst = vec!["Muller, P Keith".to_string()];
    assert!(cse_30.iter().all(|x| x.all_instructors == cse_30_inst));


    let psyc_194c = wrapper.get_course_info("psyc", "194c").await;
    assert!(psyc_194c.is_ok());
    let psyc_194c = psyc_194c.unwrap();
    assert_eq!(22, psyc_194c.len());
}


/// Literally just designed for me to test random things with the wrapper.
#[tokio::test]
async fn test_random_stuff() {
    let wrapper = WebRegWrapper::new(Client::new(), get_cookie_str(), TERM);
    assert!(wrapper.is_valid().await);
    let d = wrapper.get_course_info("MATH", "18").await.unwrap();
    println!("{}", d.len());

    d.into_iter().for_each(|x| {
        println!("{}", x.to_string());
    });

    // Panic so we can see standard output.
    panic!();
}