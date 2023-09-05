#[cfg(feature = "multi")]
use parking_lot::Mutex;
use reqwest::header::{CONNECTION, COOKIE, USER_AGENT};
use reqwest::{Client, IntoUrl, RequestBuilder};
use std::time::Duration;

pub(crate) enum ReqType<U: IntoUrl> {
    Post(U),
    Get(U),
}

/// A structure that represents data held by the wrapper or a request.
pub struct WebRegWrapperData {
    /// The cookies.
    #[cfg(feature = "multi")]
    pub(crate) cookies: Mutex<String>,
    #[cfg(not(feature = "multi"))]
    pub(crate) cookies: String,
    /// The client used to make the request.
    pub(crate) client: Client,
    /// The user agent.
    pub(crate) user_agent: String,
    /// The timeout for this request.
    pub(crate) timeout: Duration,
    /// Whether to close the connection after the request has been completed.
    pub(crate) close_after_request: bool,
}

impl<'a> ReqwestWebRegClientData<'a> for WebRegWrapperData {
    #[cfg(feature = "multi")]
    fn get_cookies(&'a self) -> String {
        self.cookies.lock().to_owned()
    }
    #[cfg(not(feature = "multi"))]
    fn get_cookies(&'a self) -> &'a str {
        self.cookies.as_str()
    }

    fn get_client(&'a self) -> &'a Client {
        &self.client
    }

    fn get_user_agent(&'a self) -> &'a str {
        self.user_agent.as_str()
    }

    fn get_timeout(&'a self) -> Duration {
        self.timeout
    }

    fn close_after_request(&'a self) -> bool {
        self.close_after_request
    }
}

/// A structure that represents data held by the wrapper or a request.
pub(crate) struct WebRegWrapperDataRef<'a> {
    #[cfg(feature = "multi")]
    pub cookies: String,
    /// The cookies.
    #[cfg(not(feature = "multi"))]
    pub cookies: &'a str,
    /// The client used to make the request.
    pub client: &'a Client,
    /// The user agent.
    pub user_agent: &'a str,
    /// The timeout for this request.
    pub timeout: Duration,
    /// Whether to close the connection after the request has been completed.
    pub close_after_request: bool,
}

impl<'a> ReqwestWebRegClientData<'a> for WebRegWrapperDataRef<'a> {
    #[cfg(feature = "multi")]
    fn get_cookies(&'a self) -> String {
        self.cookies.to_owned()
    }
    #[cfg(not(feature = "multi"))]
    fn get_cookies(&'a self) -> &'a str {
        self.cookies
    }

    fn get_client(&'a self) -> &'a Client {
        self.client
    }

    fn get_user_agent(&'a self) -> &'a str {
        self.user_agent
    }

    fn get_timeout(&'a self) -> Duration {
        self.timeout
    }

    fn close_after_request(&'a self) -> bool {
        self.close_after_request
    }
}

pub(crate) trait ReqwestWebRegClientData<'a> {
    /// The cookies for this request.
    ///
    /// # Returns
    /// The cookies.
    #[cfg(not(feature = "multi"))]
    fn get_cookies(&'a self) -> &'a str;
    #[cfg(feature = "multi")]
    fn get_cookies(&'a self) -> String;

    /// The client to be used for this request.
    ///
    /// # Returns
    /// The client.
    fn get_client(&'a self) -> &'a Client;

    /// The user agent to be used for this request.
    ///
    /// # Returns
    /// The user agent.
    fn get_user_agent(&'a self) -> &'a str;

    /// The timeout to be used for this request.
    ///
    /// # Returns
    /// The timeout.
    fn get_timeout(&'a self) -> Duration;

    /// Whether the connection should be closed after the request is completed.
    ///
    /// # Returns
    /// Whether the connection should be closed after the request is completed.
    fn close_after_request(&'a self) -> bool;

    /// Makes a request with the desired request type using the headers provided by the user.
    ///
    /// # Parameters
    /// - `req_type`: The request type.
    ///
    /// # Returns
    /// A request builder that can further be built on top of, if needed.
    fn req<U>(&'a self, req_type: ReqType<U>) -> RequestBuilder
    where
        U: IntoUrl,
    {
        let client = self.get_client();
        let mut req = match req_type {
            ReqType::Post(u) => client.post(u),
            ReqType::Get(u) => client.get(u),
        }
        .header(COOKIE, self.get_cookies())
        .header(USER_AGENT, self.get_user_agent())
        .timeout(self.get_timeout());

        if self.close_after_request() {
            req = req.header(CONNECTION, "close");
        }

        req
    }
}
