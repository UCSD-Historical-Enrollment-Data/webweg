use std::time::Duration;

use reqwest::Client;
use crate::wrapper::request_data::WebRegWrapperDataRef;

use crate::wrapper::requester_term::{WrapperTermRawRequest, WrapperTermRequest};
use crate::wrapper::WebRegWrapperData;

/// A structure that represents a request to be "built." This allows you to
/// override any settings set by the original wrapper for any requests made
/// under the soon-to-be-built requester.
///
/// Using this builder, you can either build
/// - the general requester, which is your main "gateway" to a majority of
///   the API.
/// - the raw requester, which gives you the ability to manually process
///   some API responses on your own.
pub struct WrapperTermRequestBuilder<'a> {
    pub(crate) cookies: &'a str,
    pub(crate) client: &'a Client,
    pub(crate) term: &'a str,
    pub(crate) user_agent: &'a str,
    pub(crate) timeout: Duration,
    pub(crate) close_after_request: bool,
}

impl<'a> WrapperTermRequestBuilder<'a> {
    /// Initializes a new builder with the settings derived from the wrapper.
    ///
    /// # Parameters
    /// - `wrapper`: The wrapper.
    ///
    /// # Returns
    /// The builder.
    pub fn new_request(wrapper_data: &'a WebRegWrapperData, term: &'a str) -> Self {
        Self {
            cookies: wrapper_data.cookies.as_str(),
            client: &wrapper_data.client,
            term,
            user_agent: wrapper_data.user_agent.as_str(),
            timeout: wrapper_data.timeout,
            close_after_request: wrapper_data.close_after_request,
        }
    }

    /// Overrides the cookies for any requests made under this soon-to-be requester.
    ///
    /// # Parameters
    /// - `cookies`: The cookies to use. This will _not_ override the cookies for the
    ///              wrapper, just this request.
    ///
    /// # Returns
    /// The builder.
    pub fn override_cookies(mut self, cookies: &'a str) -> Self {
        self.cookies = cookies;
        self
    }

    /// Overrides the client for any requests made under this soon-to-be requester.
    ///
    /// # Parameters
    /// - `client`: The client to use. This will _not_ override the client for the
    ///             wrapper, just this request.
    ///
    /// # Returns
    /// The builder.
    pub fn override_client(mut self, client: &'a Client) -> Self {
        self.client = client;
        self
    }

    /// Overrides the user agent for any requests made under this soon-to-be requester.
    ///
    /// # Parameters
    /// - `user_agent`: The user agent to use. This will _not_ override the user agent
    ///                 for the wrapper, just this request.
    ///
    /// # Returns
    /// The builder.
    pub fn override_user_agent(mut self, user_agent: &'a str) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Overrides the timeout for any requests made under this soon-to-be requester.
    ///
    /// # Parameters
    /// - `duration`: The timeout to use. This will _not_ override the timeout
    ///               for the wrapper, just this request.
    ///
    /// # Returns
    /// The builder.
    pub fn override_timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Whether the client should close the connection after completing the request.
    ///
    /// If you are using different cookies for this request, consider setting this to `true`.
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

    /// Builds the request builder. Note that this function is meant to be called
    /// internally by one of the two public build functions.
    ///
    /// # Returns
    /// A structure containing the actual request information.
    fn build(self) -> WebRegWrapperDataRef<'a> {
        WebRegWrapperDataRef {
            cookies: self.cookies,
            client: self.client,
            user_agent: self.user_agent,
            timeout: self.timeout,
            close_after_request: self.close_after_request,
        }
    }

    /// Builds the requester that can be used to generally obtain raw responses from WebReg.
    ///
    /// Note that you should use this requester if you want to manually parse the responses
    /// from WebReg yourself. The raw requester will handle some error handling from WebReg.
    ///
    /// It is recommended that you build the parsed requester instead, as this gives you
    /// significantly more access to the overall API. The parsed requester, as the name
    /// implies, also handles the parsing of the raw requests for you.
    ///
    /// # Returns
    /// The raw requester.
    pub fn raw(self) -> WrapperTermRawRequest<'a> {
        WrapperTermRawRequest { term: self.term, info: self.build() }
    }

    /// Builds the requester that can be used to make many different calls (GET, POST) to
    /// WebReg.
    ///
    /// # Returns
    /// The parsed requester.
    pub fn parsed(self) -> WrapperTermRequest<'a> {
        WrapperTermRequest {
            raw: self.raw(),
        }
    }
}
