use webweg::wrapper::WebRegWrapper;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapper::builder()
        .with_cookies("style=default; _saml_idp=dXJuOm1hY2U6aW5jb21tb246dWNzZC5lZHU%3D; jlinkauthserver=davis; itscookie=!zNVelLCYxKMxTkMu+zn5WhihAIiiuyriU6yiB++LiML1G0pXB/6Z+zqWAArRVW28Abq2nNr8h8BIu9s=; SID=17007652135828; SLID=zbf91222898a5556df8df3c6d8c521a9b; jlinksessionidx=z90d45d941f4f93701fb8c3b2b5e0191b; jlinkappx=/webreg2/svc/wradapter/secure/search-load-group-data; _shibsession_64656661756c7468747470733a2f2f6163742e756373642e6564752f73686962626f6c6574682d7370=_27c127c3c10d09c37ee9f7e53fcd2483; xjlinkloginStudent.Transactional=OK; TS01111c3f=01f0fc640d18ab864540ad3b893d40a4e74c70dbb46443e9e67dac2597554d5502311b02515513205e324ab86b1ac47561cbd4260ce46cf14b17b19d90b595c0d1be550ea6dc002d56f376fa64333529e8a14b7d507dbdd57d60b138657c8d5ef6b4974fe34c1edb612b5d50ffb2d9c06ac6650da9a14d23068ae8a0e3d53792a041cfb1749fca7abf4d7d97bf7e545b839c0ae14329156e0b2635e8247c913166bd43d39b1ba7b330abeffbc740da4e1ae37a314626141467cbc2ddc03db8cf746b8559ca")
        .try_build_wrapper()
        .unwrap();

    // Registers all active terms so we can switch between active quarters.
    _ = wrapper.register_all_terms().await;

    // Let's get all CSE 100 courses for FA23
    let cse100_fa23 = wrapper
        .req("WI24")
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

    // You can also register terms that are probably hidden (it's available on WebReg,
    // but is hidden from the get_term API endpoint)
    _ = wrapper.associate_term("WI24").await;

    // Of course, we can get more than just course info when switching quarters.
    let cse_course_notes = wrapper
        .req("WI24")
        .parsed()
        .get_section_notes_by_course("CSE", "290")
        .await
        .unwrap();
    println!("{cse_course_notes:?}");
}
