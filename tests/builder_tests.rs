use reqwest::Client;
use webweg::wrapper::input_types::{DayOfWeek, EnrollWaitAdd, EventAdd, GradeOption, PlanAdd};
use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;

#[test]
fn fail_construct_wrapper() {
    let wrapper = WebRegWrapperBuilder::new()
        .with_client(Client::new())
        .with_default_term("FA23")
        .try_build_wrapper();
    assert!(wrapper.is_none());
}

#[test]
fn success_construct_wrapper() {
    let wrapper = WebRegWrapperBuilder::new()
        .with_cookies("abc")
        .with_default_term("FA23")
        .try_build_wrapper();
    assert!(wrapper.is_some());
}

#[test]
fn success_construct_plan_add() {
    let plan_add = PlanAdd::builder()
        .with_section_id("my section id")
        .with_unit_count(41)
        .with_grading_option(GradeOption::P)
        .with_subject_code("CSE")
        .with_course_code("291")
        .with_schedule_name("a schedule")
        .with_section_code("A0000")
        .try_build()
        .unwrap();

    assert_eq!(plan_add.section_id, "my section id");
    assert_eq!(plan_add.unit_count, 41);
    assert!(matches!(plan_add.grading_option, Some(GradeOption::P)));
    assert_eq!(plan_add.subject_code, "CSE");
    assert_eq!(plan_add.course_code, "291");
    assert_eq!(plan_add.schedule_name.unwrap(), "a schedule");
    assert_eq!(plan_add.section_code, "A0000");
}

#[test]
fn fail_construct_plan_add() {
    // No unit count
    let plan_add = PlanAdd::builder()
        .with_section_id("my section id")
        .with_grading_option(GradeOption::P)
        .with_subject_code("CSE")
        .with_course_code("291")
        .with_schedule_name("a schedule")
        .with_section_code("A0000")
        .try_build();
    assert!(plan_add.is_none());
}

#[test]
fn success_construct_event_add() {
    let event_add = EventAdd::builder()
        .with_name("kira is bad")
        .with_location("and ruby is too")
        .with_day(DayOfWeek::Monday)
        .with_day(DayOfWeek::Wednesday)
        .with_day(DayOfWeek::Friday)
        .with_start_time(10, 0)
        .with_end_time(12 + 5, 59)
        .try_build()
        .unwrap();

    assert_eq!(event_add.event_name, "kira is bad");
    assert_eq!(event_add.location.unwrap(), "and ruby is too");
    assert_eq!(
        event_add.event_days,
        vec![DayOfWeek::Monday, DayOfWeek::Wednesday, DayOfWeek::Friday]
    );
    assert_eq!(event_add.start_hr, 10);
    assert_eq!(event_add.start_min, 0);
    assert_eq!(event_add.end_hr, 17);
    assert_eq!(event_add.end_min, 59);
}

#[test]
fn fail_construct_event_add() {
    // With invalid end time (60 > 59)
    let event_add = EventAdd::builder()
        .with_name("asdgwaesgd")
        .with_location("asdsdasdasd")
        .with_day(DayOfWeek::Monday)
        .with_start_time(10, 0)
        .with_end_time(12 + 5, 60)
        .try_build();

    assert!(event_add.is_none());
}

#[test]
fn success_construct_enroll_wait() {
    let enroll_add = EnrollWaitAdd::builder()
        .with_section_id("12345")
        .with_grading_option(GradeOption::P)
        .with_unit_count(12)
        .try_build()
        .unwrap();

    assert!(matches!(enroll_add.grading_option, Some(GradeOption::P)));
    assert_eq!(enroll_add.unit_count, Some(12));
    assert_eq!(enroll_add.section_id, "12345");
}

#[test]
fn fail_construct_enroll_wait() {
    let enroll_add = EnrollWaitAdd::builder()
        .with_grading_option(GradeOption::P)
        .with_unit_count(12)
        .try_build();

    assert!(enroll_add.is_none());
}
