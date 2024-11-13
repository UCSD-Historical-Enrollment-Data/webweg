<p align="center">
  <img src="https://raw.githubusercontent.com/ewang2002/webweg/stable/assets/banner_webweg.png"  alt="Project Banner"/>
</p>

<p align="center">
  <b>webweg</b> |
  <a href="https://github.com/ewang2002/webreg_scraper">webreg_scraper</a> |
  <a href="https://github.com/ewang2002/UCSDHistEnrollData">UCSDHistEnrollmentData</a>
</p>

An asynchronous API wrapper for the [University of California San Diego](https://ucsd.edu/)'s [WebReg](https://act.ucsd.edu/webreg2/start) course enrollment system.

## Usage
To use this crate, run the following command:
```
cargo add webweg
```
Alternatively, you can also put the following line into your `Cargo.toml`:
```toml
webweg = "0.9"
```

See the corresponding [crates.io](https://crates.io/crates/webweg) page for more information.

## Wrapper Features
A lot of the things that you can do on WebReg can be done with this wrapper. For example, you're able to:
- Get all possible classes in the quarter.
- Search for classes based on some conditions (i.e., advanced search). 
- Get detailed information about a specific class (e.g., number of students enrolled, instructor, etc.)
- Get your current schedule. 

You're also able to do things like:
- Change grading options. 
- Enroll in, or drop, a class.
- Plan, or un-plan, a class.
- Waitlist, or un-waitlist, a class.
- Create, remove, or rename your schedules. 
- Send a confirmation email to yourself.

To see some examples, check out the `examples` folder.

## Multithreading
By default, the wrapper will assume an environment where it is used in a single thread or task. So, if you want to share
an instance of the wrapper across multiple threads or tasks, you would have to use a `Mutex`. The problem with using a 
`Mutex` is that if a request is taking a long time (which isn't unusual), the entire wrapper is locked until the request
is finished, meaning the other threads using this instance of the wrapper must wait.

By enabling the `multi` feature, the wrapper can be used across multiple threads or tasks _without_ the need for a `Mutex`.

To add this crate with the `multi` feature, you can either run
```
cargo add webweg --features multi
```
or put 
```toml
webweg = { version = "0.9", features = ["multi"] }
```
in your `Cargo.toml`.

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

## Definition Files
This crate comes with two definition files:
- `raw_types`
- `types`

Most wrapper methods will make use of return types which can be found in
`types`. Very rarely will you need to use `raw_types`;
the only time you will need to use `raw_types` is if you're using
the `search_courses` method.

## Tests
Many tests here are focused on the _parsing_ aspect of the wrapper, and
not making the request itself. It is assumed that making the request 
should be relatively error-free.

## Versioning
This crate uses a versioning scheme that is roughly based on [Semantic Versioning](https://semver.org/). For a version
```
MAJOR.MINOR.PATCH
```
- the `MAJOR` version will be incremented when a very significant feature is added, _or_ **many** non-backwards compatible changes are added, _or_ a (one or more) **significant** non-backwards compatible change is added
- the `MINOR` version will be incremented when a minor feature is added, _or_ **few** (if any) minor non-backwards compatible changes are added.
- the `PATCH` version will be incremented when a minor enhancement/feature is added or a bug is fixed.

## Disclaimer
I am not responsible for any damages or other issue(s) caused by 
any use of this wrapper. In other words, by using this wrapper, 
I am not responsible if you somehow get in trouble or otherwise 
run into problems.

## License
Everything in this repository is licensed under the MIT license.