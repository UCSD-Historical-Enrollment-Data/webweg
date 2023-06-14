use reqwest::Client;
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
