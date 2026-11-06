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
    let stop = abort.clone();
    let timeout = Closure::once_into_js(move || stop.abort());
    let handle = window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            timeout.unchecked_ref(),
            deadline_ms as i32,
        )
        .ok();
    let answer = JsFuture::from(window().fetch_with_str_and_init(path, &init)).await;
    if let Some(handle) = handle {
        window().clear_timeout_with_handle(handle);
    }
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
