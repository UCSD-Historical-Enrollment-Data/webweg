use webweg::wrapper::WebRegWrapper;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapper::builder()
        .with_cookies("my cookies here")
        .try_build_wrapper()
        .unwrap();

    // Registers all active terms so we can switch between active quarters.
    _ = wrapper.register_all_terms().await;

    // Let's get all CSE 100 courses for FA23
    let cse100_fa23 = wrapper
        .req("FA23")
        .parsed()
        .get_course_info("CSE", "100")
        .await;

    match cse100_fa23 {
        Ok(courses) => {
            for course in courses {
                println!("{course}")
            }
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }

    // But we can also switch to another active quarter
    let cse100_s223 = wrapper
        .req("S223")
        .parsed()
        .get_course_info("CSE", "100")
        .await;

    match cse100_s223 {
        Ok(courses) => {
            for course in courses {
                println!("{course}")
            }
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }
}
