use webweg::wrapper::WebRegWrapper;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let wrapper = WebRegWrapper::builder()
        .with_cookies("JSESSIONID=9A76CD97B28CA003AC07875019D33C3E; TS01755bce=01f0fc640d71b016e616ef3b18639307afa81fbf097f788144140800bd1c5de34a347ae1936a9129d30e35e793147f1c94eb8b17ee00f2147c3f98a347e443d77c3e77f7559e1364e5cf4c68c88ab4ffdfc758edfe284483ad318ead750db0a73e786089d4a4e56639e56643fb93efbbaed5331c100518125a5612b439f47ceceb490122f0e5dfeaa8b230da51e0ea770215eac1c6b94ff28b03acc53e2fd1f2c1e91d9c71e8b02a86fb3891d81dfe17b5400988de861c84616481eff9b3d8765f819d25683fdba97078abe7235cd55e4b003f0a469c9b6e73d08c1fff9b1ae2e4b194bdb64b0b1386c3610dfcff5155fc99b76984347500ba5636ebdf14384c0b1e30d7e1; style=default; _saml_idp=dXJuOm1hY2U6aW5jb21tb246dWNzZC5lZHU%3D; itscookie=!PeXJOTn3/og31Vsu+zn5WhihAIiiuz8CPWGzWwE3UKxIIDGcwJHD7yy8jQnECJy0kJ+uk+Wjrt1rLlo=; stuno=A16666958; serverTime=1700426334840; sessionExpiry=1700433534840; clientTimeOffset=189; jlinkauthserver=cupples; jlinksessionidx=z21e8eec61dfb86f52b8a6b63228fcb67; jlinkappx=/webreg2/start; _shibsession_64656661756c7468747470733a2f2f6163742e756373642e6564752f73686962626f6c6574682d7370=_d1d71356b4aca12bb3aa998cfca38d85; SID=17004295161938; SLID=za63678c8014f67ab624f34d141729f3f; xjlinkloginStudent.Transactional=OK; TS01111c3f=01f0fc640d72d8b98fef1984f829a07c9ed7278a40885a6e2aed4876c79c208d6ba45d0c5651df81046817fb05dd0b4415e7c5760d6fb2f62ba9f60a245b6cb9301d2f96a189753983d8009f03a0744781ec25a5db9d47ab7694eab3744b35db2c6617847a92c837ad10d9d8df3dd49f27245b8523c122c2d001667627f2459d32f86f14c6f84d97655af87bb0f1b7aadda1ed79474ab18b36eef082334fba3f102d64e7e543bc0371e4fb541e41b227d2c71bbd52bd3976eb74cf22458570c7bf5040a919ab8408a26929cb547db037cc7c8697bfd767c19e9cb9aa79ba30c2166c3989afac808d2c735bc36c30a2ae9c76dcbb64")
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

    let cse_course_notes = wrapper
        .req("WI24")
        .parsed()
        .get_course_notes(&["CSE"])
        .await
        .unwrap();
    println!("{}", cse_course_notes.get("CSE 101").unwrap());
}
