use std::time::Duration;

use reqwest::Client;

use crate::wrapper::requester_term::{WrapperTermRawRequest, WrapperTermRequest};
use crate::wrapper::WebRegWrapper;

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
}

impl<'a> WrapperTermRequestBuilder<'a> {
    /// Initializes a new builder with the settings derived from the wrapper.
    ///
    /// # Parameters
    /// - `wrapper`: The wrapper.
    ///
    /// # Returns
    /// The builder.
    pub fn new_request(wrapper: &'a WebRegWrapper) -> Self {
        Self {
            cookies: &wrapper.cookies,
            client: &wrapper.client,
            term: &wrapper.term,
            user_agent: &wrapper.user_agent,
            timeout: wrapper.default_timeout,
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

    /// Overrides the term for any requests made under this soon-to-be requester.
    ///
    /// # Parameters
    /// - `term`: The term to use. This will _not_ override the term for the
    ///           wrapper, just this request.
    ///
    /// # Returns
    /// The builder.
    pub fn override_term(mut self, term: &'a str) -> Self {
        self.term = term;
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
    pub fn build_term_raw(self) -> WrapperTermRawRequest<'a> {
        WrapperTermRawRequest { info: self }
    }

    /// Builds the requester that can be used to make many different calls (GET, POST) to
    /// WebReg.
    ///
    /// # Returns
    /// The parsed requester.
    pub fn build_term_parser(self) -> WrapperTermRequest<'a> {
        WrapperTermRequest {
            raw: self.build_term_raw(),
        }
    }
}
