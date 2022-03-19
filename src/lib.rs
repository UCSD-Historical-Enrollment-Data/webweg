//! An asynchronous API wrapper for UCSD's [WebReg](https://act.ucsd.edu/webreg2/start) enrollment
//! system.
//!
//! At a high level, webweg is designed to do the following:
//! - Search for courses based on many different specifications (like WebReg's advanced search).
//! - Get information about a particular course (e.g. number of seats left).
//! - Enroll, plan, or waitlist courses.
//! - And many more.

mod webreg_helper;
mod webreg_raw_defn;

pub mod webreg_clean_defn;
pub mod webreg_wrapper;