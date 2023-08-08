use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapperBuilder::new()
        .with_cookies("my cookies here")
        .with_default_term("FA23")
        .try_build_wrapper()
        .unwrap();

    // Registers all active terms so we can switch between active quarters.
    _ = wrapper.register_all_terms().await;

    // Using `default_request` defaults to the default term (FA23)
    let cse100_fa23 = wrapper
        .default_request()
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
        .make_request()
        .override_term("S223")
        .build_term_parser()
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
