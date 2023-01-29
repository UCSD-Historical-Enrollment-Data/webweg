<p align="center">
  <img src="https://raw.githubusercontent.com/ewang2002/webweg/stable/assets/banner_webweg.png"  alt="Project Banner"/>
</p>

<p align="center">
  <b>webweg</b> |
  <a href="https://github.com/ewang2002/webreg_scraper">webreg_scraper</a> |
  <a href="https://github.com/ewang2002/UCSDHistEnrollData">UCSDHistEnrollmentData</a>
</p>

An asynchronous API wrapper, written in Rust, for the [University of California San Diego](https://ucsd.edu/)'s [WebReg](https://act.ucsd.edu/webreg2/start) course enrollment system.

## Usage
To use this crate, please see the **Install** section of webweg's [crates.io](https://crates.io/crates/webweg) page. 

## Wrapper Features
A lot of the things that you can do on WebReg can be done with this 
wrapper. For example, you're able to:
- Get all possible classes in the quarter.
- Search for classes based on some conditions (i.e., advanced search). 
- Get detailed information about a specific class (e.g., number of students 
enrolled, instructor, etc.)
- Get your current schedule. 

You're also able to do things like:
- Change grading options. 
- Enroll in, or drop, a class.
- Plan, or un-plan, a class.
- Waitlist, or un-waitlist, a class.
- Create, remove, or rename your schedules. 
- Send a confirmation email to yourself.


## Authentication

<details>
<summary>Click Here</summary>
<br>

The way to provide authorization for this wrapper is to provide cookies from an
active WebReg session (i.e., your authentication cookies).

To get your authentication cookies, you'll need to do the following:
- Log into WebReg.
- Select a term in the WebReg main menu.
- Open Developer Tools (With Google Chrome, go to the three dots, "More tools,"
  and then "Developer tools.")
- Go to the "Network" tab of the Developer Tools. Then, either:
    - Filter by the text `https://act.ucsd.edu/webreg2/svc/wradapter`
    - OR, filter by `Fetch/XHR`.
- Make some sort of request on WebReg (e.g., searching a course).
- Look for a request made by WebReg. 
    - Under the request headers, copy the cookie.

Keep in mind that your cookies will expire after either:
- 10 minutes of inactivity (i.e., you do not make some request that uses your
  cookies for more than 10 minutes), or
- when WebReg goes into maintenance mode; this occurs daily at around
  4:15AM pacific time.

Thus, you will need to find some way to keep yourself logged into WebReg 24/7
if you want to perform continuous requests.

</details>


## Walkthrough

<details>
<summary>Click Here</summary>
<br>

To use the wrapper, you need to create a new instance of it. For example:
```rs
use reqwest::Client;
use webweg::wrapper::WebRegWrapper;

let term = "SP22";
// For authentication cookies, see previous section.
let cookie = "your authentication cookies here";
let w = WebRegWrapper::new(Client::new(), cookie.to_string(), term);
```
For your convenience, `reqwest` is automatically exported so you can use that
from `webweg`.

Once created, you're able to use the various wrapper functions. Some useful 
examples are shown below (note that `w` refers to the declaration above).

Most wrapper functions will return a `Result<T, Error>`, where `T` is the 
result type. So, if a request is successful, you'll get `T` back. If the
request is unsuccessful, you will get back an `Error`, which contains
information about why the request failed.

### Check Login Status
You can check to see if you are logged in (i.e., if the wrapper can actually 
perform any useful requests).
```rs
if !w.is_valid().await {
    eprintln!("You aren't logged in!");
    return; 
}
```

### Get Schedule
You can get your current schedule, which lists your Enrolled, Planned, and
Waitlisted courses. You are able to fetch either the default schedule (`None`) 
or a specific schedule (e.g., `My Schedule 2`)

Example: Suppose you wanted to see what courses are currently in your *default* 
schedule. We can use the following code:

```rs
let my_schedule = w.get_schedule(None).await;
match my_schedule {
    Ok(s) => s.into_iter().for_each(|p| println!("{}", p.to_string())),
    Err(e) => eprintln!("{}", e),
};
```

**Remark:** If you wanted to see what courses you have planned in some other 
schedule, you can replace `None` with `Some("your schedule name here")`.



### Get Course Information
You are able to search up course information for a particular course. If no 
issues occur, then this function will return a vector where each element 
contains the instructor name, number of seats, and all meetings.

Example: Suppose we wanted to look up all CSE 101 sections. We can use the following code:

```rs
let courses_101 = w.get_course_info("CSE", "101").await;
match courses_101 {
    Ok(s) => s.into_iter().for_each(|p| println!("{}", p.to_string())),
    Err(e) => eprintln!("{}", e),
};
```

### Search Courses
You can also search up courses that meet a particular criteria. This is 
very similar in nature to the Advanced Search option.


Example 1: Suppose we wanted to search for specific sections. In our 
example below, we'll search for one section of CSE 100, one section 
of Math 184, and one section of POLI 28 (for Winter 2022). The following 
code will do just that: 

```rs
use webweg::wrapper::SearchType;

let search_res = w
    .search_courses_detailed(SearchType::ByMultipleSections(&[
        "079913", "078616", "075219",
    ]))
    .await;
match search_res {
    Ok(s) => s.into_iter().for_each(|p| println!("{}", p.to_string())),
    Err(e) => eprintln!("{}", e),
};
```

Example 2: Suppose we wanted to search for any lower- or upper-division 
CSE course. We can use the following code:

```rs 
use webweg::wrapper::{CourseLevelFilter, SearchRequestBuilder};

let search_res = w
    .search_courses_detailed(SearchType::Advanced(
        &SearchRequestBuilder::new()
            .add_department("CSE")
            .filter_courses_by(CourseLevelFilter::UpperDivision)
            .filter_courses_by(CourseLevelFilter::LowerDivision),
    ))
    .await;
match search_res {
    Ok(s) => s.into_iter().for_each(|p| println!("{}", p.to_string())),
    Err(e) => eprintln!("{}", e),
};
```


### Planning & Un-planning a Section
You can use the wrapper to plan a section, adding it to your schedule.

Example 1: Suppose you wanted to plan a section of CSE 100 to your default 
schedule. You can use the following code:

```rs
let res = w.add_to_plan(PlanAdd {
    subject_code: "CSE",
    course_code: "100",
    section_id: "079911",
    section_code: "A01",
    // Using S/U grading.
    grading_option: Some(GradeOption::S),
    // Put in default schedule
    schedule_name: None,
    unit_count: 4
}, true).await;
match res {
    Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    Err(e) => eprintln!("{}", e),
};
```

Example 2: Suppose you want to remove the section of CSE 100 from your default
schedule. You can use the following code:

```rs
let res = w.remove_from_plan("079911", None).await;
match res {
    Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    Err(e) => eprintln!("{}", e),
};
```

**Remark:** If you wanted to add (or remove) this section to (from) a different 
schedule, you can do so by replacing `None` with `Some("your schedule name here")`.

### Enrolling & Waitlisting Sections
You can also use the wrapper to programmatically enroll or waitlist particular 
sections.

Example: Suppose we wanted to enroll or waitlist a section of Math 184 with 
section number `078616`, and then drop it afterwards. This is how we could
do this.

```rs
use std::time::Duration;
use webweg::types::EnrollmentStatus;
use webweg::wrapper::EnrollWaitAdd;

let section_res = w
    .search_courses_detailed(SearchType::BySection("078616"))
    .await
    .unwrap_or_else(|_| vec![]);

if section_res.is_empty() {
    eprintln!("No section found.");
    return;
}

let add_res = w
    .add_section(
        section_res[0].has_seats(),
        EnrollWaitAdd {
            section_id: "078616",
            // Use default grade option
            grading_option: None,
            // Use default unit count
            unit_count: None,
        },
        // Validate our request with WebReg
        true,
    )
    .await;

match add_res {
    Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    Err(e) => {
        eprintln!("{}", e);
        return;
    }
};

// Wait a bit, although this is unnecessary.
std::thread::sleep(Duration::from_secs(2));

// Get your current schedule
let course_to_drop = w
    .get_schedule(None)
    .await
    .unwrap_or_else(|_| vec![])
    .into_iter()
    .find(|x| x.section_id == 78616);

// Check if we're enrolled in this course
let is_enrolled = if let Some(r) = course_to_drop {
    match r.enrolled_status {
        EnrollmentStatus::Enrolled => true,
        _ => false,
    }
} else {
    eprintln!("Course not enrolled or waitlisted.");
    return;
};

// Drop the class.
let rem_res = w.drop_section(is_enrolled, "078616").await;

match rem_res {
    Ok(o) => println!("{}", if o { "Successful" } else { "Unsuccessful" }),
    Err(e) => eprintln!("{}", e),
};
```

</details>

## Definition Files
This crate comes with two definition files:
- `raw_types`
- `types`

Most wrapper methods will make use of return types which can be found in
`types`. Very rarely will you need to use `raw_types`;
the only time you will need to use `raw_types` is if you're using
the `search_courses` method.

## Tests
Very basic tests can be found in the `tests` folder. You will need
to provide your cookies in the `cookie.txt` file; place this file in
the project root directory (i.e., the directory with the `src` and 
`tests` directories).

Due to WebReg constantly changing, making long-term tests is not
feasible. Tests for a specific term are not guaranteed to be maintained. 

That being said, there are tests for all utility functions (things that
can be tested in the long-term).

## Versioning
This crate uses a versioning scheme that is roughly based on [Semantic Versioning](https://semver.org/). For a version
```
MAJOR.MINOR.PATCH
```
- the `MAJOR` version will be incremented when a very significant feature is added, _or_ **many** non-backwards compatible changes are added, _or_ a (one or more) **significant** non-backwards compatible change is added
- the `MINOR` version will be incremented when a minor feature is added, _or_ **few** (if any) minor non-backwards compatible changes are added.
- the `PATCH` version will be incremented when a minor enhancement is added or a bug is fixed.

## Disclaimer
I am not responsible for any damages or other issue(s) caused by 
any use of this wrapper. In other words, by using this wrapper, 
I am not responsible if you somehow get in trouble or otherwise 
run into problems.

## License
Everything in this repository is licensed under the MIT license.