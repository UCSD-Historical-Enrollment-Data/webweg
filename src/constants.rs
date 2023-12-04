pub(crate) const MY_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, \
like Gecko) Chrome/97.0.4692.71 Safari/537.36";

pub(crate) const STATUS_ENROLL: &str = "EN";
pub(crate) const STATUS_WAITLIST: &str = "WT";
pub(crate) const STATUS_PLANNED: &str = "PL";

/// The default schedule name.
pub(crate) const DEFAULT_SCHEDULE_NAME: &str = "My Schedule";

// URLs for WebReg
pub(crate) const WEBREG_SEARCH: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-by-all?";
pub(crate) const WEBREG_SEARCH_SEC: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-by-sectionid?";
pub(crate) const ACC_NAME: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/get-current-name";
pub(crate) const COURSE_DATA: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-load-group-data?";
pub(crate) const CURR_SCHEDULE: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/get-class?";
pub(crate) const SEND_EMAIL: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/send-email";
pub(crate) const CHANGE_ENROLL: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/change-enroll";

pub(crate) const REMOVE_SCHEDULE: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/sched-remove";
pub(crate) const RENAME_SCHEDULE: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-rename";
pub(crate) const ALL_SCHEDULE: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/sched-get-schednames";

pub(crate) const PING_SERVER: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/ping-server";

pub(crate) const PLAN_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-add";
pub(crate) const PLAN_REMOVE: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-remove";
pub(crate) const PLAN_EDIT: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/edit-plan";
pub(crate) const PLAN_REMOVE_ALL: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/plan-remove-all";

pub(crate) const ENROLL_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/add-enroll";
pub(crate) const ENROLL_EDIT: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/edit-enroll";
pub(crate) const ENROLL_DROP: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/drop-enroll";

pub(crate) const WAITLIST_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/add-wait";
pub(crate) const WAITLIST_EDIT: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/edit-wait";
pub(crate) const WAITLIST_DROP: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/drop-wait";

pub(crate) const PREREQS_INFO: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/get-prerequisites?";

pub(crate) const EVENT_ADD: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-add";
pub(crate) const EVENT_EDIT: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-edit";
pub(crate) const EVENT_REMOVE: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-remove";
pub(crate) const EVENT_GET: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/secure/event-get?";

pub(crate) const STATUS_START: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/get-status-start?";
pub(crate) const ELIGIBILITY: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/check-eligibility?";

pub(crate) const SUBJ_LIST: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-load-subject?";
pub(crate) const DEPT_LIST: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-load-department?";

pub(crate) const COURSE_TEXT: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-get-crse-text?";

pub(crate) const SECTION_TEXT: &str =
    "https://act.ucsd.edu/webreg2/svc/wradapter/secure/search-get-section-text?";

pub(crate) const TERM_LIST: &str = "https://act.ucsd.edu/webreg2/svc/wradapter/get-term?";

pub(crate) const VERIFY_FAIL_ERR: &str = "[{\"VERIFY\":\"FAIL\"}]";
