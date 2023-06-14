use std::fmt::{Display, Formatter};

use serde::Serialize;
use thiserror::Error;

use crate::wrapper::search::DayOfWeek;

/// A section, which consists of a lecture, usually a discussion, and usually a final.
#[derive(Debug, Clone, Serialize)]
pub struct CourseSection {
    /// The subject, course ID. For example, `CSE 100`.
    pub subj_course_id: String,
    /// The section ID. For example, `079912`.
    pub section_id: String,
    /// The section code. For example, `B01`.
    pub section_code: String,
    /// All instructors (i.e., all of the instructors that appear in the `meetings`).
    pub all_instructors: Vec<String>,
    /// The number of available seats. For example, suppose a section had 30 seats
    /// total and there are 5 people enrolled. Then, this will be `25`.
    pub available_seats: i64,
    /// The number of students enrolled in this section. For example, suppose a
    /// section had 30 seats total and there are 5 people enrolled. Then, this will
    /// be `5`.
    pub enrolled_ct: i64,
    /// The total number of seats.
    pub total_seats: i64,
    /// The waitlist count.
    pub waitlist_ct: i64,
    /// All meetings.
    pub meetings: Vec<Meeting>,
    /// Whether you need to waitlist this.
    pub needs_waitlist: bool,
    /// Whether this is visible on WebReg
    pub is_visible: bool,
}

impl CourseSection {
    /// Checks if this section has any seats left.
    ///
    /// This function should be used because, sometimes, WebReg will say that
    /// there are some seats available; however, in reality, no seats are
    /// available and, usually, there is still a waitlist.
    ///
    /// # Returns
    /// `true` if there are seats and `false` otherwise.
    pub fn has_seats(&self) -> bool {
        self.available_seats > 0 && self.waitlist_ct == 0
    }
}

impl Display for CourseSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[{} / {}] {}",
            self.section_code, self.section_id, self.subj_course_id
        )?;
        writeln!(f, "\tInstructors: [{}]", self.all_instructors.join(", "))?;
        writeln!(f, "\tEnrolled: {}", self.available_seats)?;
        writeln!(f, "\tAvailable: {}", self.available_seats)?;
        writeln!(f, "\tWaitlist: {}", self.waitlist_ct)?;
        writeln!(f, "\tTotal Seats: {}", self.total_seats)?;
        writeln!(f, "\tCan Enroll? {}", self.has_seats())?;
        writeln!(f, "\tMeeting Information:")?;
        for meeting in &self.meetings {
            writeln!(f, "\t\t{meeting}")?;
        }

        Ok(())
    }
}

/// A meeting. Usually represents a lecture, final exam, discussion, and more.
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct Meeting {
    /// The meeting type. For example, this can be `LE`, `FI`, `DI`, etc.
    pub meeting_type: String,
    /// The meeting day(s). This is an enum that represents either a reoccurring meeting
    /// or one-time meeting.
    #[serde(rename = "meeting_days")]
    pub meeting_days: MeetingDay,
    /// The start hour. For example, if the meeting starts at 14:15, this would be `14`.
    pub start_hr: i16,
    /// The start minute. For example, if the meeting starts at 14:15, this would be `15`.
    pub start_min: i16,
    /// The end hour. For example, if the meeting ends at 15:05, this would be `15`.
    pub end_hr: i16,
    /// The end minute. For example, if the meeting ends at 15:05, this would be `5`.
    pub end_min: i16,
    /// The building where this meeting will occur. For example, if the meeting is held in
    /// `CENTR 115`, then this would be `CENTR`.
    pub building: String,
    /// The room number where this meeting will occur. For example, if the meeting is held in
    /// `CENTR 115`, then this would be `115`.
    pub room: String,
    /// The instructors assigned to this meeting.
    pub instructors: Vec<String>,
}

/// An enum that represents the meeting days for a section meeting.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum MeetingDay {
    /// The meeting is repeated. In this case, each element in the vector will be one of the
    /// following: `M`, `Tu`, `W`, `Th`, `F`, `Sa`, or `Su`.
    Repeated(Vec<String>),
    /// The meeting occurs once. In this case, the string will just be the date representation
    /// in the form `YYYY-MM-DD`.
    OneTime(String),
    /// There is no meeting.
    None,
}

impl Display for Meeting {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] ", self.meeting_type)?;
        match &self.meeting_days {
            MeetingDay::Repeated(r) => write!(f, "{} ", r.join("")),
            MeetingDay::OneTime(r) => write!(f, "{} ", r),
            MeetingDay::None => write!(f, "N/A "),
        }?;

        write!(
            f,
            "at {}:{:02} - {}:{:02} ",
            self.start_hr, self.start_min, self.end_hr, self.end_min
        )?;
        write!(f, "in {} {}", self.building, self.room)?;

        Ok(())
    }
}

/// A section that is currently in your schedule. Note that this can either be a course that you
/// are enrolled in, waitlisted for, or planned.
#[derive(Debug, Clone, Serialize)]
pub struct ScheduledSection {
    /// The section ID, for example `79903`.
    pub section_id: String,
    /// The subject code. For example, if this represents `CSE 100`, then this would be `CSE`.
    pub subject_code: String,
    /// The subject code. For example, if this represents `CSE 100`, then this would be `100`.
    pub course_code: String,
    /// The course title, for example `Advanced Data Structure`.
    pub course_title: String,
    /// The section code, for example `A01`.
    pub section_code: String,
    /// The section capacity (maximum number of people that can enroll in this section).
    pub section_capacity: i64,
    /// The number of people enrolled in this section.
    pub enrolled_count: i64,
    /// The number of available seats left.
    pub available_seats: i64,
    /// The grading option. This can be one of `L`, `P`, or `S`.
    pub grade_option: String,
    /// All instructors that appear in all of the meetings.
    pub all_instructors: Vec<String>,
    /// The number of units that you are taking this course for.
    pub units: f32,
    /// Your enrollment status.
    #[serde(rename = "enrolled_status")]
    pub enrolled_status: EnrollmentStatus,
    /// The number of people on the waitlist.
    pub waitlist_ct: i64,
    /// All relevant meetings for this section.
    pub meetings: Vec<Meeting>,
}

impl Display for ScheduledSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[{} / {}] {} {}: {}",
            self.section_code,
            self.section_id,
            self.section_code,
            self.course_code,
            self.course_title
        )?;
        writeln!(f, "\tInstructors: [{}]", self.all_instructors.join(", "))?;
        writeln!(f, "\tCourse Enrollment Information:")?;
        writeln!(f, "\t\tEnrolled: {}", self.available_seats)?;
        writeln!(f, "\t\tAvailable: {}", self.available_seats)?;
        writeln!(f, "\t\tWaitlist: {}", self.waitlist_ct)?;
        writeln!(f, "\t\tTotal Seats: {}", self.section_capacity)?;
        writeln!(f, "\tEnrollment Information:")?;
        write!(f, "\t\tStatus: ")?;
        match self.enrolled_status {
            EnrollmentStatus::Enrolled => writeln!(f, "Enrolled"),
            EnrollmentStatus::Waitlist { waitlist_pos } => {
                writeln!(f, "Waitlisted (Position {waitlist_pos})")
            }
            EnrollmentStatus::Planned => writeln!(f, "Planned"),
            EnrollmentStatus::Unknown => writeln!(f, "Unknown"),
        }?;

        writeln!(f, "\t\tUnits: {}", self.units)?;
        writeln!(f, "\t\tGrade Option: {}", self.grade_option)?;
        writeln!(f, "\tMeeting Information:")?;
        for meeting in &self.meetings {
            writeln!(f, "\t\t{meeting}")?;
        }

        Ok(())
    }
}

/// An enum that represents your enrollment status.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "enroll_status")]
pub enum EnrollmentStatus {
    Enrolled,
    Waitlist { waitlist_pos: i64 },
    Planned,
    Unknown,
}

/// A prerequisite for a course.
#[derive(Debug, Clone, Serialize)]
pub struct PrerequisiteInfo {
    /// Any course prerequsiites. This is a vector of vector of prerequisites,
    /// where each vector contains one or more prerequisites. Any prerequisites
    /// in the same vector means that you only need one of those prerequisites to
    /// fulfill that requirement.
    ///
    /// For example, if this value was `[[a, b], [c, d, e], [f]], then this means
    /// that you need
    /// - one of 'a' or 'b', *and*
    /// - one of 'c', 'd', or 'e', *and*
    /// - f.
    pub course_prerequisites: Vec<Vec<CoursePrerequisite>>,

    /// Any exam prerequisites. Exam prerequisites will satisfy all of the requirements
    /// defined by course prerequisites. In other words, if you satisfy one of the
    /// exam prerequisites, you automatically satisfy all of the course prerequisites.
    pub exam_prerequisites: Vec<String>,
}

/// A course prerequisite.
#[derive(Debug, Clone, Serialize)]
pub struct CoursePrerequisite {
    /// The subject, course ID. For example, `CSE 100`.
    pub subj_course_id: String,

    /// The course title, for example `Advanced Data Structure`.
    pub course_title: String,
}

/// An event on WebReg.
#[derive(Debug, Clone, Serialize)]
pub struct Event {
    /// The location of the event.
    pub location: String,
    /// The start hour. For example, if the meeting starts at 14:15, this would be `14`.
    pub start_hr: i16,
    /// The start minute. For example, if the meeting starts at 14:15, this would be `15`.
    pub start_min: i16,
    /// The end hour. For example, if the meeting ends at 15:05, this would be `15`.
    pub end_hr: i16,
    /// The end minute. For example, if the meeting ends at 15:05, this would be `5`.
    pub end_min: i16,
    /// The name of the event.
    pub name: String,
    /// The days that this event will occur.
    pub days: Vec<String>,
    /// The time when this event was created.
    pub timestamp: String,
}

// Helper structure for organizing meetings. Only used once for now.
#[derive(Debug)]
pub(crate) struct GroupedSection<'a, T> {
    /// All general meetings. These include meetings that are consistent across *all* sections.
    /// For example, lectures and final exams.
    pub general_meetings: Vec<&'a T>,

    /// All unique meetings. These are generally meetings that are unique the one section.
    /// For example, discussions.
    pub child_meetings: Vec<&'a T>,
}

/// Use this struct to add more information regarding the section that you want to enroll/waitlist
/// in.
pub struct EnrollWaitAdd<'a> {
    /// The section ID. For example, `0123123`.
    pub section_id: &'a str,
    /// The grading option. Can either be L, P, or S.
    /// If None is specified, this uses the default option.
    pub grading_option: Option<GradeOption>,
    /// The number of units. If none is specified, this
    /// uses the default unit count.
    pub unit_count: Option<u8>,
}

impl<'a> EnrollWaitAdd<'a> {
    /// Creates a new `EnrollWaitAdd` structure with the specified `section_id` and default grading
    /// option and unit count.
    ///
    /// # Parameters
    /// - `section_id`: The section ID.
    ///
    /// # Returns
    /// The structure.
    pub fn new(section_id: &'a str) -> Self {
        Self {
            section_id,
            grading_option: None,
            unit_count: None,
        }
    }
}

// This trait implementation may be helpful later.
impl<'a> AsRef<EnrollWaitAdd<'a>> for EnrollWaitAdd<'a> {
    fn as_ref(&self) -> &EnrollWaitAdd<'a> {
        self
    }
}

/// Use this struct to add more information regarding the course that you want to plan.
pub struct PlanAdd<'a> {
    /// The subject code. For example, `CSE`.
    pub subject_code: &'a str,
    /// The course code. For example, `12`.
    pub course_code: &'a str,
    /// The section ID. For example, `0123123`.
    pub section_id: &'a str,
    /// The section code. For example `A00`.
    pub section_code: &'a str,
    /// The grading option.
    pub grading_option: Option<GradeOption>,
    /// The schedule name.
    pub schedule_name: Option<&'a str>,
    /// The number of units.
    pub unit_count: u8,
}

/// A struct that represents an event to be added.
pub struct EventAdd<'a> {
    /// The name of the event. This is required.
    pub event_name: &'a str,
    /// The location of the event. This is optional.
    pub location: Option<&'a str>,
    /// The days that this event will be held.
    pub event_days: Vec<DayOfWeek>,
    /// The hour start time. For example, if the event starts at
    /// 3:50 PM, use `15` (since `12 + 3 = 15`).
    pub start_hr: i16,
    /// The minute start time. For example, if the event starts at
    /// 3:50 PM, use `50`.
    pub start_min: i16,
    /// The hour end time. For example, if the event ends at 3:50 PM,
    /// use `15` (since `12 + 3 = 15`).
    pub end_hr: i16,
    /// The minute end time. For example, if the event ends at 3:50 PM,
    /// use `50`.
    pub end_min: i16,
}

/// The possible grading options.
pub enum GradeOption {
    /// S/U grading (Satisfactory/Unsatisfactory) option.
    S,

    /// P/NP grading (Pass/No Pass) option.
    P,

    /// Letter grading option.
    L,
}

#[derive(Error, Debug)]
pub enum WrapperError {
    /// Occurs if there was an error encountered by the reqwest library.
    #[error("request error occurred: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Occurs when there was an error with serde.
    #[error("serde error occurred: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Occurs when the wrapper encounters a bad status code
    #[error("unsuccessful status code: {0}")]
    BadStatusCode(u16),

    /// Occurs when an error from WebReg was returned.
    #[error("error from WebReg: {0}")]
    WebRegError(String),

    /// Occurs when the given input is not valid.
    #[error("invalid input for '{0}' provided: {1}")]
    InputError(&'static str, &'static str),

    /// The general error, given when the particular error doesn't
    /// fit into any of the other categories.
    #[error("error: {0}")]
    GeneralError(String),

    /// Occurs when there was an error parsing the URL.
    #[error("malformed url: {0}")]
    UrlParseError(#[from] url::ParseError),
}

/// The generic type is the return value. Otherwise, regardless of request type,
/// we're just returning the error string if there is an error.
pub type Result<T, E = WrapperError> = std::result::Result<T, E>;

/// A term that is available on WebReg.
#[derive(Debug, Clone, Serialize)]
pub struct Term {
    /// The term ID.
    pub seq_id: i64,
    /// The term code (e.g., `SP23`).
    pub term_code: String,
}
