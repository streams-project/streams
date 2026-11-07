//! The screens. The core says which one shows; each draws what `Shared` holds.

mod browse;
mod door;
mod title;
mod watch;

use contract::address;
use contract::core_msg::Target;
use contract::screen::{Image, ImageRole};
use leptos::prelude::*;

use crate::host;
use crate::state::Shared;

/// The first image of the first role that has one.
pub fn image(images: &[Image], roles: &[ImageRole]) -> Option<String> {
    roles.iter().find_map(|role| {
        images
            .iter()
            .find(|image| image.role == *role)
            .map(|image| image.url.clone())
    })
}

/// The brand's name, as the server states it.
pub fn brand_name(shared: Shared) -> String {
    shared
        .hello
        .with(|hello| hello.as_ref().map(|hello| hello.brand.name.clone()))
        .unwrap_or_default()
}

/// For a label with no text yet, so a notice is never blank.
const FALLBACK: &str = "That did not work. Try again.";

/// The text of a label, as the server states it.
pub fn label_text(shared: Shared, label: &str) -> String {
    let text = shared
        .strings
        .with(|strings| strings.get(label).cloned())
        .unwrap_or_else(|| FALLBACK.to_owned());
    shared
        .hello
        .with(|hello| hello.as_ref().map(|hello| hello.brand.fill(&text)))
        .unwrap_or(text)
}

#[component]
fn Message(text: &'static str) -> impl IntoView {
    view! { <div class="message"><p>{text}</p><a href=address::HOME>"Back to home"</a></div> }
}

#[component]
fn NoticeBar() -> impl IntoView {
    let shared = expect_context::<Shared>();
    move || {
        shared.notice.get().map(|notice| {
            let text = if notice.text.is_empty() {
                label_text(shared, &notice.label)
            } else {
                notice.text
            };
            view! {
                <div class="notice" role="status">
                    <span>{text}</span>
                    <button class="plain" on:click=move |_| shared.notice.set(None)>"Close"</button>
                </div>
            }
        })
    }
}

/// The page.
#[component]
pub fn App() -> impl IntoView {
    let shared = Shared::new();
    provide_context(shared);
    Effect::new(move |_| host::start(shared));
    let target = Memo::new(move |_| {
        shared
            .route
            .with(|route| route.as_ref().map(|route| route.target))
    });
    let screen = move || match target.get() {
        None | Some(Target::Wait) => {
            view! { <div class="wait"><div class="ring"></div></div> }.into_any()
        }
        Some(Target::Start) => view! { <door::Landing/> }.into_any(),
        Some(Target::SignIn) => view! { <door::SignIn/> }.into_any(),
        Some(Target::Profiles) => view! { <door::Profiles/> }.into_any(),
        Some(Target::Activate) => view! { <door::Activate/> }.into_any(),
        Some(Target::Watch) => view! { <watch::Watch/> }.into_any(),
        Some(Target::Title) => view! { <browse::Browse/><title::Details/> }.into_any(),
        Some(Target::NotFound) => view! { <Message text="That page is not here."/> }.into_any(),
        Some(Target::Upgrade) => {
            view! { <Message text="Please refresh to get the latest version."/> }.into_any()
        }
        Some(_) => view! { <browse::Browse/> }.into_any(),
    };
    view! {
        <NoticeBar/>
        {screen}
        {move || shared.gate.get().then(|| view! { <door::Profiles/> })}
    }
}
