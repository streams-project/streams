//! The way in: the landing page, sign in, joining, and who is watching.

use contract::account::Profile;
use contract::core_msg::{Action, HttpMethod};
use leptos::prelude::*;
use serde_json::json;

use super::brand_name;
use crate::host;
use crate::state::Shared;

#[component]
pub fn Logo() -> impl IntoView {
    let shared = expect_context::<Shared>();
    view! {
        <a class="logo" href="/home">
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
                <button class="glass" on:click=move |_| host::go("/sign-in")>"Sign in"</button>
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
                                <button class="glass" on:click=move |_| host::go("/home")>"Browse"</button>
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
