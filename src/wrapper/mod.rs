use crate::raw_types::RawTermListItem;
use crate::types::{Term, WrapperError};
use crate::util::get_term_seq_id;
use crate::wrapper::constants::*;
use crate::wrapper::req_term_builder::{WrapperTermRequest, WrapperTermRequestBuilder};
use crate::wrapper::ww_helper::process_get_result;
use crate::{types, util};
use reqwest::header::{COOKIE, USER_AGENT};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use url::Url;

mod constants;
mod req_term_builder;
pub mod search;
pub mod wrapper_builder;
mod ww_helper;
pub mod ww_parser;

/// A wrapper for [UCSD's WebReg](https://act.ucsd.edu/webreg2/start). For more information,
/// please see the README.
pub struct WebRegWrapper {
    cookies: String,
    client: Client,
    term: String,
    user_agent: String,
    default_timeout: Duration,
}

impl WebRegWrapper {
    /// Creates a new instance of the `WebRegWrapper` with the specified `Client`, cookies, and
    /// default term. A default timeout and user agent will be provided. To override these, use
    /// [`WrapperBuilder`].
    ///
    /// After providing your cookies, you should ensure that each term is "bound" to your cookies.
    /// This can be done in several ways:
    /// - Calling `associate_term` with the specified term you want to use.
    /// - Calling `register_all_terms` to bind all terms to your cookie.
    /// - Manually selecting a term from WebReg (this is effectively what `associate_term` does).
    ///
    /// You are expected to provide a
    /// [`reqwest::Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html). This
    /// can be as simple as the default client (`Client::new()`), or can be customized to suit
    /// your needs. Note that the timeout set on the `Client` will be ignored in favor of the
    /// `timeout` field here.
    ///
    /// # Parameters
    /// - `client`: The `reqwest` client. You are able to override this on a per-request basis.
    /// - `cookies`: The cookies from your session of WebReg. You are able to override this on
    ///              a per-request basis.
    /// - `term`: The term. You are able to override this on a per-request basis.
    ///
    /// # Returns
    /// The new instance of the `WebRegWrapper`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// let client = Client::new();
    /// let wrapper = WebRegWrapper::new(client, "my cookies".to_string(), "FA22");
    /// ```
    pub fn new(client: Client, cookies: impl Into<String>, term: impl Into<String>) -> Self {
        Self {
            cookies: cookies.into(),
            client,
            term: term.into(),
            default_timeout: Duration::from_secs(30),
            user_agent: MY_USER_AGENT.to_owned(),
        }
    }

    /// Sets the cookies to the new, specified cookies.
    ///
    /// This might be useful if you want to use the existing wrapper but need to change the
    /// cookies.
    ///
    /// As a warning, this does NOT change the internal `term`. If this is something that
    /// needs to be changed, either use `set_term` method.
    ///
    /// # Parameters
    /// - `new_cookies`: The new cookies.
    pub fn set_cookies(&mut self, new_cookies: impl Into<String>) {
        self.cookies = new_cookies.into();
    }

    /// Sets the default term to the new, specified term.
    ///
    /// # Parameters
    /// - `new_term`: The term to use.
    pub fn set_term(&mut self, new_term: impl Into<String>) {
        self.term = new_term.into();
    }

    /// Checks if the current WebReg instance is valid. Specifically, this will check if you
    /// are logged in.
    ///
    /// # Returns
    /// `true` if the instance is valid and `false` otherwise.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// assert!(wrapper.is_valid().await);
    /// # }
    /// ```
    pub async fn is_valid(&self) -> bool {
        self.ping_server().await
    }

    /// Gets the name of the owner associated with this account.
    ///
    /// # Returns
    /// The name of the person, or an empty string if the cookies that were given were invalid.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// assert_eq!("Your name here", wrapper.get_account_name().await.unwrap());
    /// # }
    /// ```
    pub async fn get_account_name(&self) -> types::Result<String> {
        if !self.is_valid().await {
            return Err(WrapperError::GeneralError("Could not get name.".into()));
        }

        Ok(self
            .client
            .get(ACC_NAME)
            .header(COOKIE, &self.cookies)
            .header(USER_AGENT, MY_USER_AGENT)
            .send()
            .await?
            .text()
            .await?)
    }

    /// Registers all terms to your current session so that you can freely
    /// access any terms using this wrapper.
    ///
    /// By default, when you provide brand new WebReg cookies, it won't be
    /// associated with any terms. In order to actually use your cookies to
    /// make requests, you need to tell WebReg that you want to "associate"
    /// your cookies with a particular term.
    ///
    /// # Returns
    /// A result, where nothing is returned if everything went well and an
    /// error is returned if something went wrong.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// assert!(wrapper.register_all_terms().await.is_ok());
    /// # }
    /// ```
    pub async fn register_all_terms(&self) -> types::Result<()> {
        let terms = self.get_all_terms().await?;
        for term in terms {
            self.associate_term(term.term_code).await?;
        }

        Ok(())
    }

    /// Gets all terms available on WebReg.
    ///
    /// # Returns
    /// A vector of term objects, with each object containing the term name and
    /// term ID. If an error occurs, you will get that instead.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA22");
    /// assert!(wrapper.get_all_terms().await.unwrap().len() > 0);
    /// # }
    /// ```
    pub async fn get_all_terms(&self) -> types::Result<Vec<Term>> {
        let url = Url::parse_with_params(
            TERM_LIST,
            &[("_", util::get_epoch_time().to_string().as_str())],
        )?;

        process_get_result::<Vec<RawTermListItem>>(
            self.client
                .get(url)
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await
        .map(|raw_term_list| {
            raw_term_list
                .into_iter()
                .map(
                    |RawTermListItem {
                         seq_id, term_code, ..
                     }| Term { seq_id, term_code },
                )
                .collect()
        })
    }

    /// Associates a particular term to this current instance of the wrapper.
    ///
    /// After calling this function, you should be able to make requests to
    /// WebReg with the specified term.
    ///
    /// Note that WebReg doesn't actually do any validation with your input,
    /// so you should ensure that the term you want to use is actually valid.
    ///
    /// # Parameters
    /// - `term`: The term to associate with your session cookies.
    ///
    /// # Returns
    /// A result, where nothing is returned if everything went well and an
    /// error is returned if something went wrong.
    ///
    /// # Example
    /// ```rust,no_run
    /// use reqwest::Client;
    /// use webweg::wrapper::WebRegWrapper;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string(), "FA23");
    /// // Associate this wrapper with S123, S223, FA23.
    /// wrapper.associate_term("S123").await;
    /// wrapper.associate_term("S223").await;
    /// wrapper.associate_term("FA23").await;
    /// // We should now be able to use those three terms.
    /// # }
    /// ```
    pub async fn associate_term(&self, term: impl AsRef<str>) -> types::Result<()> {
        let term = term.as_ref().to_uppercase();
        let seq_id = get_term_seq_id(&term);
        if seq_id == 0 {
            return Err(WrapperError::InputError("term", "term is not valid."));
        }

        let seqid_str = seq_id.to_string();
        // Step 1: call get_status_start endpoint
        let status_start_url = Url::parse_with_params(
            STATUS_START,
            &[
                ("termcode", term.as_str()),
                ("seqid", seqid_str.as_str()),
                ("_", util::get_epoch_time().to_string().as_str()),
            ],
        )?;

        process_get_result::<Value>(
            self.client
                .get(status_start_url)
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await?;

        // Step 2: call eligibility endpoint
        let eligibility_url = Url::parse_with_params(
            ELIGIBILITY,
            &[
                ("termcode", term.as_str()),
                ("seqid", seqid_str.as_str()),
                ("logged", "true"),
                ("_", util::get_epoch_time().to_string().as_str()),
            ],
        )?;

        process_get_result::<Value>(
            self.client
                .get(eligibility_url)
                .header(COOKIE, &self.cookies)
                .header(USER_AGENT, MY_USER_AGENT)
                .send()
                .await,
        )
        .await?;

        Ok(())
    }

    /// Pings the WebReg server. Presumably, this is the endpoint that is used to ensure that
    /// your (authenticated) session is still valid. In other words, if this isn't called, I
    /// assume that you will be logged out, rendering your cookies invalid.
    ///
    /// # Returns
    /// `true` if the ping was successful and `false` otherwise.
    pub async fn ping_server(&self) -> bool {
        let res = self
            .client
            .get(format!("{}?_={}", PING_SERVER, util::get_epoch_time()))
            .header(COOKIE, &self.cookies)
            .header(USER_AGENT, MY_USER_AGENT)
            .send()
            .await;

        if let Ok(r) = res {
            let text = r.text().await.unwrap_or_else(|_| {
                json!({
                    "SESSION_OK": false
                })
                .to_string()
            });

            let json: Value = serde_json::from_str(&text).unwrap_or_default();
            // Use of unwrap here is safe since we know that there is a boolean value beforehand
            json["SESSION_OK"].is_boolean() && json["SESSION_OK"].as_bool().unwrap()
        } else {
            false
        }
    }

    /// Gets the current term.
    ///
    /// # Returns
    /// The current term.
    #[inline(always)]
    pub fn get_term(&self) -> &str {
        self.term.as_str()
    }

    /// Returns a request builder that can be used to customize any settings for a specific
    /// request only.
    ///
    /// # Returns
    /// A builder allowing you to customize any settings for your request, like the cookies,
    /// client, term, user agent, and timeout.
    pub fn make_request(&self) -> WrapperTermRequestBuilder {
        WrapperTermRequestBuilder::new_request(self)
    }

    /// Returns the requester that can be used to make request sot WebReg.
    ///
    /// # Returns
    /// The requester.
    pub fn default_request(&self) -> WrapperTermRequest {
        WrapperTermRequestBuilder::new_request(self).finish_building()
    }
}
