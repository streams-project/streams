//! The way in: the landing page, sign in, joining, and who is watching.
//!
//! Every word here came from this file until 2026-11-12 - twenty-six English literals, which is
//! most of the English that was written into this crate. They come from `strings/` now, and where
//! `strings/` has no key for what a control says, the key that is wanted is named in a comment
//! beside it and the control takes the nearest key the server really has. Nothing here invents
//! English, and nothing here is silent.

use contract::account::Profile;
use contract::address;
use contract::core_msg::{Action, HttpMethod};
use contract::error::ErrorBody;
use leptos::prelude::*;
use serde_json::json;

use super::{brand_name, label_text, maybe_text};
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

/// A field with a label a reader can reach.
///
/// A placeholder is not a label: it is gone the moment anything is typed, no browser is obliged to
/// expose it, and a form of five placeholders is a form a person who cannot see it fills blind.
/// Every field on this page had one and nothing else before today. The label is drawn, visually
/// small, and tied to the field by `for`, which is what makes it the field's name to a reader and
/// what makes tapping it move the caret into the field.
#[component]
fn Field(
    id: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = "")] fills: &'static str,
    #[prop(default = false)] numeric: bool,
    #[prop(default = 0)] least: u32,
) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let words = move || label_text(shared, label);
    view! {
        <p class="field">
            <label class="field-name" for=id>{words}</label>
            <input
                id=id
                type=kind
                required
                aria-label=words
                autocomplete=(!fills.is_empty()).then_some(fills)
                inputmode=numeric.then_some("numeric")
                minlength=(least > 0).then(|| least.to_string())
                bind:value=value
            />
        </p>
    }
}

#[component]
pub fn Landing() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let joining = RwSignal::new(false);
    // Whether this server lets a visitor look before joining.
    let open = move || {
        shared.hello.with(|hello| {
            hello
                .as_ref()
                .is_some_and(|hello| !hello.features.accounts_required)
        })
    };
    // The phone's door is the brand's name and a way in, and nothing else
    // (`apps/android/surfaces/phone/.../door/Doors.kt:94-107`). What was here was two sentences of
    // English written into this file - a tagline and a line under it - for which `strings/` has no
    // key and no surface has an equivalent. They are gone rather than translated into eighteen
    // languages by an assistant.
    view! {
        <div class="door">
            <header class="door-top">
                <Logo/>
                <button class="glass" on:click=move |_| host::go(address::SIGN_IN)>
                    {move || label_text(shared, "door.sign_in")}
                </button>
            </header>
            <main class="door-card">
                {move || if joining.get() {
                    view! { <Join/> }.into_any()
                } else {
                    view! {
                        <h1>{label_text(shared, "door.watch")}</h1>
                        <div class="row-of-buttons">
                            <button class="join" on:click=move |_| joining.set(true)>
                                {label_text(shared, "door.join")}
                            </button>
                            {move || open().then(|| view! {
                                <button class="glass" on:click=move |_| host::go(address::HOME)>
                                    {label_text(shared, "nav.home")}
                                </button>
                            })}
                        </div>
                    }.into_any()
                }}
            </main>
        </div>
    }
}

/// The two keys the email-code way in needs, which no language file has yet.
///
/// `strings/en.json` carries 301 keys and none of them is the name of a code that arrives by email
/// or the words of the control that asks for one. The controls are drawn only when the server has
/// both, which is the rule this crate already follows for a screen whose whole point is its words
/// (`browse::NothingFound`), and it costs something real: the email-code way in is unreachable from
/// a browser until the keys exist. It was reachable before, labelled in English written into this
/// file. **This is in the report as the one feature a missing key takes away.**
const CODE_FIELD: &str = "signin.code";
const CODE_SEND: &str = "signin.code.send";

#[component]
pub fn SignIn() -> impl IntoView {
    let shared = expect_context::<Shared>();
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
                        <h1>{label_text(shared, "door.sign_in")}</h1>
                        <form on:submit=submit>
                            <Field id="signin-email" label="signin.email" kind="email"
                                fills="username" value=email/>
                            {move || if sent.get() {
                                view! {
                                    <Field id="signin-code" label=CODE_FIELD numeric=true
                                        fills="one-time-code" value=code/>
                                }.into_any()
                            } else {
                                view! {
                                    <Field id="signin-password" label="signin.password"
                                        kind="password" fills="current-password" value=password/>
                                }.into_any()
                            }}
                            <button class="solid" type="submit">
                                {label_text(shared, "door.sign_in")}
                            </button>
                        </form>
                        <div class="door-links">
                            {move || maybe_text(shared, CODE_SEND).map(|words| view! {
                                <button class="plain" on:click=send_code>{words}</button>
                            })}
                            <button class="plain" on:click=move |_| joining.set(true)>
                                {label_text(shared, "door.join")}
                            </button>
                        </div>
                    }.into_any()
                }}
            </main>
        </div>
    }
}

/// A person's name as the account route wants it: everything before the last space, and the last
/// word.
///
/// One field and not two. `strings/` has one key for a person's name (`settings.account_name`) and
/// none for a given name or a family name, and a form of two fields both labelled "Name" is worse
/// for a reader than one field labelled once. A name of one word has no family name, which the
/// route allows, and the extra spaces people really type are dropped.
fn name_parts(typed: &str) -> (String, String) {
    let tidy = typed.split_whitespace().collect::<Vec<_>>();
    match tidy.split_last() {
        Some((last, [])) => ((*last).to_owned(), String::new()),
        Some((last, first)) => (first.join(" "), (*last).to_owned()),
        None => (String::new(), String::new()),
    }
}

#[component]
fn Join() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    // The label of what went wrong, so the words come from the server like every other word. It was
    // two English sentences written into this file.
    let trouble = RwSignal::new(None::<&'static str>);
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        trouble.set(None);
        let (address, secret) = (email.get(), password.get());
        let (first, last) = name_parts(&name.get());
        let body = json!({
            "email": address, "password": secret, "first_name": first, "last_name": last,
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
                    trouble.set(Some("notice.off.accounts"));
                } else {
                    trouble.set(Some("error.sign_up"));
                }
            },
        );
    };
    view! {
        <h1>{move || label_text(shared, "door.join")}</h1>
        <form on:submit=submit>
            <Field id="join-name" label="settings.account_name" fills="name" value=name/>
            <Field id="join-email" label="signin.email" kind="email" fills="username" value=email/>
            <Field id="join-password" label="signin.password" kind="password"
                fills="new-password" least=8 value=password/>
            {move || trouble.get().map(|label| view! {
                <p class="trouble" role="alert">{label_text(shared, label)}</p>
            })}
            <button class="join" type="submit">{move || label_text(shared, "door.join")}</button>
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
///
/// This page is the phone or tablet half of a television's sign in, which is what `signin.step2`
/// names: "Confirm this code on your phone or tablet". So the heading is that key, and the one
/// field on the page is named by the heading rather than by words of its own, which is what
/// `aria-labelledby` is for. What it says when the answer is in has no key at all - **`activate.done`
/// and `activate.denied` are in the report** - so it falls back to the two words the two answers
/// are, `common.done` and `common.cancel`.
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
            trouble.set(Some(String::from("error.code")));
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
                    trouble.set(Some(refused_label(status, &text)));
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
                        <h1>{label_text(shared, "common.done")}</h1>
                    }.into_any(),
                    Some(Answered::Denied) => view! {
                        <h1>{label_text(shared, "common.cancel")}</h1>
                    }.into_any(),
                    None => view! {
                        <h1 id="activate-said">{label_text(shared, "signin.step2")}</h1>
                        <form on:submit=submit>
                            <input
                                aria-labelledby="activate-said"
                                required
                                autocomplete="off"
                                autocapitalize="characters"
                                spellcheck="false"
                                bind:value=code
                            />
                            {move || trouble.get().map(|label| view! {
                                <p class="trouble" role="alert">{label_text(shared, &label)}</p>
                            })}
                            <div class="row-of-buttons">
                                <button class="solid" type="submit" disabled=move || sending.get()>
                                    {label_text(shared, "common.done")}
                                </button>
                                <button class="glass" type="button" disabled=move || sending.get()
                                    on:click=move |_| answer("/v1/auth/device/deny", Answered::Denied)>
                                    {label_text(shared, "common.cancel")}
                                </button>
                            </div>
                        </form>
                    }.into_any(),
                }}
            </main>
        </div>
    }
}

/// Who is watching.
///
/// A profile is a picture or a letter and a name, and whether it is locked is a padlock nobody can
/// hear, so a locked profile says so from `say.profile_locked` as the phone's does
/// (`apps/android/surfaces/phone/.../profiles/GateGrid.kt`, through `says`).
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
            <h1>{move || label_text(shared, "profiles.choose")}</h1>
            <div class="people">
                {move || shared.profiles.get().into_iter().map(|profile| {
                    let chosen = profile.clone();
                    let letter = profile.name.chars().next().unwrap_or('?').to_string();
                    let says = if profile.has_pin {
                        super::words::filled(shared, "say.profile_locked", "name", &profile.name)
                            .unwrap_or_else(|| profile.name.clone())
                    } else {
                        profile.name.clone()
                    };
                    let kids = profile.kids;
                    view! {
                        <button class="person" aria-label=says on:click=move |_| enter(&chosen)>
                            // The letter stands in for a picture of the person, and the name is
                            // drawn under it, so it is a decoration.
                            <span class="face" class:kids=kids aria-hidden="true">{letter}</span>
                            <span>{profile.name.clone()}</span>
                            {kids.then(|| view! {
                                <span class="badge">{label_text(shared, "profile.badge.kids")}</span>
                            })}
                        </button>
                    }
                }).collect_view()}
            </div>
            {move || pin_for.get().map(|_| view! {
                <form class="pin" on:submit=submit_pin>
                    <label class="said" for="gate-pin">{label_text(shared, "say.pin")}</label>
                    <p class="pin-said">{label_text(shared, "profile.pin.enter")}</p>
                    <input id="gate-pin" inputmode="numeric" maxlength="4" required bind:value=pin/>
                    <button class="solid" type="submit">{label_text(shared, "common.done")}</button>
                </form>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{CODE_FIELD, CODE_SEND, name_parts, plain_code, refused_label};
    use crate::surfaces::tests_support::english;

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

    /// Every label a refusal can name is one the strings file carries, so a refused sign in always
    /// says something.
    #[test]
    fn every_label_a_refusal_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            refused_label(0, ""),
            refused_label(404, ""),
            refused_label(500, "not json"),
        ] {
            assert!(book.contains_key(&label), "strings/en.json has no {label}");
        }
    }

    #[test]
    fn one_name_field_fills_the_two_the_route_wants() {
        assert_eq!(
            name_parts("Ada Lovelace"),
            (String::from("Ada"), String::from("Lovelace"))
        );
        // A name of three words keeps the last as the family name and the rest as the given one.
        assert_eq!(
            name_parts("Jean Baptiste Lamarck"),
            (String::from("Jean Baptiste"), String::from("Lamarck"))
        );
        // A name of one word is a name, and has no family name.
        assert_eq!(
            name_parts("Prince"),
            (String::from("Prince"), String::new())
        );
        // The spaces people really type are dropped rather than carried into the account.
        assert_eq!(
            name_parts("  Ada   Lovelace \n"),
            (String::from("Ada"), String::from("Lovelace"))
        );
        assert_eq!(name_parts("   "), (String::new(), String::new()));
    }

    /// Every label this page names is in the strings file, and the two it names that are not are
    /// still not: filling one fails this, which is how the gap cannot outlive itself.
    #[test]
    fn every_label_this_page_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.cancel",
            "common.done",
            "door.join",
            "door.sign_in",
            "door.watch",
            "nav.home",
            "notice.off.accounts",
            "error.code",
            "error.sign_up",
            "profile.badge.kids",
            "profile.pin.enter",
            "profiles.choose",
            "say.pin",
            "say.profile_locked",
            "settings.account_name",
            "signin.email",
            "signin.password",
            "signin.step2",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
        for missing in [CODE_FIELD, CODE_SEND] {
            assert!(
                !book.contains_key(missing),
                "{missing} is in strings/en.json now: move it to the list above and the \
                 email-code way in starts working"
            );
        }
    }
}
