use std::borrow::Cow;

/// Use this struct to add more information regarding the section that you want to enroll/waitlist
/// in.
pub struct EnrollWaitAdd<'a> {
    /// The section ID. For example, `0123123`.
    pub section_id: Cow<'a, str>,
    /// The grading option. Can either be L, P, or S.
    /// If None is specified, this uses the default option.
    pub grading_option: Option<GradeOption>,
    /// The number of units. If none is specified, this
    /// uses the default unit count.
    pub unit_count: Option<u8>,
}

impl<'a> EnrollWaitAdd<'a> {
    /// Creates a new builder for the `EnrollWaitAdd` structure.
    ///
    /// # Returns
    /// The builder.
    pub fn builder() -> EnrollWaitAddBuilder<'a> {
        EnrollWaitAddBuilder {
            section_id: None,
            grading_option: None,
            unit_count: None,
        }
    }
}

pub struct EnrollWaitAddBuilder<'a> {
    section_id: Option<Cow<'a, str>>,
    grading_option: Option<GradeOption>,
    unit_count: Option<u8>,
}

impl<'a> EnrollWaitAddBuilder<'a> {
    pub fn new() -> Self {
        EnrollWaitAddBuilder {
            section_id: None,
            grading_option: None,
            unit_count: None,
        }
    }
    /// Sets the section ID for this builder. For example, `0123123` is a possible section ID.
    ///
    /// # Parameters
    /// - `section_id`: The section ID.
    ///
    /// # Returns
    /// The builder.
    pub fn with_section_id(mut self, section_id: impl Into<Cow<'a, str>>) -> Self {
        self.section_id = Some(section_id.into());
        self
    }

    /// Sets the grading option (L, P/NP, S/U) for this builder.
    ///
    /// # Parameters
    /// - `grading_option`: The grading option.
    ///
    /// # Returns
    /// The builder.
    pub fn with_grading_option(mut self, grading_option: GradeOption) -> Self {
        self.grading_option = Some(grading_option);
        self
    }

    /// Sets the number of units for this builder.
    ///
    /// # Parameters
    /// - `units`: The number of units.
    ///
    /// # Returns
    /// The builder.
    pub fn with_unit_count(mut self, units: u8) -> Self {
        self.unit_count = Some(units);
        self
    }

    /// Tries to build the `EnrollWaitAdd` object.
    ///
    /// # Returns
    /// The result of constructing this `EnrollWaitAdd` object. It is guaranteed that this will
    /// result the `EnrollWaitAdd` object if the section ID has been provided.
    pub fn try_build(self) -> Option<EnrollWaitAdd<'a>> {
        if let Some(section_id) = self.section_id {
            Some(EnrollWaitAdd {
                section_id,
                grading_option: self.grading_option,
                unit_count: self.unit_count,
            })
        } else {
            None
        }
    }
}

impl<'a> Default for EnrollWaitAddBuilder<'a> {
    fn default() -> Self {
        EnrollWaitAddBuilder::new()
    }
}

// This trait implementation may be helpful later.
impl<'a> AsRef<EnrollWaitAdd<'a>> for EnrollWaitAdd<'a> {
    fn as_ref(&self) -> &EnrollWaitAdd<'a> {
        self
    }
}

/// Use this struct to add more information regarding the course that you want to plan.
///
/// Prefer using the `PlanAddBuilder` to construct this object.
pub struct PlanAdd<'a> {
    /// The subject code. For example, `CSE`.
    pub subject_code: Cow<'a, str>,
    /// The course code. For example, `12`.
    pub course_code: Cow<'a, str>,
    /// The section ID. For example, `0123123`.
    pub section_id: Cow<'a, str>,
    /// The section code. For example `A00`.
    pub section_code: Cow<'a, str>,
    /// The grading option.
    pub grading_option: Option<GradeOption>,
    /// The schedule name. By default, this will use the default schedule.
    pub schedule_name: Option<Cow<'a, str>>,
    /// The number of units.
    pub unit_count: u8,
}

impl<'a> PlanAdd<'a> {
    /// Creates a builder to construct this `PlanAdd` object. This is recommended over
    /// directly creating the object yourself.
    ///
    /// # Returns
    /// The builder.
    pub fn builder() -> PlanAddBuilder<'a> {
        PlanAddBuilder::new()
    }
}

pub struct PlanAddBuilder<'a> {
    subject_code: Option<Cow<'a, str>>,
    course_code: Option<Cow<'a, str>>,
    section_id: Option<Cow<'a, str>>,
    section_code: Option<Cow<'a, str>>,
    grading_option: Option<GradeOption>,
    schedule_name: Option<Cow<'a, str>>,
    unit_count: Option<u8>,
}

impl<'a> PlanAddBuilder<'a> {
    /// Creates a new builder for the `PlanAdd` structure.
    ///
    /// # Returns
    /// The builder.
    pub fn new() -> Self {
        PlanAddBuilder {
            subject_code: None,
            course_code: None,
            section_id: None,
            section_code: None,
            grading_option: None,
            schedule_name: None,
            unit_count: None,
        }
    }

    /// Sets the subject code for this builder. For example, if `CSE 100` is the course,
    /// then you would use `CSE`.
    ///
    /// # Parameters
    /// - `subj_code`: The subject code.
    ///
    /// # Returns
    /// The builder.
    pub fn with_subject_code(mut self, subj_code: impl Into<Cow<'a, str>>) -> Self {
        self.subject_code = Some(subj_code.into());
        self
    }

    /// Sets the course code for this builder. For example, if `CSE 100` is the course,
    /// then you would use `100`.
    ///
    /// # Parameters
    /// - `course_code`: The course code.
    ///
    /// # Returns
    /// The builder.
    pub fn with_course_code(mut self, course_code: impl Into<Cow<'a, str>>) -> Self {
        self.course_code = Some(course_code.into());
        self
    }

    /// Sets the section ID for this builder. For example, `0123123` is a possible section ID.
    ///
    /// # Parameters
    /// - `section_id`: The section ID.
    ///
    /// # Returns
    /// The builder.
    pub fn with_section_id(mut self, section_id: impl Into<Cow<'a, str>>) -> Self {
        self.section_id = Some(section_id.into());
        self
    }

    /// Sets the section code for this builder. For example, `A01` is a possible section code.
    ///
    /// # Parameters
    /// - `section_code`: The section code.
    ///
    /// # Returns
    /// The builder.
    pub fn with_section_code(mut self, section_code: impl Into<Cow<'a, str>>) -> Self {
        self.section_code = Some(section_code.into());
        self
    }

    /// Sets the grading option (L, P/NP, S/U) for this builder.
    ///
    /// # Parameters
    /// - `grading_option`: The grading option.
    ///
    /// # Returns
    /// The builder.
    pub fn with_grading_option(mut self, grading_option: GradeOption) -> Self {
        self.grading_option = Some(grading_option);
        self
    }

    /// Sets the schedule name for this builder.
    ///
    /// # Parameters
    /// - `schedule_name`: The schedule name.
    ///
    /// # Returns
    /// The builder.
    pub fn with_schedule_name(mut self, schedule_name: impl Into<Cow<'a, str>>) -> Self {
        self.schedule_name = Some(schedule_name.into());
        self
    }

    /// Sets the number of units for this builder.
    ///
    /// # Parameters
    /// - `units`: The number of units.
    ///
    /// # Returns
    /// The builder.
    pub fn with_unit_count(mut self, units: u8) -> Self {
        self.unit_count = Some(units);
        self
    }

    /// Tries to build the `PlanAdd` object.
    ///
    /// # Returns
    /// The result of constructing this `PlanAdd` object. It is guaranteed that this will result
    /// the `PlanAdd` object if the following have been provided at the time of construction:
    /// - subject code,
    /// - course code,
    /// - section ID,
    /// - section code,
    /// - unit count.
    pub fn try_build(self) -> Option<PlanAdd<'a>> {
        if let (Some(s), Some(c), Some(sec_id), Some(sec_code), Some(u)) = (
            self.subject_code,
            self.course_code,
            self.section_id,
            self.section_code,
            self.unit_count,
        ) {
            Some(PlanAdd {
                subject_code: s,
                course_code: c,
                section_id: sec_id,
                section_code: sec_code,
                grading_option: self.grading_option,
                schedule_name: self.schedule_name,
                unit_count: u,
            })
        } else {
            None
        }
    }
}

impl<'a> Default for PlanAddBuilder<'a> {
    fn default() -> Self {
        PlanAddBuilder::new()
    }
}

/// A struct that represents an event to be added.
///
/// Prefer using the corresponding `EventAddBuilder` to build this object.
pub struct EventAdd<'a> {
    /// The name of the event. This is required.
    pub event_name: Cow<'a, str>,
    /// The location of the event. This is optional.
    pub location: Option<Cow<'a, str>>,
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

impl<'a> EventAdd<'a> {
    /// Creates a builder to construct this `EventAdd` object. This is recommended over
    /// directly creating the object yourself.
    ///
    /// # Returns
    /// The builder.
    pub fn builder() -> EventAddBuilder<'a> {
        EventAddBuilder::new()
    }
}

pub struct EventAddBuilder<'a> {
    event_name: Option<Cow<'a, str>>,
    location: Option<Cow<'a, str>>,
    event_days: Vec<DayOfWeek>,
    start_hr: Option<i16>,
    start_min: Option<i16>,
    end_hr: Option<i16>,
    end_min: Option<i16>,
}

impl<'a> EventAddBuilder<'a> {
    /// Creates a new builder for the `EventAdd` structure.
    ///
    /// # Returns
    /// The builder.
    pub fn new() -> Self {
        EventAddBuilder {
            event_name: None,
            location: None,
            event_days: vec![],
            start_hr: None,
            start_min: None,
            end_hr: None,
            end_min: None,
        }
    }

    /// Sets the name of this event.
    ///
    /// # Parameter
    /// - `name`: The name of the event.
    ///
    /// # Return
    /// The builder.
    pub fn with_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.event_name = Some(name.into());
        self
    }

    /// Sets the location of this event.
    ///
    /// # Parameter
    /// - `loc`: The location of the event.
    ///
    /// # Return
    /// The builder.
    pub fn with_location(mut self, loc: impl Into<Cow<'a, str>>) -> Self {
        self.location = Some(loc.into());
        self
    }

    /// Adds a day when this event will occur.
    ///
    /// # Parameter
    /// - `day`: The day that the event will be held.
    ///
    /// # Return
    /// The builder.
    pub fn with_day(mut self, day: DayOfWeek) -> Self {
        self.event_days.push(day);
        self
    }

    /// Sets the start time of the event.
    ///
    /// # Parameter
    /// - `hr`: The starting hour of the event, in 24-hour format.
    /// - `min`: The starting minute of the event.
    ///
    /// # Return
    /// The builder. The builder will only be modified if the `hr` and `min` arguments
    /// are valid (`0 <= hr <= 23` AND `0 <= min <= 59`).
    pub fn with_start_time(mut self, hr: i16, min: i16) -> Self {
        if (0..=23).contains(&hr) && (0..=59).contains(&min) {
            self.start_hr = Some(hr);
            self.start_min = Some(min);
        }

        self
    }

    /// Sets the end time of the event.
    ///
    /// # Parameter
    /// - `hr`: The ending hour of the event, in 24-hour format.
    /// - `min`: The ending minute of the event.
    ///
    /// # Return
    /// The builder. The builder will only be modified if the `hr` and `min` arguments
    /// are valid (`0 <= hr <= 23` AND `0 <= min <= 59`).
    pub fn with_end_time(mut self, hr: i16, min: i16) -> Self {
        if (0..=23).contains(&hr) && (0..=59).contains(&min) {
            self.end_hr = Some(hr);
            self.end_min = Some(min);
        }

        self
    }

    /// Attempts to build the event.
    ///
    /// # Returns
    /// The result of the construction of this object. It is guaranteed that this construction
    /// will be successful if the following fields were set:
    /// - the event name,
    /// - the event start time, and
    /// - the event end time.
    pub fn try_build(self) -> Option<EventAdd<'a>> {
        if let (Some(name), Some(s_hr), Some(s_min), Some(e_hr), Some(e_min)) = (
            self.event_name,
            self.start_hr,
            self.start_min,
            self.end_hr,
            self.end_min,
        ) {
            Some(EventAdd {
                event_name: name,
                location: self.location,
                event_days: self.event_days,
                start_hr: s_hr,
                start_min: s_min,
                end_hr: e_hr,
                end_min: e_min,
            })
        } else {
            None
        }
    }
}

impl<'a> Default for EventAddBuilder<'a> {
    fn default() -> Self {
        EventAddBuilder::new()
    }
}

/// The possible grading options.
#[derive(PartialOrd, PartialEq, Debug)]
pub enum GradeOption {
    /// S/U grading (Satisfactory/Unsatisfactory) option.
    S,

    /// P/NP grading (Pass/No Pass) option.
    P,

    /// Letter grading option.
    L,
}

impl GradeOption {
    /// Gets the (static) string representation of this `enum`.
    ///
    /// # Returns
    /// The static string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            GradeOption::L => "L",
            GradeOption::S => "S",
            GradeOption::P => "P",
        }
    }
}

/// An enum that represents how a course should be added to the person's schedule when
/// calling the corresponding `add_section` method (and associated methods).
pub enum AddType {
    /// Indicates that the user wants to enroll into the section.
    Enroll,
    /// Indicates that the user wants to waitlist the section.
    Waitlist,
    /// Have the library check whether the user should enroll or waitlist.
    DecideForMe,
}

/// An enum that's similar to `AddType`, but explicitly only allows `Enroll` or `Waitlist`
/// actions.
pub enum ExplicitAddType {
    /// Indicates that the user wants to enroll into the section.
    Enroll,
    /// Indicates that the user wants to waitlist the section.
    Waitlist,
}

/// Used to construct search requests for the `search_courses` function.
///
/// When building your request, you can either use one of the helper methods
/// to add information to your request, or even just directly modify the fields.
/// Note that the former does some validation.
#[derive(Clone)]
pub struct SearchRequestBuilder {
    pub subjects: Vec<String>,
    pub courses: Vec<String>,
    pub departments: Vec<String>,
    pub instructor: Option<String>,
    pub title: Option<String>,
    pub level_filter: u32,
    pub days: u32,
    pub start_time: Option<(u32, u32)>,
    pub end_time: Option<(u32, u32)>,
    pub only_open: bool,
}

impl SearchRequestBuilder {
    /// Creates a new instance of the `SearchRequestBuilder`, which is used to search for specific
    /// courses.
    ///
    /// # Returns
    /// The empty `SearchRequestBuilder`.
    pub fn new() -> Self {
        Self {
            subjects: vec![],
            courses: vec![],
            departments: vec![],
            instructor: None,
            title: None,
            level_filter: 0,
            days: 0,
            start_time: None,
            end_time: None,
            only_open: false,
        }
    }

    /// Adds a subject to this search request. Valid search requests are uppercase and at most
    /// 4 characters long. Some examples include `MATH` or `CSE`.
    ///
    /// # Parameters
    /// - `subject`: The subject.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn add_subject(mut self, subject: impl Into<String>) -> Self {
        let s: String = subject.into();
        if s.len() > 4 {
            return self;
        }

        self.subjects.push(s.to_uppercase());
        self
    }

    /// Adds a course (either a subject code, course code, or both) to the search request. Some
    /// examples include `20E`, `math 20d`, `101`, `CSE`.
    ///
    /// # Parameters
    /// - `course`: The course.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn add_course(mut self, course: impl Into<String>) -> Self {
        self.courses.push(course.into());
        self
    }

    /// Adds a department to the search request. Valid search requests are uppercase and at most 4
    /// characters long. Some examples include `MATH` or `CSE`.
    ///
    /// # Parameters
    /// - `department`: The department.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn add_department(mut self, department: impl Into<String>) -> Self {
        let d: String = department.into();
        if d.len() > 4 {
            return self;
        }

        self.departments.push(d.to_uppercase());
        self
    }

    /// Sets the instructor to the specified instructor.
    ///
    /// # Parameters
    /// - `instructor`: The instructor. This should be formatted in `Last Name, First Name` form.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_instructor(mut self, instructor: impl Into<String>) -> Self {
        self.instructor = Some(instructor.into());
        self
    }

    /// Sets the course title to the specified title. Some examples could be `differential equ`,
    /// `data structures`, `algorithms`, and so on.
    ///
    /// # Parameters
    /// - `title`: The title of the course.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Restrict search results to to the specified filter. This can be applied multiple times.
    ///
    /// # Parameters
    /// - `filter`: The filter.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn filter_courses_by(mut self, filter: CourseLevelFilter) -> Self {
        self.level_filter |= match filter {
            CourseLevelFilter::LowerDivision => 1 << 11,
            CourseLevelFilter::FreshmenSeminar => 1 << 10,
            CourseLevelFilter::LowerDivisionIndependentStudy => 1 << 9,
            CourseLevelFilter::UpperDivision => 1 << 8,
            CourseLevelFilter::Apprenticeship => 1 << 7,
            CourseLevelFilter::UpperDivisionIndependentStudy => 1 << 6,
            CourseLevelFilter::Graduate => 1 << 5,
            CourseLevelFilter::GraduateIndependentStudy => 1 << 4,
            CourseLevelFilter::GraduateResearch => 1 << 3,
            CourseLevelFilter::Lvl300 => 1 << 2,
            CourseLevelFilter::Lvl400 => 1 << 1,
            CourseLevelFilter::Lvl500 => 1 << 0,
        };

        self
    }

    /// Only shows courses based on the specified day(s).
    ///
    /// # Parameters
    /// - `day`: The day.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn apply_day(mut self, day: DayOfWeek) -> Self {
        let day = match day {
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
            DayOfWeek::Sunday => 7,
        };

        self.days |= 1 << (7 - day);
        self
    }

    /// Sets the start time to the specified time.
    ///
    /// # Parameters
    /// - `hour`: The hour. This should be between 0 and 23, inclusive.
    /// - `min`: The minute. This should be between 0 and 59, inclusive.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_start_time(mut self, hour: u32, min: u32) -> Self {
        if hour > 23 || min > 59 {
            return self;
        }

        self.start_time = Some((hour, min));
        self
    }

    /// Sets the end time to the specified time.
    ///
    /// # Parameters
    /// - `hour`: The hour. This should be between 0 and 23, inclusive.
    /// - `min`: The minute. This should be between 0 and 59, inclusive.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn set_end_time(mut self, hour: u32, min: u32) -> Self {
        if hour > 23 || min > 59 {
            return self;
        }

        self.end_time = Some((hour, min));
        self
    }

    /// Whether to only show sections with open seats.
    ///
    /// # Returns
    /// The `SearchRequestBuilder`
    pub fn only_allow_open(mut self) -> Self {
        self.only_open = true;
        self
    }
}

impl Default for SearchRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The day of week enum, which designates what days you want
/// to filter specific sections by.
#[derive(PartialOrd, PartialEq, Debug)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// The course level filter enum, which can be used to filter
/// specific sections by.
pub enum CourseLevelFilter {
    /// Level 1-99 courses.
    LowerDivision,
    /// Level 87, 90 courses.
    FreshmenSeminar,
    /// Level 99 courses.
    LowerDivisionIndependentStudy,
    /// Level 100-198 courses
    UpperDivision,
    /// Level 195 courses
    Apprenticeship,
    /// Level 199 courses
    UpperDivisionIndependentStudy,
    /// Level 200-297 courses
    Graduate,
    /// Level 298 courses
    GraduateIndependentStudy,
    /// Level 299 courses
    GraduateResearch,
    /// Level 300+ courses
    Lvl300,
    /// Level 400+ courses
    Lvl400,
    /// Level 500+ courses
    Lvl500,
}

/// Lets you choose how you want to search for a course. It is recommended that
/// you use one of the associated functions to create this enum instance.
pub enum SearchType {
    /// Searches for a course by section ID.
    BySection(String),

    /// Searches for a course by more than one section ID.
    ByMultipleSections(Vec<String>),

    /// Searches for a (set of) course(s) by multiple specifications.
    Advanced(SearchRequestBuilder),
}
