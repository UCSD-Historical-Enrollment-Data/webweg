use std::time::Duration;
use reqwest::{Client, IntoUrl, RequestBuilder};
use reqwest::header::{CONNECTION, COOKIE, USER_AGENT};

pub enum ReqType<U: IntoUrl> {
    Post(U),
    Get(U),
}

pub struct WebRegWrapperData {
    pub cookies: String,
    pub client: Client,
    pub user_agent: String,
    pub timeout: Duration,
    pub close_after_request: bool,
}

impl<'a> ReqwestWebRegClientData<'a> for WebRegWrapperData {
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

pub(crate) struct WebRegWrapperDataRef<'a> {
    pub cookies: &'a str,
    pub client: &'a Client,
    pub user_agent: &'a str,
    pub timeout: Duration,
    pub close_after_request: bool,
}

impl<'a> ReqwestWebRegClientData<'a> for WebRegWrapperDataRef<'a> {
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

pub trait ReqwestWebRegClientData<'a> {
    /// The cookies for this request.
    ///
    /// # Returns
    /// The cookies.
    fn get_cookies(&'a self) -> &'a str;

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
            U: IntoUrl
    {
        let client = self.get_client();
        let mut req = match req_type {
            ReqType::Post(u) => client.post(u),
            ReqType::Get(u) => client.get(u)
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