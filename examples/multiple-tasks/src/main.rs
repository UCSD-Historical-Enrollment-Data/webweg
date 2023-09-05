use std::sync::Arc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use webweg::wrapper::WebRegWrapper;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = Arc::new(WebRegWrapper::builder()
        .with_cookies("my cookies here")
        .try_build_wrapper()
        .unwrap());

    const TERMS: [&str; 3] = ["FA23", "S223", "S323"];
    let mut tasks = FuturesUnordered::new();
    for term in TERMS {
        let cloned = wrapper.clone();
        tasks.push(tokio::spawn(async move {
            cloned.set_cookies(format!("pretend I have cookies for {}", term));
            let data = cloned.req(term).parsed().get_course_info("CSE", "100").await;
            println!("{data:?}");
        }));
    }

    while let Some(_) = tasks.next().await {
        println!("done!");
    }
}
