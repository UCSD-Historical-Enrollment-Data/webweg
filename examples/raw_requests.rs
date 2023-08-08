use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapperBuilder::new()
        .with_cookies("my cookies here")
        .with_default_term("FA23")
        .try_build_wrapper()
        .unwrap();

    // Registers all active terms so we can switch between active quarters.
    _ = wrapper.associate_term("FA23").await;

    // `default_request` will by default give you a "parsed" response. A parsed response
    // is one that takes the original WebReg response and "cleans" it and then returns
    // the cleaned version.
    let my_schedule = wrapper.default_request().get_schedule(None).await;
    println!("{my_schedule:?}");

    // The above few lines are equivalent to...
    let my_schedule = wrapper
        .make_request()
        .build_term_parser()
        .get_schedule(None)
        .await;

    println!("{my_schedule:?}");

    // But let's say you want a raw response (not a cleaned one from WebReg)...
    let my_raw_schedule = wrapper
        .make_request()
        .build_term_raw()
        .get_schedule(None)
        .await;

    println!("{my_raw_schedule:?}");
}
