#[cfg(feature = "multi")]
use parking_lot::Mutex;
use std::time::Duration;

use reqwest::Client;
use serde_json::{json, Value};
use url::Url;

use crate::constants::*;
use crate::raw_types::RawTermListItem;
use crate::types::{Term, WrapperError};
use crate::util::get_term_seq_id;
use crate::wrapper::request_builder::WrapperTermRequestBuilder;
use crate::wrapper::request_data::{ReqType, ReqwestWebRegClientData, WebRegWrapperData};
use crate::wrapper::wrapper_builder::WebRegWrapperBuilder;
use crate::wrapper::ww_helper::process_get_result;
use crate::{types, util};

pub mod input_types;
pub mod request_builder;
mod request_data;
pub mod requester_term;
pub mod wrapper_builder;
mod ww_helper;

/// A wrapper for [UCSD's WebReg](https://act.ucsd.edu/webreg2/start). For more information,
/// please see the README.
pub struct WebRegWrapper {
    data: WebRegWrapperData,
}

impl<'a> WebRegWrapper {
    /// Creates a new instance of the `WebRegWrapper` with the specified `Client` and session
    /// cookies. A default timeout and user agent will be provided. To override these, use
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
    /// let wrapper = WebRegWrapper::new(client, "my cookies".to_string());
    /// ```
    pub fn new(client: Client, cookies: impl Into<String>) -> Self {
        Self {
            data: WebRegWrapperData {
                #[cfg(feature = "multi")]
                cookies: Mutex::new(cookies.into()),
                #[cfg(not(feature = "multi"))]
                cookies: cookies.into(),
                client,
                timeout: Duration::from_secs(30),
                user_agent: MY_USER_AGENT.to_owned(),
                close_after_request: false,
            },
        }
    }

    /// Creates a new builder that can be used to construct a `WebRegWrapper`. This is the
    /// preferred method for creating a wrapper for more complex situations.
    ///
    /// # Returns
    /// The builder.
    pub fn builder() -> WebRegWrapperBuilder {
        WebRegWrapperBuilder::new()
    }

    /// Sets the cookies to the new, specified cookies.
    ///
    /// This might be useful if you want to use the existing wrapper but need to change the
    /// cookies.
    ///
    /// # Parameters
    /// - `new_cookies`: The new cookies.
    #[cfg(not(feature = "multi"))]
    pub fn set_cookies(&mut self, new_cookies: impl Into<String>) {
        self.data.cookies = new_cookies.into();
    }

    /// Sets the cookies to the new, specified cookies.
    ///
    /// This might be useful if you want to use the existing wrapper but need to change the
    /// cookies.
    ///
    /// Note that a mutex is internally used to store the cookies.
    ///
    /// # Parameters
    /// - `new_cookies`: The new cookies.
    #[cfg(feature = "multi")]
    pub fn set_cookies(&self, new_cookies: impl Into<String>) {
        let mut cookies = self.data.cookies.lock();
        *cookies = new_cookies.into();
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
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string());
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
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string());
    /// assert_eq!("Your name here", wrapper.get_account_name().await.unwrap());
    /// # }
    /// ```
    pub async fn get_account_name(&self) -> types::Result<String> {
        if !self.is_valid().await {
            return Err(WrapperError::SessionNotValid);
        }

        Ok(self
            .data
            .req(ReqType::Get(ACC_NAME))
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
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string());
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
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string());
    /// assert!(wrapper.get_all_terms().await.unwrap().len() > 0);
    /// # }
    /// ```
    pub async fn get_all_terms(&self) -> types::Result<Vec<Term>> {
        let url = Url::parse_with_params(
            TERM_LIST,
            &[("_", util::get_epoch_time().to_string().as_str())],
        )?;

        process_get_result::<Vec<RawTermListItem>>(self.data.req(ReqType::Get(url)).send().await)
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
    /// let wrapper = WebRegWrapper::new(Client::new(), "my cookies".to_string());
    /// // Associate this wrapper with S123, S223, FA23.
    /// _ = wrapper.associate_term("S123").await;
    /// _ = wrapper.associate_term("S223").await;
    /// _ = wrapper.associate_term("FA23").await;
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

        process_get_result::<Value>(self.data.req(ReqType::Get(status_start_url)).send().await)
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

        process_get_result::<Value>(self.data.req(ReqType::Get(eligibility_url)).send().await)
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
            .data
            .req(ReqType::Get(format!(
                "{}?_={}",
                PING_SERVER,
                util::get_epoch_time()
            )))
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

    /// Returns a request builder that can be used to customize any settings for a specific
    /// request only.
    ///
    /// # Parameters
    /// - `term`: The term to use for this request.
    ///
    /// # Returns
    /// A builder allowing you to customize any settings for your request, like the cookies,
    /// client, term, user agent, and timeout.
    pub fn req(&'a self, term: &'a str) -> WrapperTermRequestBuilder {
        WrapperTermRequestBuilder::new_request(&self.data, term)
    }
}
