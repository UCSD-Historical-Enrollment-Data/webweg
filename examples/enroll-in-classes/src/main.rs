use webweg::wrapper::input_types::{AddType, EnrollWaitAdd, GradeOption};
use webweg::wrapper::WebRegWrapper;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapper::builder()
        .with_cookies("my cookies here")
        .try_build_wrapper()
        .unwrap();

    // Essentially registers the FA23 term with our session
    _ = wrapper.associate_term("FA23");

    let course_to_enroll = EnrollWaitAdd::builder()
        .with_section_id("123456")
        .with_grading_option(GradeOption::P)
        .try_build()
        .unwrap();

    let enroll_result = wrapper
        .req("FA23")
        .parsed()
        .add_section(AddType::DecideForMe, course_to_enroll, true)
        .await;

    match enroll_result {
        Ok(res) => {
            println!("Enrolled in class? {res}");
        }
        Err(e) => {
            println!("Got an error when trying to enroll: {e}")
        }
    }
}
