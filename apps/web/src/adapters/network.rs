//! The network port, on fetch.

use std::collections::BTreeMap;

use contract::core_msg::{BodyTarget, EventBody, HttpMethod, NoAnswerReason};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, RequestInit, Response};

use crate::host::{answer, hand_reply, keep_page, step, window};

pub async fn perform(
    id: String,
    method: HttpMethod,
    path: String,
    headers: BTreeMap<String, String>,
    body: Option<String>,
    deadline_ms: u32,
    to: BodyTarget,
) {
    let carried_token = headers.contains_key("Authorization");
    let result = send(method, &path, &headers, body, deadline_ms).await;
    let event = match result {
        Err(reason) => {
            hand_reply(&id, 0, String::new());
            EventBody::HttpFailed { reason }
        }
        Ok((status, etag, text)) => {
            let mut headers = BTreeMap::new();
            if let Some(etag) = etag {
                headers.insert("etag".to_owned(), etag);
            }
            let good = (200..300).contains(&status);
            let to_screen = to == BodyTarget::Screen && good;
            if to_screen {
                keep_page(&path, &text);
            }
            // The core answers this 401 by refreshing and sending the same id again, so the reply
            // waits for that answer. Sending again from a screen would be a second request.
            if status != 401 || !carried_token {
                hand_reply(&id, status, text.clone());
            }
            EventBody::HttpDone {
                status,
                headers,
                body: (!to_screen && !text.is_empty()).then_some(text),
            }
        }
    };
    step(vec![answer(id, event)]);
}

/// A request's deadline, held for as long as the request is. It is dropped only when the whole
/// call is over, body and all, so the deadline the core asked for covers reading the answer: a
/// server that sends headers at once and then stalls used to hold a call open for ever.
struct Deadline(Option<i32>);

impl Deadline {
    fn stops(ms: u32, abort: &AbortController) -> Self {
        let stop = abort.clone();
        let fire = Closure::once_into_js(move || stop.abort());
        Self(
            window()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    fire.unchecked_ref(),
                    ms as i32,
                )
                .ok(),
        )
    }
}

impl Drop for Deadline {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            window().clear_timeout_with_handle(handle);
        }
    }
}

fn method_word(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        _ => "GET",
    }
}

async fn send(
    method: HttpMethod,
    path: &str,
    headers: &BTreeMap<String, String>,
    body: Option<String>,
    deadline_ms: u32,
) -> Result<(u32, Option<String>, String), NoAnswerReason> {
    let abort = AbortController::new().map_err(|_| NoAnswerReason::Offline)?;
    let init = RequestInit::new();
    init.set_method(method_word(method));
    init.set_signal(Some(&abort.signal()));
    let list = web_sys::Headers::new().map_err(|_| NoAnswerReason::Offline)?;
    for (name, value) in headers {
        let _ = list.set(name, value);
    }
    if let Some(body) = body {
        let _ = list.set("content-type", "application/json");
        init.set_body(&body.into());
    }
    init.set_headers(&list);
    let _deadline = Deadline::stops(deadline_ms, &abort);
    let answer = JsFuture::from(window().fetch_with_str_and_init(path, &init)).await;
    let response: Response = answer
        .map_err(|_| NoAnswerReason::Offline)?
        .dyn_into()
        .map_err(|_| NoAnswerReason::Offline)?;
    let status = u32::from(response.status());
    let etag = response.headers().get("etag").ok().flatten();
    let reading = response.text().map_err(|_| NoAnswerReason::Offline)?;
    let text = JsFuture::from(reading)
        .await
        .map_err(|_| NoAnswerReason::Timeout)?
        .as_string()
        .unwrap_or_default();
    Ok((status, etag, text))
}
