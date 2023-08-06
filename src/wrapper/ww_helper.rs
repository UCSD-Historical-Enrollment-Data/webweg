use reqwest::{Error, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

use crate::constants::{ELIGIBILITY, STATUS_START, VERIFY_FAIL_ERR};
use crate::types::WrapperError;
use crate::util::get_term_seq_id;
use crate::wrapper::ReqwestClientWrapper;
use crate::{types, util};

/// Extracts text from the given response, handling the possibility that a bad status code
/// or a verification error occurs.
///
/// # Parameters
/// - `res`: The initial response.
///
/// # Returns
/// The result of processing the response.
pub(crate) async fn extract_text(res: Result<Response, Error>) -> types::Result<String> {
    let r = res?;
    let status_code = r.status();
    if !status_code.is_success() {
        let text = r.text().await.ok();
        return Err(WrapperError::BadStatusCode(status_code.as_u16(), text));
    }

    let text = r.text().await?;
    if text.contains(VERIFY_FAIL_ERR) {
        Err(WrapperError::WebRegError(
            "Verification error: register your term using the `associate_term` function.".into(),
        ))
    } else {
        Ok(text)
    }
}

/// Processes a GET response from the resulting text representing JSON, if any.
///
/// # Parameters
/// - `res`: The string containing JSON to convert.
///
/// # Returns
/// The result of converting the JSON to the deserialized form.
pub(crate) fn process_get_text<T: DeserializeOwned>(res: String) -> types::Result<T> {
    serde_json::from_str::<T>(&res).map_err(WrapperError::SerdeError)
}

/// Processes a GET response from the resulting JSON, if any. This is equivalent to calling
/// - `extract_text`, followed by
/// - `process_get_text`.
///
/// # Parameters
/// - `res`: The initial response.
///
/// # Returns
/// The result of processing the response.
pub(crate) async fn process_get_result<T: DeserializeOwned>(
    res: Result<Response, Error>,
) -> types::Result<T> {
    let r = extract_text(res).await?;
    process_get_text(r)
}

/// Processes a POST response from the resulting JSON, if any.
///
/// # Parameters
/// - `res`: The initial response.
///
/// # Returns
/// Either one of:
/// - `true` or `false`, depending on what WebReg returns.
/// - or some error message if an error occurred.
pub(crate) async fn process_post_response(res: Result<Response, Error>) -> types::Result<bool> {
    let r = res?;
    let status_code = r.status();
    if !status_code.is_success() {
        let text = r.text().await.ok();
        return Err(WrapperError::BadStatusCode(status_code.as_u16(), text));
    }

    let text = r.text().await?;
    // Unwrap should not be a problem since we should be getting a valid JSON response
    // every time.
    let json: Value = serde_json::from_str(&text)?;
    if json["OPS"].is_string() && json["OPS"].as_str().unwrap() == "SUCCESS" {
        return Ok(true);
    }

    // Purely to handle an error
    let mut parsed_str = String::new();
    let mut is_in_brace = false;
    json["REASON"]
        .as_str()
        .unwrap_or("")
        .trim()
        .chars()
        .for_each(|c| {
            if c == '<' {
                is_in_brace = true;
                return;
            }

            if c == '>' {
                is_in_brace = false;
                return;
            }

            if is_in_brace {
                return;
            }

            parsed_str.push(c);
        });

    Err(WrapperError::WebRegError(parsed_str))
}

/// Associates a particular term to an instance that implements the `ReqwestClientWrapper`
/// trait. Useful for generalizing by different types of requests.
///
/// # Parameters
/// - `obj`: A reference to an object implementing the `ReqwestClientWrapper` trait.
/// - `term`: The term to associate with your session cookies.
///
/// # Returns
/// A result, where nothing is returned if everything went well and an
/// error is returned if something went wrong.
pub(crate) async fn associate_term_helper<'a>(
    obj: &'a impl ReqwestClientWrapper<'a>,
    term: impl AsRef<str>,
) -> types::Result<()> {
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

    process_get_result::<Value>(obj.req_get(status_start_url).send().await).await?;

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

    process_get_result::<Value>(obj.req_get(eligibility_url).send().await).await?;

    Ok(())
}
