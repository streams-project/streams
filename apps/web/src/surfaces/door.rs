//! The way in: the landing page, sign in, joining, and who is watching.

use contract::account::Profile;
use contract::address;
use contract::core_msg::{Action, HttpMethod};
use contract::error::ErrorBody;
use leptos::prelude::*;
use serde_json::json;

use super::{brand_name, label_text};
use crate::host;
use crate::state::Shared;

#[component]
pub fn Logo() -> impl IntoView {
    let shared = expect_context::<Shared>();
    view! {
        <a class="logo" href=address::HOME>
            <img src="/lockup.svg" alt=move || brand_name(shared)/>
        </a>
    }
}

#[component]
pub fn Landing() -> impl IntoView {
    let joining = RwSignal::new(false);
    let visitors = expect_context::<Shared>();
    let open = move || {
        visitors.hello.with(|hello| {
            hello
                .as_ref()
                .is_some_and(|hello| !hello.features.accounts_required)
        })
    };
    view! {
        <div class="door">
            <header class="door-top">
                <Logo/>
                <button class="glass" on:click=move |_| host::go(address::SIGN_IN)>"Sign in"</button>
            </header>
            <main class="door-card">
                {move || if joining.get() {
                    view! { <Join/> }.into_any()
                } else {
                    view! {
                        <h1>"Films and series, in one place."</h1>
                        <p>"Sign up and start watching."</p>
                        <div class="row-of-buttons">
                            <button class="join" on:click=move |_| joining.set(true)>"Get started"</button>
                            {move || open().then(|| view! {
                                <button class="glass" on:click=move |_| host::go(address::HOME)>"Browse"</button>
                            })}
                        </div>
                    }.into_any()
                }}
            </main>
        </div>
    }
}

#[component]
pub fn SignIn() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());
    let sent = RwSignal::new(false);
    let joining = RwSignal::new(false);
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if sent.get() {
            host::act(Action::SignInCode {
                email: email.get(),
                code: code.get(),
            });
        } else {
            host::act(Action::SignIn {
                email: email.get(),
                password: password.get(),
            });
        }
    };
    let send_code = move |_| {
        sent.set(true);
        host::act(Action::SendCode { email: email.get() });
    };
    view! {
        <div class="door">
            <header class="door-top"><Logo/></header>
            <main class="door-card">
                {move || if joining.get() {
                    view! { <Join/> }.into_any()
                } else {
                    view! {
                        <h1>"Sign in"</h1>
                        <form on:submit=submit>
                            <input type="email" required placeholder="Email" autocomplete="username" bind:value=email/>
                            {move || if sent.get() {
                                view! { <input inputmode="numeric" required placeholder="Code from your email" autocomplete="one-time-code" bind:value=code/> }.into_any()
                            } else {
                                view! { <input type="password" required placeholder="Password" autocomplete="current-password" bind:value=password/> }.into_any()
                            }}
                            <button class="solid" type="submit">"Sign in"</button>
                        </form>
                        <div class="door-links">
                            <button class="plain" on:click=send_code>"Email me a code instead"</button>
                            <button class="plain" on:click=move |_| joining.set(true)>"New here? Create an account"</button>
                        </div>
                    }.into_any()
                }}
            </main>
        </div>
    }
}

#[component]
fn Join() -> impl IntoView {
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let trouble = RwSignal::new(None::<&'static str>);
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        trouble.set(None);
        let (address, secret) = (email.get(), password.get());
        let body = json!({
            "email": address, "password": secret, "first_name": first.get(), "last_name": last.get(),
        });
        host::ask(
            HttpMethod::Post,
            "/v1/auth/signup",
            Some(body.to_string()),
            move |status, _| {
                if (200..300).contains(&status) {
                    host::act(Action::SignIn {
                        email: address,
                        password: secret,
                    });
                } else if status == 503 {
                    trouble.set(Some("Joining is not available right now."));
                } else {
                    trouble.set(Some(
                        "That did not work. Check the details, or sign in if you already joined.",
                    ));
                }
            },
        );
    };
    view! {
        <h1>"Create your account"</h1>
        <form on:submit=submit>
            <input required placeholder="First name" autocomplete="given-name" bind:value=first/>
            <input required placeholder="Last name" autocomplete="family-name" bind:value=last/>
            <input type="email" required placeholder="Email" autocomplete="username" bind:value=email/>
            <input type="password" required minlength="8" placeholder="Password" autocomplete="new-password" bind:value=password/>
            {move || trouble.get().map(|text| view! { <p class="trouble">{text}</p> })}
            <button class="join" type="submit">"Get started"</button>
        </form>
    }
}

/// A code as the server keeps it. A screen may group it, and people type it with spaces, hyphens
/// and small letters.
fn plain_code(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_uppercase()
}

/// The label for an answer that is not a yes.
fn refused_label(status: u32, body: &str) -> String {
    match status {
        0 => "problem-offline".to_owned(),
        404 => "error.code".to_owned(),
        _ => serde_json::from_str::<ErrorBody>(body)
            .map_or_else(|_| "problem-failed".to_owned(), |error| error.label),
    }
}

/// How the person answered a device.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Answered {
    Approved,
    Denied,
}

/// Approve or refuse a device that shows a code. The core shows this page only to a signed in
/// person, so a session that is still starting never reaches these buttons.
#[component]
pub fn Activate() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let code = RwSignal::new(String::new());
    let sending = RwSignal::new(false);
    let answered = RwSignal::new(None::<Answered>);
    let trouble = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        let held = shared
            .route
            .with(|route| route.as_ref().and_then(|route| route.code.clone()));
        if let Some(held) = held {
            code.set(plain_code(&held));
        }
    });
    let answer = move |path: &'static str, when_done: Answered| {
        let user_code = plain_code(&code.get_untracked());
        if user_code.is_empty() {
            trouble.set(Some(label_text(shared, "error.code")));
            return;
        }
        sending.set(true);
        trouble.set(None);
        let body = json!({ "user_code": user_code }).to_string();
        host::ask(
            HttpMethod::Post,
            path,
            Some(body),
            move |status, text| match status {
                200..300 => answered.set(Some(when_done)),
                // No token went out, and the core is on its way to sign in with this address
                // kept. The code is still good, so this is no refusal, and the answer is pressed
                // again from here.
                401 => sending.set(false),
                _ => {
                    sending.set(false);
                    trouble.set(Some(label_text(shared, &refused_label(status, &text))));
                }
            },
        );
    };
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        answer("/v1/auth/device/approve", Answered::Approved);
    };
    view! {
        <div class="door">
            <header class="door-top"><Logo/></header>
            <main class="door-card">
                {move || match answered.get() {
                    Some(Answered::Approved) => view! {
                        <h1>"Your device is signing in"</h1>
                        <p>"You can close this page."</p>
                    }.into_any(),
                    Some(Answered::Denied) => view! {
                        <h1>"That device stays signed out"</h1>
                        <p>"You can close this page."</p>
                    }.into_any(),
                    None => view! {
                        <h1>"Sign in on your device"</h1>
                        <form on:submit=submit>
                            <input required placeholder="Code on your screen" aria-label="Code on your screen" autocomplete="off" autocapitalize="characters" spellcheck="false" bind:value=code/>
                            {move || trouble.get().map(|text| view! { <p class="trouble">{text}</p> })}
                            <div class="row-of-buttons">
                                <button class="solid" type="submit" disabled=move || sending.get()>"Approve"</button>
                                <button class="glass" type="button" disabled=move || sending.get()
                                    on:click=move |_| answer("/v1/auth/device/deny", Answered::Denied)>"Not me"</button>
                            </div>
                        </form>
                    }.into_any(),
                }}
            </main>
        </div>
    }
}

#[component]
pub fn Profiles() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let pin_for = RwSignal::new(None::<String>);
    let pin = RwSignal::new(String::new());
    let enter = move |profile: &Profile| {
        if profile.has_pin {
            pin_for.set(Some(profile.id.clone()));
        } else {
            host::act(Action::EnterProfile {
                profile: profile.id.clone(),
                pin: None,
            });
        }
    };
    let submit_pin = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if let Some(profile) = pin_for.get() {
            host::act(Action::EnterProfile {
                profile,
                pin: Some(pin.get()),
            });
        }
    };
    view! {
        <div class="gate">
            <h1>"Who's watching?"</h1>
            <div class="people">
                {move || shared.profiles.get().into_iter().map(|profile| {
                    let chosen = profile.clone();
                    let letter = profile.name.chars().next().unwrap_or('?').to_string();
                    view! {
                        <button class="person" on:click=move |_| enter(&chosen)>
                            <span class="face" class:kids=profile.kids>{letter}</span>
                            <span>{profile.name.clone()}</span>
                        </button>
                    }
                }).collect_view()}
            </div>
            {move || pin_for.get().map(|_| view! {
                <form class="pin" on:submit=submit_pin>
                    <input inputmode="numeric" maxlength="4" required placeholder="PIN" bind:value=pin/>
                    <button class="solid" type="submit">"Enter"</button>
                </form>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{plain_code, refused_label};

    #[test]
    fn a_code_loses_spaces_hyphens_and_small_letters() {
        for typed in [
            "BCDFGHJK",
            "bcdf-ghjk",
            " BCDF GHJK ",
            "bcdf\u{a0}- ghjk",
            "BCDF+GHJK",
        ] {
            assert_eq!(plain_code(typed), "BCDFGHJK", "{typed}");
        }
        assert_eq!(plain_code(" - "), "");
    }

    #[test]
    fn a_refusal_names_what_a_person_can_do() {
        assert_eq!(refused_label(0, ""), "problem-offline");
        assert_eq!(refused_label(404, ""), "error.code");
        let decided = r#"{"code":"refused","label":"error.device_decided","build":""}"#;
        assert_eq!(refused_label(403, decided), "error.device_decided");
        let busy =
            r#"{"code":"rate_limited","label":"error.rate_limited","build":"","retry_after_s":60}"#;
        assert_eq!(refused_label(429, busy), "error.rate_limited");
        assert_eq!(refused_label(500, "not json"), "problem-failed");
    }
}
