extern crate core;

use reqwest::Client;
use std::collections::HashSet;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use webweg::types::{CourseSection, Meeting};
use webweg::wrapper::wrapper_builder::WebRegWrapperBuilder;
use webweg::wrapper::WebRegWrapper;

async fn test() {
    let wrapper = WebRegWrapperBuilder::new()
        .with_cookies("your cookies here")
        .with_default_term("FA23")
        .try_build_wrapper()
        .unwrap();

    let my_default_schedule = wrapper.default_request().get_schedule(None).await;

    if let Ok(s) = my_default_schedule {
        s.iter().for_each(|sec| println!("{sec:?}"));
    }
}

#[cfg(test)]
mod util_tests {
    use webweg::util;

    #[test]
    fn test_parse_day_code_simple() {
        assert_eq!(["Su", "M", "W"].as_slice(), &util::parse_day_code("013"));
    }

    #[test]
    fn test_parse_day_code_all() {
        assert_eq!(
            ["Su", "M", "Tu", "W", "Th", "F", "Sa"].as_slice(),
            &util::parse_day_code("0123456")
        );
    }

    #[test]
    fn test_parse_day_code_none() {
        assert!(util::parse_day_code("").is_empty());
    }

    #[test]
    fn test_parse_day_code_out_bounds() {
        assert_eq!(
            ["Su", "F", "M", "Tu"].as_slice(),
            &util::parse_day_code("051928")
        );
    }

    #[test]
    fn test_parse_binary_days_simple() {
        assert_eq!(
            ["M", "W", "F", "Su"].as_slice(),
            &util::parse_binary_days("1010101")
        );
    }

    #[test]
    fn test_parse_binary_days_all() {
        assert_eq!(
            ["M", "Tu", "W", "Th", "F", "Sa", "Su"].as_slice(),
            &util::parse_binary_days("1111111")
        );
    }

    #[test]
    fn test_parse_binary_days_none() {
        assert!(util::parse_binary_days("0000000").is_empty());
    }

    #[test]
    fn test_term_seq_id_base() {
        assert_eq!(5200, util::get_term_seq_id("SP22"));
        assert_eq!(5210, util::get_term_seq_id("S122"));
        assert_eq!(5220, util::get_term_seq_id("S222"));
        assert_eq!(5230, util::get_term_seq_id("S322"));
        assert_eq!(5250, util::get_term_seq_id("FA22"));
        assert_eq!(5260, util::get_term_seq_id("WI23"));
    }

    #[test]
    fn test_term_seq_id_one_year() {
        assert_eq!(5270, util::get_term_seq_id("SP23"));
        assert_eq!(5340, util::get_term_seq_id("SP24"));
        assert_eq!(5330, util::get_term_seq_id("WI24"));
        assert_eq!(5320, util::get_term_seq_id("FA23"));
        assert_eq!(5300, util::get_term_seq_id("S323"));
        assert_eq!(5290, util::get_term_seq_id("S223"));
        assert_eq!(5280, util::get_term_seq_id("S123"));
        // Try using an older term, too
        assert_eq!(5190, util::get_term_seq_id("WI22"));
    }

    #[test]
    fn test_term_seq_id_invalid() {
        // Invalid term
        assert_eq!(0, util::get_term_seq_id("XX24"));
        // Invalid year
        assert_eq!(0, util::get_term_seq_id("WI2T"));
    }

    #[test]
    fn test_format_course_code() {
        assert_eq!("  8B", util::get_formatted_course_num("8B"));
        assert_eq!("  1", util::get_formatted_course_num("1"));
        assert_eq!(" 15L", util::get_formatted_course_num("15L"));
        assert_eq!(" 12", util::get_formatted_course_num("12"));
        assert_eq!("158R", util::get_formatted_course_num("158R"));
        assert_eq!("101", util::get_formatted_course_num("101"));
        assert_eq!("MATH", util::get_formatted_course_num("MATH"));
    }

    #[test]
    fn test_format_multiple_courses_full() {
        assert_eq!(
            "CSE:  8B;CSE: 95;MATH:100A",
            util::format_multiple_courses(&["CSE 8B", "CSE 95", "MATH 100A"])
        );
        assert_eq!(
            "CSE:101;MATH:170A;MATH: 20D;MATH:187A;CSE: 11;POLI:102D;POLI:112A;COGS:  9",
            util::format_multiple_courses(&[
                "CSE 101",
                "MATH 170A",
                "math 20d",
                "MATH 187A",
                "cse 11",
                "POLI 102D",
                "poli 112a",
                "cogs 9"
            ])
        );
        assert_eq!(
            "CSE:101;MATH: 20D;MATH:187A;CSE: 11;POLI:102D;POLI:112A;COGS:  9",
            util::format_multiple_courses(&[
                "CSE 101",
                "math20d",
                "MATH187A",
                "cse 11",
                "POLI102D",
                "poli 112a",
                "cogs9"
            ])
        )
    }

    #[test]
    fn test_format_multiple_courses_subj() {
        assert_eq!(
            "CSE;CSE;MATH",
            util::format_multiple_courses(&["CSE", "CSE", "MATH"])
        );
        assert_eq!(
            "COGS;CSE;MATH;POLI;HIST",
            util::format_multiple_courses(&["cogs", "CSE", "Math", "Poli", "hist"])
        );
    }

    #[test]
    fn test_format_multiple_courses_num() {
        assert_eq!(
            "105;101; 30;108;  8A;  5",
            util::format_multiple_courses(&["105", "101", "30", "108", "8A", "5"])
        );
        assert_eq!(
            " 95;  1;  8B;190A;101; 15L;105;171; 30",
            util::format_multiple_courses(&[
                "95", "1", "8B", "190A", "101", "15L", "105", "171", "30"
            ])
        );
    }

    #[test]
    fn test_format_multiple_courses_mixed() {
        assert_eq!("", util::format_multiple_courses([].as_slice() as &[&str]));
        assert_eq!(
            "  8A;CSE: 12",
            util::format_multiple_courses(&["8a", "", "cse12"])
        );
        assert_eq!(
            "CSE:101;105;COGS: 10;  8",
            util::format_multiple_courses(&["cse 101", "105", "cogs 10", "8"])
        );
        assert_eq!(
            "MATH: 20;CSE: 95;COGS:100;MATH: 10",
            util::format_multiple_courses(&["math 20", "cse95", "cogs100", "math10"])
        )
    }
}
