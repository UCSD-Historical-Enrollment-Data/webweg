use std::time::Duration;

use crate::constants::MY_USER_AGENT;
use crate::wrapper::request_data::WebRegWrapperData;
use reqwest::Client;

use crate::wrapper::WebRegWrapper;

/// A builder for the `WebRegWrapper`. This should be used to construct a new wrapper.
///
/// # Example
/// ```rs
/// let wrapper = WebRegWrapperBuilder::new()
///     .with_cookies("abc")
///     .with_default_term("FA23")
///     .with_default_timeout(Duration::from_secs(10))
///     .try_build_wrapper();
///
/// assert!(wrapper.is_some());
/// ```
pub struct WebRegWrapperBuilder {
    cookies: Option<String>,
    client: Client,
    user_agent: String,
    default_timeout: Duration,
    close_after_request: bool,
}

impl WebRegWrapperBuilder {
    /// Constructs a `WebRegWrapperBuilder` with the default client, user agent, and default timeout.
    /// You are responsible for providing the cookies and term.
    ///
    /// # Returns
    /// A `WebRegWrapperBuilder`.
    pub fn new() -> Self {
        Self {
            cookies: None,
            client: Client::new(),
            user_agent: MY_USER_AGENT.to_owned(),
            default_timeout: Duration::from_secs(30),
            close_after_request: false,
        }
    }

    /// Sets the cookies to the specified cookies.
    ///
    /// # Parameters
    /// - `cookie`: The WebReg cookies.
    ///
    /// # Returns
    /// The builder.
    pub fn with_cookies(mut self, cookie: impl Into<String>) -> Self {
        self.cookies = Some(cookie.into());
        self
    }

    /// Sets the client to the specified client.
    ///
    /// # Parameters
    /// - `client`: The client to use.
    ///
    /// # Returns
    /// The builder.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Sets the user agent to the specified user agent.
    ///
    /// # Parameters
    /// - `user_agent`: The user agent to use.
    ///
    /// # Returns
    /// The builder.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Sets the timeout to the specified timeout.
    ///
    /// # Parameters
    /// - `timeout`: The timeout to use.
    ///
    /// # Returns
    /// The builder.
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Whether the client should close the connection after completing the request.
    ///
    /// If you are planning on using multiple active cookies for the same wrapper, set
    /// this to `true`. Otherwise, you might get stale login errors.
    ///
    /// # Parameters
    /// - `close`: Whether to close the connection after completing the request.
    ///
    /// # Returns
    /// The builder.
    pub fn should_close_after_request(mut self, close: bool) -> Self {
        self.close_after_request = close;
        self
    }

    /// Attempts to build the wrapper. To successfully build the wrapper, the cookies and term
    /// must be provided.
    ///
    /// # Returns
    /// The `WebRegWrapper` if both the `cookies` and `term` are specified. If any of those
    /// are not specified, `None` will be returned.
    pub fn try_build_wrapper(self) -> Option<WebRegWrapper> {
        if let Some(cookies) = self.cookies {
            Some(WebRegWrapper {
                data: WebRegWrapperData {
                    cookies,
                    client: self.client,
                    user_agent: self.user_agent,
                    timeout: self.default_timeout,
                    close_after_request: self.close_after_request,
                },
            })
        } else {
            None
        }
    }
}

impl Default for WebRegWrapperBuilder {
    fn default() -> Self {
        Self::new()
    }
}
