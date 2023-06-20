use reqwest::{Error, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::types;
use crate::types::WrapperError;
use crate::wrapper::constants::VERIFY_FAIL_ERR;

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
    if !r.status().is_success() {
        return Err(WrapperError::BadStatusCode(r.status().as_u16()));
    }

    let text = r.text().await?;
    if text.contains(VERIFY_FAIL_ERR) {
        Err(WrapperError::WebRegError(
            "verification error; did you pick the wrong term?".into(),
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
    if !r.status().is_success() {
        return Err(WrapperError::BadStatusCode(r.status().as_u16()));
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
