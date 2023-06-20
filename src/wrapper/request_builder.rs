use crate::wrapper::requester_term::{WrapperTermRawRequest, WrapperTermRequest};
use crate::wrapper::WebRegWrapper;
use reqwest::Client;
use std::time::Duration;

pub struct WrapperTermRequestBuilder<'a> {
    pub(crate) cookies: &'a str,
    pub(crate) client: &'a Client,
    pub(crate) term: &'a str,
    pub(crate) user_agent: &'a str,
    pub(crate) timeout: Duration,
}

impl<'a> WrapperTermRequestBuilder<'a> {
    pub fn new_request(wrapper: &'a WebRegWrapper) -> Self {
        Self {
            cookies: &wrapper.cookies,
            client: &wrapper.client,
            term: &wrapper.term,
            user_agent: &wrapper.user_agent,
            timeout: wrapper.default_timeout,
        }
    }

    pub fn override_cookies(mut self, cookies: &'a str) -> Self {
        self.cookies = cookies;
        self
    }

    pub fn override_client(mut self, client: &'a Client) -> Self {
        self.client = client;
        self
    }

    pub fn override_term(mut self, term: &'a str) -> Self {
        self.term = term;
        self
    }

    pub fn override_user_agent(mut self, user_agent: &'a str) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn override_timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn build_term_raw(self) -> WrapperTermRawRequest<'a> {
        WrapperTermRawRequest { info: self }
    }

    pub fn build_term_parser(self) -> WrapperTermRequest<'a> {
        WrapperTermRequest {
            raw: self.build_term_raw(),
        }
    }
}
