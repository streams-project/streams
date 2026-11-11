//! The network port, on reqwest over rustls.

use std::time::Duration;

use contract::core_msg::{HttpMethod, NoAnswerReason};
use reqwest::{Client, Method};
use tokio::runtime::Handle;

use crate::ports::{Answer, Call, Done, ETAG, Network};

pub struct HttpNetwork {
    client: Client,
    runtime: Handle,
}

impl HttpNetwork {
    /// A client on this runtime.
    ///
    /// # Errors
    /// The TLS backend of this machine could not be made ready.
    pub fn new(runtime: Handle) -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: Client::builder().build()?,
            runtime,
        })
    }
}

/// The method word reqwest wants. Anything this build does not know reads as a GET, which is the
/// one method that changes nothing.
fn method_of(method: HttpMethod) -> Method {
    match method {
        HttpMethod::Post => Method::POST,
        HttpMethod::Put => Method::PUT,
        HttpMethod::Patch => Method::PATCH,
        HttpMethod::Delete => Method::DELETE,
        _ => Method::GET,
    }
}

/// Why no answer came. reqwest 0.12 tells a timeout and a failure to connect apart and says no
/// more, so `Tls` and `ClockWrong` are never reported from here: a certificate a machine refuses
/// arrives as `Offline`. Naming it wrongly would be worse than naming it broadly.
fn reason_of(error: &reqwest::Error) -> NoAnswerReason {
    if error.is_timeout() {
        NoAnswerReason::Timeout
    } else {
        NoAnswerReason::Offline
    }
}

impl Network for HttpNetwork {
    fn send(&self, call: Call, done: Done) {
        let client = self.client.clone();
        self.runtime.spawn(async move {
            let Call {
                method,
                url,
                headers,
                body,
                deadline_ms,
            } = call;
            // The deadline covers reading the answer as well as reaching the server: a server that
            // sends its headers at once and then stalls would otherwise hold a call open for ever.
            let mut request = client
                .request(method_of(method), url)
                .timeout(Duration::from_millis(u64::from(deadline_ms)));
            for (name, value) in &headers {
                request = request.header(name, value);
            }
            if let Some(body) = body {
                request = request
                    .header("content-type", "application/json")
                    .body(body);
            }
            let answer = match request.send().await {
                Err(error) => Answer::NoReply {
                    reason: reason_of(&error),
                },
                Ok(reply) => {
                    let status = u32::from(reply.status().as_u16());
                    let etag = reply
                        .headers()
                        .get(ETAG)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_owned);
                    match reply.text().await {
                        Ok(text) => Answer::Reply { status, etag, text },
                        // The status reached this host and the body did not, which is not an
                        // answer: half an answer read as one would be kept as a page.
                        Err(error) => Answer::NoReply {
                            reason: reason_of(&error),
                        },
                    }
                }
            };
            done(answer);
        });
    }

    /// Nothing here watches the connection yet. Every path a desktop has for it is an OS call
    /// behind another dependency, so this says nothing rather than guessing, and a caller that
    /// learns of a change elsewhere can still hand the core an `EventBody::Network` event itself.
    fn watch(&self, _changed: Box<dyn Fn(bool) + Send + Sync + 'static>) {}
}
