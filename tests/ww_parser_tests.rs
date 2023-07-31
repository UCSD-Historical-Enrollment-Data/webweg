#[cfg(test)]
mod prerequisites_tests {
    use webweg::raw_types::RawPrerequisite;
    use webweg::types::{CoursePrerequisite, PrerequisiteInfo};
    use webweg::ww_parser::parse_prerequisites;

    /// Sorts the prerequisite objects so that we can check equality without needing to use
    /// a HashMap.
    ///
    /// This will modify the `course_prerequisites` and `exam_prerequisites` fields so that
    /// each element in the corresponding vector is in sorted order, but won't add or modify
    /// each element specifically.
    ///
    /// # Parameters
    /// - `prereqs`: The prerequisite object.
    fn sort_prerequisites(prereqs: &mut PrerequisiteInfo) {
        prereqs
            .course_prerequisites
            .iter_mut()
            .for_each(|prereq| prereq.sort_unstable_by(|a, b| a.course_title.cmp(&b.course_title)));

        prereqs
            .course_prerequisites
            .sort_unstable_by(|a, b| a[0].course_title.cmp(&b[0].course_title));

        prereqs.exam_prerequisites.sort_unstable();
    }

    #[test]
    pub fn test_course_single_grouping() {
        let prereq_str = include_str!("json/prereq1.json");
        let raw_prereqs = serde_json::from_str::<Vec<RawPrerequisite>>(prereq_str).unwrap();

        let mut res = parse_prerequisites(raw_prereqs).unwrap();
        let mut expected = PrerequisiteInfo {
            course_prerequisites: vec![vec![
                CoursePrerequisite::new("CSE 8B", "Intro to Programming 2"),
                CoursePrerequisite::new("CSE 11", "Accel. Intro to Programming"),
            ]],
            exam_prerequisites: vec![],
        };

        sort_prerequisites(&mut res);
        sort_prerequisites(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_multiple_course_grouping() {
        let prereq_str = include_str!("json/prereq2.json");
        let raw_prereqs = serde_json::from_str::<Vec<RawPrerequisite>>(prereq_str).unwrap();

        let mut res = parse_prerequisites(raw_prereqs).unwrap();
        let mut expected = PrerequisiteInfo {
            course_prerequisites: vec![
                vec![CoursePrerequisite::new(
                    "CSE 12",
                    "Basic Data Struct & OO Design",
                )],
                vec![CoursePrerequisite::new(
                    "CSE 15L",
                    "Software Tools&Techniques Lab",
                )],
            ],
            exam_prerequisites: vec![],
        };

        sort_prerequisites(&mut res);
        sort_prerequisites(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_multiple_course_grouping_many() {
        let prereq_str = include_str!("json/prereq3.json");
        let raw_prereqs = serde_json::from_str::<Vec<RawPrerequisite>>(prereq_str).unwrap();

        let mut res = parse_prerequisites(raw_prereqs).unwrap();
        let mut expected = PrerequisiteInfo {
            course_prerequisites: vec![
                vec![
                    CoursePrerequisite::new("CSE 21", "Math/Algorithm&Systems Analys"),
                    CoursePrerequisite::new("MATH 154", "Discrete Math & Graph Theory"),
                    CoursePrerequisite::new("MATH 158", "Extremal Combinatorics/Graph"),
                    CoursePrerequisite::new("MATH 184", "Enumerative Combinatorics"),
                    CoursePrerequisite::new("MATH 188", "Algebraic Combinatorics"),
                ],
                vec![CoursePrerequisite::new(
                    "CSE 12",
                    "Basic Data Struct & OO Design",
                )],
                vec![CoursePrerequisite::new(
                    "CSE 15L",
                    "Software Tools&Techniques Lab",
                )],
                vec![
                    CoursePrerequisite::new("CSE 30", "Computer Organiz&Systms Progrm"),
                    CoursePrerequisite::new("ECE 15", "Engineering Computation"),
                ],
            ],
            exam_prerequisites: vec![],
        };

        sort_prerequisites(&mut res);
        sort_prerequisites(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_exam_and_course_grouping() {
        let prereq_str = include_str!("json/prereq4.json");
        let raw_prereqs = serde_json::from_str::<Vec<RawPrerequisite>>(prereq_str).unwrap();

        let mut res = parse_prerequisites(raw_prereqs).unwrap();
        let mut expected = PrerequisiteInfo {
            course_prerequisites: vec![vec![CoursePrerequisite::new(
                "MATH 20B",
                "Calculus/Science & Engineering",
            )]],
            exam_prerequisites: vec!["AP-Math BC".into()],
        };

        sort_prerequisites(&mut res);
        sort_prerequisites(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_exam_single_grouping() {
        let prereq_str = include_str!("json/prereq5.json");
        let raw_prereqs = serde_json::from_str::<Vec<RawPrerequisite>>(prereq_str).unwrap();

        let mut res = parse_prerequisites(raw_prereqs).unwrap();
        let mut expected = PrerequisiteInfo {
            course_prerequisites: vec![],
            exam_prerequisites: vec![
                "ACT Math Subscore".into(),
                "Intermed Algebra Pre-Calc".into(),
                "SAT Math Section Score".into(),
            ],
        };

        sort_prerequisites(&mut res);
        sort_prerequisites(&mut expected);
        assert_eq!(expected, res);
    }
}

#[cfg(test)]
mod schedule_tests {
    use webweg::raw_types::RawScheduledMeeting;
    use webweg::types::{EnrollmentStatus, Meeting, MeetingDay, ScheduledSection};
    use webweg::ww_parser::parse_schedule;

    /// Sorts the schedule objects so that we can check equality without needing to use
    /// a HashMap.
    ///
    /// This will modify the ordering of the elements as well as the meetings, but won't
    /// add or modify the elements themselves.
    ///
    /// # Parameters
    /// - `sch`: The scheduled section objects.
    fn sort_schedules(sch: &mut [ScheduledSection]) {
        sch.sort_unstable_by(|a, b| a.section_id.cmp(&b.section_id));
        sch.iter_mut().for_each(|s| {
            s.meetings
                .sort_unstable_by(|a, b| a.meeting_type.cmp(&b.meeting_type))
        });
    }

    #[test]
    pub fn test_no_meetings() {
        let schedule = include_str!("json/schedule2.json");
        let raw_schedule = serde_json::from_str::<Vec<RawScheduledMeeting>>(schedule).unwrap();

        let res = parse_schedule(raw_schedule).unwrap();
        let expected = vec![ScheduledSection {
            section_id: "290181".into(),
            subject_code: "CSE".into(),
            course_code: "199".into(),
            course_title: "Independent Study".into(),
            section_code: "001".into(),
            section_capacity: 9999,
            enrolled_count: 1,
            available_seats: 9998,
            grade_option: "P".into(),
            all_instructors: vec!["Sahoo, Debashis".into()],
            units: 2,
            enrolled_status: EnrollmentStatus::Planned,
            waitlist_ct: 0,
            meetings: vec![Meeting {
                meeting_type: "IN".into(),
                meeting_days: MeetingDay::Repeated(vec![]),
                start_hr: 0,
                start_min: 0,
                end_hr: 0,
                end_min: 0,
                building: "TBA".into(),
                room: "TBA".into(),
                instructors: vec!["Sahoo, Debashis".into()],
            }],
        }];

        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_complex_schedule() {
        let schedule = include_str!("json/schedule1.json");
        let raw_schedule = serde_json::from_str::<Vec<RawScheduledMeeting>>(schedule).unwrap();

        let mut res = parse_schedule(raw_schedule).unwrap();
        let mut expected = vec![
            ScheduledSection {
                section_id: "185826".into(),
                subject_code: "HILA".into(),
                course_code: "102".into(),
                course_title: "Latin America/Twentieth Centry".into(),
                section_code: "A00".into(),
                section_capacity: 20,
                enrolled_count: 7,
                available_seats: 13,
                grade_option: "P".into(),
                all_instructors: vec!["Staff".into()],
                units: 4,
                enrolled_status: EnrollmentStatus::Enrolled,
                waitlist_ct: 0,
                meetings: vec![
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["M".into()]),
                        start_hr: 12,
                        start_min: 30,
                        end_hr: 12 + 1,
                        end_min: 50,
                        building: "YORK".into(),
                        room: "4050B".into(),
                        instructors: vec!["Staff".into()],
                    },
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["Tu".into()]),
                        start_hr: 12,
                        start_min: 30,
                        end_hr: 12 + 1,
                        end_min: 50,
                        building: "YORK".into(),
                        room: "4050B".into(),
                        instructors: vec!["Staff".into()],
                    },
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["W".into()]),
                        start_hr: 12,
                        start_min: 30,
                        end_hr: 12 + 1,
                        end_min: 50,
                        building: "YORK".into(),
                        room: "4050B".into(),
                        instructors: vec!["Staff".into()],
                    },
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["Th".into()]),
                        start_hr: 12,
                        start_min: 30,
                        end_hr: 12 + 1,
                        end_min: 50,
                        building: "YORK".into(),
                        room: "4050B".into(),
                        instructors: vec!["Staff".into()],
                    },
                    Meeting {
                        meeting_type: "FI".into(),
                        meeting_days: MeetingDay::OneTime("2023-09-08".into()),
                        start_hr: 11,
                        start_min: 30,
                        end_hr: 12 + 2,
                        end_min: 29,
                        building: "YORK".into(),
                        room: "4050B".into(),
                        instructors: vec!["Staff".into()],
                    },
                ],
            },
            ScheduledSection {
                section_id: "184959".into(),
                subject_code: "COGS".into(),
                course_code: "118B".into(),
                course_title: "Intro to Machine Learning".into(),
                section_code: "A01".into(),
                section_capacity: 90,
                enrolled_count: 90,
                available_seats: 0,
                grade_option: "L".into(),
                all_instructors: vec!["Gupta, Anjum".into()],
                units: 4,
                enrolled_status: EnrollmentStatus::Waitlist { waitlist_pos: 26 },
                waitlist_ct: 26,
                meetings: vec![
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["M".into()]),
                        start_hr: 12 + 5,
                        start_min: 0,
                        end_hr: 12 + 7,
                        end_min: 50,
                        building: "RCLAS".into(),
                        room: "R01".into(),
                        instructors: vec!["Gupta, Anjum".into()],
                    },
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["W".into()]),
                        start_hr: 12 + 5,
                        start_min: 0,
                        end_hr: 12 + 7,
                        end_min: 50,
                        building: "RCLAS".into(),
                        room: "R01".into(),
                        instructors: vec!["Gupta, Anjum".into()],
                    },
                    Meeting {
                        meeting_type: "FI".into(),
                        meeting_days: MeetingDay::OneTime("2023-09-08".into()),
                        start_hr: 12 + 7,
                        start_min: 0,
                        end_hr: 12 + 9,
                        end_min: 59,
                        building: "RCLAS".into(),
                        room: "R01".into(),
                        instructors: vec!["Gupta, Anjum".into()],
                    },
                    Meeting {
                        meeting_type: "DI".into(),
                        meeting_days: MeetingDay::Repeated(vec!["M".into()]),
                        start_hr: 12 + 4,
                        start_min: 0,
                        end_hr: 12 + 4,
                        end_min: 50,
                        building: "RCLAS".into(),
                        room: "R02".into(),
                        instructors: vec!["Gupta, Anjum".into()],
                    },
                    Meeting {
                        meeting_type: "DI".into(),
                        meeting_days: MeetingDay::Repeated(vec!["W".into()]),
                        start_hr: 12 + 4,
                        start_min: 0,
                        end_hr: 12 + 4,
                        end_min: 50,
                        building: "RCLAS".into(),
                        room: "R02".into(),
                        instructors: vec!["Gupta, Anjum".into()],
                    },
                ],
            },
        ];

        sort_schedules(&mut expected);
        sort_schedules(&mut res);
        assert_eq!(expected, res);
    }
}

#[cfg(test)]
mod course_info_tests {
    use webweg::raw_types::RawWebRegMeeting;
    use webweg::types::{CourseSection, Meeting, MeetingDay};
    use webweg::ww_parser::parse_course_info;

    /// Sorts the course section objects so that we can check equality without needing to use
    /// a HashMap.
    ///
    /// This will modify the ordering of the elements as well as the meetings, but won't
    /// add or modify the elements themselves.
    ///
    /// # Parameters
    /// - `sch`: The course section objects.
    fn sort_course_sections(sections: &mut [CourseSection]) {
        sections.sort_unstable_by(|a, b| a.section_id.cmp(&b.section_id));
        sections.iter_mut().for_each(|s| {
            s.meetings
                .sort_unstable_by(|a, b| a.meeting_type.cmp(&b.meeting_type))
        });
    }

    #[test]
    pub fn test_one_section_family_one_section() {
        let schedule = include_str!("json/courseinfo1.json");
        let raw_schedule = serde_json::from_str::<Vec<RawWebRegMeeting>>(schedule).unwrap();
        let mut res = parse_course_info(raw_schedule, "CSE 101".into()).unwrap();

        let mut expected = vec![CourseSection {
            subj_course_id: "CSE 101".into(),
            section_id: "260739".into(),
            section_code: "A01".into(),
            all_instructors: vec!["Bach, Quang Tran".into()],
            available_seats: 0,
            enrolled_ct: 329,
            total_seats: 245,
            waitlist_ct: 125,
            meetings: vec![
                Meeting {
                    meeting_type: "LE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["M".into(), "W".into(), "F".into()]),
                    start_hr: 12 + 2,
                    start_min: 0,
                    end_hr: 12 + 2,
                    end_min: 50,
                    building: "WLH".into(),
                    room: "2001".into(),
                    instructors: vec!["Bach, Quang Tran".into()],
                },
                Meeting {
                    meeting_type: "DI".into(),
                    meeting_days: MeetingDay::Repeated(vec!["F".into()]),
                    start_hr: 12 + 4,
                    start_min: 0,
                    end_hr: 12 + 4,
                    end_min: 50,
                    building: "PETER".into(),
                    room: "108".into(),
                    instructors: vec!["Bach, Quang Tran".into()],
                },
                Meeting {
                    meeting_type: "MI".into(),
                    meeting_days: MeetingDay::OneTime("2023-10-27".into()),
                    start_hr: 12 + 7,
                    start_min: 0,
                    end_hr: 12 + 8,
                    end_min: 50,
                    building: "GH".into(),
                    room: "242".into(),
                    instructors: vec!["Bach, Quang Tran".into()],
                },
                Meeting {
                    meeting_type: "MI".into(),
                    meeting_days: MeetingDay::OneTime("2023-11-17".into()),
                    start_hr: 12 + 7,
                    start_min: 0,
                    end_hr: 12 + 8,
                    end_min: 50,
                    building: "YORK".into(),
                    room: "2722".into(),
                    instructors: vec!["Bach, Quang Tran".into()],
                },
                Meeting {
                    meeting_type: "FI".into(),
                    meeting_days: MeetingDay::OneTime("2023-12-13".into()),
                    start_hr: 12 + 3,
                    start_min: 0,
                    end_hr: 12 + 5,
                    end_min: 59,
                    building: "WLH".into(),
                    room: "2001".into(),
                    instructors: vec!["Bach, Quang Tran".into()],
                },
            ],
            is_visible: true,
        }];

        sort_course_sections(&mut res);
        sort_course_sections(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_two_section_family_one_section() {
        let schedule = include_str!("json/courseinfo2.json");
        let raw_schedule = serde_json::from_str::<Vec<RawWebRegMeeting>>(schedule).unwrap();
        let mut res = parse_course_info(raw_schedule, "CSE 30".into()).unwrap();

        let mut expected = vec![
            CourseSection {
                subj_course_id: "CSE 30".into(),
                section_id: "260735".into(),
                section_code: "A01".into(),
                all_instructors: vec!["Chin, Bryan W.".into()],
                available_seats: 0,
                enrolled_ct: 152,
                total_seats: 100,
                waitlist_ct: 53,
                meetings: vec![
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                        start_hr: 12,
                        start_min: 30,
                        end_hr: 12 + 1,
                        end_min: 50,
                        building: "FAH".into(),
                        room: "1301".into(),
                        instructors: vec!["Chin, Bryan W.".into()],
                    },
                    Meeting {
                        meeting_type: "DI".into(),
                        meeting_days: MeetingDay::Repeated(vec!["W".into()]),
                        start_hr: 12 + 6,
                        start_min: 0,
                        end_hr: 12 + 6,
                        end_min: 50,
                        building: "FAH".into(),
                        room: "1301".into(),
                        instructors: vec!["Chin, Bryan W.".into()],
                    },
                    Meeting {
                        meeting_type: "MI".into(),
                        meeting_days: MeetingDay::OneTime("2023-10-26".into()),
                        start_hr: 12 + 8,
                        start_min: 0,
                        end_hr: 12 + 9,
                        end_min: 50,
                        building: "MOS".into(),
                        room: "0113".into(),
                        instructors: vec!["Chin, Bryan W.".into()],
                    },
                    Meeting {
                        meeting_type: "FI".into(),
                        meeting_days: MeetingDay::OneTime("2023-12-09".into()),
                        start_hr: 11,
                        start_min: 30,
                        end_hr: 12 + 2,
                        end_min: 29,
                        building: "MOS".into(),
                        room: "0113".into(),
                        instructors: vec!["Chin, Bryan W.".into()],
                    },
                ],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "CSE 30".into(),
                section_id: "249208".into(),
                section_code: "B01".into(),
                all_instructors: vec!["Cao, Yingjun".into()],
                available_seats: 0,
                enrolled_ct: 127,
                total_seats: 100,
                waitlist_ct: 29,
                meetings: vec![
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                        start_hr: 8,
                        start_min: 0,
                        end_hr: 9,
                        end_min: 20,
                        building: "LEDDN".into(),
                        room: "AUD".into(),
                        instructors: vec!["Cao, Yingjun".into()],
                    },
                    Meeting {
                        meeting_type: "DI".into(),
                        meeting_days: MeetingDay::Repeated(vec!["W".into()]),
                        start_hr: 12 + 5,
                        start_min: 0,
                        end_hr: 12 + 5,
                        end_min: 50,
                        building: "FAH".into(),
                        room: "1301".into(),
                        instructors: vec!["Cao, Yingjun".into()],
                    },
                    Meeting {
                        meeting_type: "MI".into(),
                        meeting_days: MeetingDay::OneTime("2023-10-26".into()),
                        start_hr: 12 + 8,
                        start_min: 0,
                        end_hr: 12 + 9,
                        end_min: 50,
                        building: "MOS".into(),
                        room: "0114".into(),
                        instructors: vec!["Cao, Yingjun".into()],
                    },
                    Meeting {
                        meeting_type: "FI".into(),
                        meeting_days: MeetingDay::OneTime("2023-12-09".into()),
                        start_hr: 11,
                        start_min: 30,
                        end_hr: 12 + 2,
                        end_min: 29,
                        building: "MOS".into(),
                        room: "0114".into(),
                        instructors: vec!["Cao, Yingjun".into()],
                    },
                ],
                is_visible: true,
            },
        ];

        sort_course_sections(&mut res);
        sort_course_sections(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_one_section_family_canceled_one() {
        let schedule = include_str!("json/courseinfo3.json");
        let raw_schedule = serde_json::from_str::<Vec<RawWebRegMeeting>>(schedule).unwrap();
        let mut res = parse_course_info(raw_schedule, "MATH 100C".into()).unwrap();

        let mut expected = vec![
            CourseSection {
                subj_course_id: "MATH 100C".into(),
                section_id: "142034".into(),
                section_code: "A01".into(),
                all_instructors: vec!["Pollack, Aaron".into()],
                available_seats: 9,
                enrolled_ct: 18,
                total_seats: 27,
                waitlist_ct: 0,
                meetings: vec![
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec![
                            "M".into(),
                            "W".into(),
                            "F".into(),
                        ]),
                        start_hr: 12,
                        start_min: 0,
                        end_hr: 12,
                        end_min: 50,
                        building: "WLH".into(),
                        room: "2204".into(),
                        instructors: vec!["Pollack, Aaron".into()],
                    },
                    Meeting {
                        meeting_type: "DI".into(),
                        meeting_days: MeetingDay::Repeated(vec!["Tu".into()]),
                        start_hr: 9,
                        start_min: 0,
                        end_hr: 9,
                        end_min: 50,
                        building: "APM".into(),
                        room: "B412".into(),
                        instructors: vec!["Pollack, Aaron".into()],
                    },
                    Meeting {
                        meeting_type: "FI".into(),
                        meeting_days: MeetingDay::OneTime("2023-06-14".into()),
                        start_hr: 11,
                        start_min: 30,
                        end_hr: 12 + 2,
                        end_min: 29,
                        building: "WLH".into(),
                        room: "2204".into(),
                        instructors: vec!["Pollack, Aaron".into()],
                    },
                ],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "MATH 100C".into(),
                section_id: "254672".into(),
                section_code: "A03".into(),
                all_instructors: vec!["Pollack, Aaron".into()],
                available_seats: 12,
                enrolled_ct: 13,
                total_seats: 25,
                waitlist_ct: 0,
                meetings: vec![
                    Meeting {
                        meeting_type: "LE".into(),
                        meeting_days: MeetingDay::Repeated(vec![
                            "M".into(),
                            "W".into(),
                            "F".into(),
                        ]),
                        start_hr: 12,
                        start_min: 0,
                        end_hr: 12,
                        end_min: 50,
                        building: "WLH".into(),
                        room: "2204".into(),
                        instructors: vec!["Pollack, Aaron".into()],
                    },
                    Meeting {
                        meeting_type: "DI".into(),
                        meeting_days: MeetingDay::Repeated(vec!["Tu".into()]),
                        start_hr: 8,
                        start_min: 0,
                        end_hr: 8,
                        end_min: 50,
                        building: "APM".into(),
                        room: "B412".into(),
                        instructors: vec!["Pollack, Aaron".into()],
                    },
                    Meeting {
                        meeting_type: "FI".into(),
                        meeting_days: MeetingDay::OneTime("2023-06-14".into()),
                        start_hr: 11,
                        start_min: 30,
                        end_hr: 12 + 2,
                        end_min: 29,
                        building: "WLH".into(),
                        room: "2204".into(),
                        instructors: vec!["Pollack, Aaron".into()],
                    },
                ],
                is_visible: true,
            },
        ];

        sort_course_sections(&mut res);
        sort_course_sections(&mut expected);
        assert_eq!(expected, res);
    }

    #[test]
    pub fn test_number_sections() {
        let schedule = include_str!("json/courseinfo4.json");
        let raw_schedule = serde_json::from_str::<Vec<RawWebRegMeeting>>(schedule).unwrap();
        let mut res = parse_course_info(raw_schedule, "WCWP 10A".into()).unwrap();

        let mut expected = vec![
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144434".into(),
                section_code: "001".into(),
                all_instructors: vec!["Gagnon, Jeffrey C".into()],
                available_seats: 0,
                enrolled_ct: 15,
                total_seats: 15,
                waitlist_ct: 0,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["M".into(), "W".into()]),
                    start_hr: 11,
                    start_min: 0,
                    end_hr: 12,
                    end_min: 20,
                    building: "EBU3B".into(),
                    room: "1113".into(),
                    instructors: vec!["Gagnon, Jeffrey C".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144435".into(),
                section_code: "002".into(),
                all_instructors: vec!["Gagnon, Jeffrey C".into()],
                available_seats: 0,
                enrolled_ct: 15,
                total_seats: 15,
                waitlist_ct: 2,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["M".into(), "W".into()]),
                    start_hr: 12,
                    start_min: 30,
                    end_hr: 12 + 1,
                    end_min: 50,
                    building: "EBU3B".into(),
                    room: "1113".into(),
                    instructors: vec!["Gagnon, Jeffrey C".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144437".into(),
                section_code: "003".into(),
                all_instructors: vec!["Gagnon, Jeffrey C".into()],
                available_seats: 0,
                enrolled_ct: 15,
                total_seats: 15,
                waitlist_ct: 1,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                    start_hr: 9,
                    start_min: 30,
                    end_hr: 10,
                    end_min: 50,
                    building: "EBU3B".into(),
                    room: "1113".into(),
                    instructors: vec!["Gagnon, Jeffrey C".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144438".into(),
                section_code: "004".into(),
                all_instructors: vec!["Gagnon, Jeffrey C".into()],
                available_seats: 0,
                enrolled_ct: 15,
                total_seats: 15,
                waitlist_ct: 2,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                    start_hr: 11,
                    start_min: 0,
                    end_hr: 12,
                    end_min: 20,
                    building: "EBU3B".into(),
                    room: "1113".into(),
                    instructors: vec!["Gagnon, Jeffrey C".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144439".into(),
                section_code: "005".into(),
                all_instructors: vec!["Susi, Natalie".into()],
                available_seats: 1,
                enrolled_ct: 19,
                total_seats: 20,
                waitlist_ct: 0,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                    start_hr: 9,
                    start_min: 30,
                    end_hr: 10,
                    end_min: 50,
                    building: "SOLIS".into(),
                    room: "105".into(),
                    instructors: vec!["Susi, Natalie".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144440".into(),
                section_code: "006".into(),
                all_instructors: vec!["Gagnon, Jeffrey C".into()],
                available_seats: 0,
                enrolled_ct: 20,
                total_seats: 20,
                waitlist_ct: 1,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["M".into(), "W".into()]),
                    start_hr: 12,
                    start_min: 30,
                    end_hr: 12 + 1,
                    end_min: 50,
                    building: "EBU3B".into(),
                    room: "1124".into(),
                    instructors: vec!["Gagnon, Jeffrey C".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144441".into(),
                section_code: "007".into(),
                all_instructors: vec!["Ornelas, Tricia".into()],
                available_seats: 0,
                enrolled_ct: 20,
                total_seats: 20,
                waitlist_ct: 1,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                    start_hr: 8,
                    start_min: 0,
                    end_hr: 9,
                    end_min: 20,
                    building: "WSAC".into(),
                    room: "138".into(),
                    instructors: vec!["Ornelas, Tricia".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144442".into(),
                section_code: "008".into(),
                all_instructors: vec!["Ornelas, Tricia".into()],
                available_seats: 0,
                enrolled_ct: 20,
                total_seats: 20,
                waitlist_ct: 0,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                    start_hr: 9,
                    start_min: 30,
                    end_hr: 10,
                    end_min: 50,
                    building: "WSAC".into(),
                    room: "138".into(),
                    instructors: vec!["Ornelas, Tricia".into()],
                }],
                is_visible: true,
            },
            CourseSection {
                subj_course_id: "WCWP 10A".into(),
                section_id: "144443".into(),
                section_code: "009".into(),
                all_instructors: vec!["Ornelas, Tricia".into()],
                available_seats: 0,
                enrolled_ct: 20,
                total_seats: 20,
                waitlist_ct: 1,
                meetings: vec![Meeting {
                    meeting_type: "SE".into(),
                    meeting_days: MeetingDay::Repeated(vec!["Tu".into(), "Th".into()]),
                    start_hr: 12,
                    start_min: 30,
                    end_hr: 12 + 1,
                    end_min: 50,
                    building: "WSAC".into(),
                    room: "138".into(),
                    instructors: vec!["Ornelas, Tricia".into()],
                }],
                is_visible: true,
            },
        ];

        sort_course_sections(&mut res);
        sort_course_sections(&mut expected);
        assert_eq!(expected, res);
    }
}
