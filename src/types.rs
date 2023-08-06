use std::fmt::{Display, Formatter};

use serde::Serialize;
use thiserror::Error;

/// The generic type is the return value. Otherwise, regardless of request type,
/// we're just returning the error string if there is an error.
pub type Result<T, E = WrapperError> = std::result::Result<T, E>;

/// The person's schedule.
pub type Schedule = Vec<ScheduledSection>;

/// All courses with the specified subject code & course number.
pub type Courses = Vec<CourseSection>;

/// Represents a search result from WebReg.
pub type SearchResult = Vec<SearchResultItem>;

/// Represents a vector of all events.
pub type Events = Vec<Event>;

/// The type that will be used to represent hours and minutes.
pub type TimeType = u32;

/// Represents a single search result item from WebReg.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct SearchResultItem {
    /// The subject code. For example, `CSE` or `MATH` are both possible option.
    pub subj_code: String,
    /// The course code. For example, `100B`.
    pub course_code: String,
    /// The course title. For example, `Abstract Algebra II`.
    pub course_title: String,
}

impl Display for SearchResultItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} - {}",
            self.subj_code, self.course_code, self.course_title
        )
    }
}

/// A section, which consists of a lecture, usually a discussion, and usually a final.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
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
        writeln!(f, "\tEnrolled: {}", self.enrolled_ct)?;
        writeln!(f, "\tAvailable: {}", self.available_seats)?;
        writeln!(f, "\tWaitlist: {}", self.waitlist_ct)?;
        writeln!(f, "\tTotal Seats: {}", self.total_seats)?;
        writeln!(f, "\tCan Enroll? {}", self.has_seats())?;
        writeln!(f, "\tMeeting Information:")?;
        for meeting in &self.meetings {
            write!(f, "\t\t{meeting}")?;
        }

        Ok(())
    }
}

/// A meeting. Usually represents a lecture, final exam, discussion, and more.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Meeting {
    /// The meeting type. For example, this can be `LE`, `FI`, `DI`, etc.
    pub meeting_type: String,
    /// The meeting day(s). This is an enum that represents either a reoccurring meeting
    /// or one-time meeting.
    #[serde(rename = "meeting_days")]
    pub meeting_days: MeetingDay,
    /// The start hour. For example, if the meeting starts at 14:15, this would be `14`.
    pub start_hr: TimeType,
    /// The start minute. For example, if the meeting starts at 14:15, this would be `15`.
    pub start_min: TimeType,
    /// The end hour. For example, if the meeting ends at 15:05, this would be `15`.
    pub end_hr: TimeType,
    /// The end minute. For example, if the meeting ends at 15:05, this would be `5`.
    pub end_min: TimeType,
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
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
        writeln!(f, "in {} {}", self.building, self.room)?;

        Ok(())
    }
}

/// A section that is currently in your schedule. Note that this can either be a course that you
/// are enrolled in, waitlisted for, or planned.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
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
    pub units: i64,
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
        writeln!(f, "\t\tEnrolled: {}", self.enrolled_count)?;
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
            write!(f, "\t\t{meeting}")?;
        }

        Ok(())
    }
}

/// An enum that represents your enrollment status.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "enroll_status")]
pub enum EnrollmentStatus {
    Enrolled,
    Waitlist { waitlist_pos: i64 },
    Planned,
    Unknown,
}

/// A prerequisite for a course.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct PrerequisiteInfo {
    /// Any course prerequisites. This is a vector of vector of prerequisites,
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct CoursePrerequisite {
    /// The subject, course ID. For example, `CSE 100`.
    pub subj_course_id: String,

    /// The course title, for example `Advanced Data Structure`.
    pub course_title: String,
}

impl CoursePrerequisite {
    /// Creates a new `CoursePrerequisite` object with the specified course information.
    ///
    /// # Parameters
    /// - `subj_course_id`: The subject, course ID (e.g., `CSE 100`)
    /// - `course_title`: The course title (e.g., `Advanced Data Structures`).
    ///
    /// # Returns
    /// The new `CoursePrerequisite` object.
    pub fn new(subj_course_id: impl Into<String>, course_title: impl Into<String>) -> Self {
        Self {
            subj_course_id: subj_course_id.into(),
            course_title: course_title.into(),
        }
    }
}

/// An event on WebReg.
#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub struct Event {
    /// The location of the event.
    pub location: String,
    /// The start hour. For example, if the meeting starts at 14:15, this would be `14`.
    pub start_hr: TimeType,
    /// The start minute. For example, if the meeting starts at 14:15, this would be `15`.
    pub start_min: TimeType,
    /// The end hour. For example, if the meeting ends at 15:05, this would be `15`.
    pub end_hr: TimeType,
    /// The end minute. For example, if the meeting ends at 15:05, this would be `5`.
    pub end_min: TimeType,
    /// The name of the event.
    pub name: String,
    /// The days that this event will occur.
    pub days: Vec<String>,
    /// The time when this event was created. Use this to replace or delete an event.
    pub timestamp: String,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Event] {}", self.name)?;
        writeln!(f, "\tLocation: {}", self.location)?;
        writeln!(f, "\tDay of Week: {}", self.days.join(""))?;
        writeln!(
            f,
            "\tTime: {}:{:02} - {}:{:02}",
            self.start_hr, self.start_min, self.end_hr, self.end_min
        )?;
        writeln!(f, "\tTimestamp: {}", self.timestamp)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum WrapperError {
    /// Occurs if there was an error encountered by the reqwest library.
    #[error("Request error occurred: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Occurs when there was an error parsing the URL.
    #[error("Malformed url: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Occurs when the given input is not valid.
    #[error("Invalid input for '{0}' provided: {1}")]
    InputError(&'static str, &'static str),

    /// Occurs when there was an error with serde. This error will most likely occur
    /// if you attempt to make a request to WebReg but your session isn't valid.
    #[error("Serde error occurred: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Occurs when the wrapper encounters a bad status code. This also includes some
    /// context as to why the error may occur, although the context is not cleaned so
    /// it may be very large (e.g., raw HTML).
    #[error("Unsuccessful status code: {0} (context: {1:?})")]
    BadStatusCode(u16, Option<String>),

    /// Occurs if there's a problem with parsing a time unit (minute or hour). For example,
    /// if hour was a negative value, then you can expect this error to occur.
    #[error("A time value, either minute or hour, is not formatted correctly.")]
    BadTimeError,

    // =============== //
    /// Occurs when an error from WebReg was returned. These are usually errors relating
    /// to you not being able to perform some operation (e.g, attempting to enroll in a
    /// class that you aren't able to enroll in).
    #[error("Error from WebReg: {0}")]
    WebRegError(String),

    /// Occurs if a section that you're trying to look for isn't available.
    #[error("Section ID not found: {0} (context: {1}")]
    SectionIdNotFound(String, SectionIdNotFoundContext),

    /// Occurs if there's an error with the parsing logic.
    #[error("An error occurred when parsing the response from WebReg: {0}")]
    WrapperParsingError(String),

    /// Occurs when your cookies may have expired.
    #[error("The current session is not valid. Are your cookies valid?")]
    SessionNotValid,
}

/// An enum to be used for giving more context into where the section ID wasn't found.
#[derive(Debug)]
pub enum SectionIdNotFoundContext {
    /// Whether the section ID wasn't found in your schedule (i.e., you didn't enroll in
    /// that section).
    Schedule,
    /// Whether the section ID wasn't found in the catalog (i.e., it's not offered
    /// in the specified term).
    Catalog,
}

impl Display for SectionIdNotFoundContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionIdNotFoundContext::Schedule => write!(f, "Schedule"),
            SectionIdNotFoundContext::Catalog => write!(f, "Offered"),
        }
    }
}

/// A term that is available on WebReg.
#[derive(Debug, Clone, Serialize)]
pub struct Term {
    /// The term ID.
    pub seq_id: i64,
    /// The term code (e.g., `SP23`).
    pub term_code: String,
}
