use webweg::wrapper::WebRegWrapper;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapper::builder()
        .with_cookies("my cookies here")
        .try_build_wrapper()
        .unwrap();

    // Registers all active terms so we can switch between active quarters.
    _ = wrapper.associate_term("FA23").await;

    // `default_request` will by default give you a "parsed" response. A parsed response
    // is one that takes the original WebReg response and "cleans" it and then returns
    // the cleaned version.
    let my_schedule = wrapper.req("FA23").parsed().get_schedule(None).await;
    println!("{my_schedule:?}");

    // The above few lines are equivalent to...
    let my_schedule = wrapper.req("FA23").parsed().get_schedule(None).await;

    println!("{my_schedule:?}");

    // But let's say you want a raw response (not a cleaned one from WebReg)...
    let my_raw_schedule = wrapper.req("FA23").raw().get_schedule(None).await;

    println!("{my_raw_schedule:?}");
}
