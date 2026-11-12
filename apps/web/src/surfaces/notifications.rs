//! The notices page: what has arrived, what is coming, and what a reminder was set for.
//!
//! A page of its own and not a page of rows. It reads `/v1/notifications`, which no screen route
//! builds, so until today a browser opening `/notifications` asked the server for a screen by that
//! name and drew whatever came back, which is nothing.
//!
//! The port of `apps/android/surfaces/phone/src/main/kotlin/streams/phone/notifications/Notifications.kt`,
//! behaviour for behaviour: one page read at a time through the feed's own cursor, the next part
//! read three items from the end (`Notifications.kt:100-115`), a notice marked read when it is
//! opened and the bell's count asked again (`:189-199`), the kind as a chip, an accented chip for
//! the two kinds that mean the title can be watched now (`:273-277`), and a dot for a notice not
//! read yet.

use contract::core_msg::HttpMethod;
use contract::notification::{NoticeItem, NoticeKind, Notifications};
use contract::screen::ImageRole;
use leptos::prelude::*;
use serde_json::json;

use super::words::{filled, in_digits};
use super::{image, label_text, maybe_text, title_address};
use crate::host;
use crate::ports::address;
use crate::state::Shared;

/// How near the end of the feed the next part is read. `Notifications.kt:105`.
const READ_AHEAD: usize = 3;

/// The label for a kind of notice. The two kinds that mean "watch it now" wear the accent, as the
/// phone's do (`Notifications.kt:273-274`).
fn kind_label(kind: NoticeKind) -> Option<&'static str> {
    Some(match kind {
        NoticeKind::Coming => "notification.coming",
        NoticeKind::ReminderLive => "notification.reminder_live",
        NoticeKind::NewArrival => "notification.new_arrival",
        NoticeKind::NewSeason => "notification.new_season",
        NoticeKind::NewEpisode => "notification.new_episode",
        _ => return None,
    })
}

/// True for a kind that means the title can be watched now. `Notifications.kt:274`.
fn is_now(kind: NoticeKind) -> bool {
    matches!(kind, NoticeKind::ReminderLive | NoticeKind::NewArrival)
}

/// Where the feed's reading is up to: the cursor it would ask for next, and whether it is asking.
///
/// One value, not two, for the reason `browse/Paging.kt:50-57` gives: held apart, the end of a read
/// wrote them one after the other and a draw that fell between the two writes asked the server for
/// a cursor that had already been spent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Reading {
    cursor: Option<String>,
    asking: bool,
    started: bool,
}

#[component]
pub fn Notices() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let items = RwSignal::new(Vec::<NoticeItem>::new());
    let reading = RwSignal::new(Reading::default());

    let read_part = move |cursor: Option<String>, append: bool| {
        if reading.get_untracked().asking {
            return;
        }
        reading.update(|reading| {
            reading.asking = true;
            reading.started = true;
        });
        let path = match &cursor {
            Some(cursor) => format!("/v1/notifications?cursor={cursor}"),
            None => String::from("/v1/notifications"),
        };
        host::ask(HttpMethod::Get, &path, None, move |status, text| {
            let feed = (200..300)
                .contains(&status)
                .then(|| serde_json::from_str::<Notifications>(&text).ok())
                .flatten();
            // A part that does not arrive stops this page: asking the same cursor again would be a
            // loop against a server that has already said no (`Paging.kt:93-95`).
            let next = feed
                .as_ref()
                .and_then(|feed| feed.has_more.then(|| feed.cursor.clone()).flatten());
            if let Some(feed) = feed {
                items.update(|held| {
                    if append {
                        held.extend(feed.items);
                    } else {
                        *held = feed.items;
                    }
                });
            }
            reading.set(Reading {
                cursor: next,
                asking: false,
                started: true,
            });
        });
    };

    Effect::new(move |_| {
        if !reading.with(|reading| reading.started) {
            read_part(None, false);
        }
    });

    // The next part, read as the viewer reaches the last few. The page scrolls the window, so the
    // window's own scroll is what says so, as it does on the browse page.
    let _listener = window_event_listener(leptos::ev::scroll, move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let seen = window
            .inner_height()
            .ok()
            .and_then(|high| high.as_f64())
            .unwrap_or(0.0);
        let y = window.scroll_y().unwrap_or(0.0);
        let total = window
            .document()
            .and_then(|document| document.document_element())
            .map_or(0.0, |root| f64::from(root.scroll_height()));
        let held = items.with_untracked(Vec::len);
        if held == 0 || total <= 0.0 {
            return;
        }
        // Three items from the end, as a share of the page: the phone counts items because it draws
        // a list it can ask; a page of a document has to ask the document.
        let left = total - (y + seen);
        let item_high = total / held as f64;
        if left <= item_high * READ_AHEAD as f64 {
            let next = reading.get_untracked().cursor;
            if next.is_some() {
                read_part(next, true);
            }
        }
    });

    let open = move |item: &NoticeItem| {
        if item.unread {
            let body = json!({ "up_to": item.id }).to_string();
            host::ask(
                HttpMethod::Post,
                "/v1/notifications/read",
                Some(body),
                move |_, _| host::count_notices(shared),
            );
            let id = item.id.clone();
            items.update(|held| {
                for held in held.iter_mut() {
                    if held.id == id {
                        held.unread = false;
                    }
                }
            });
        }
        host::go(&title_address(item.tile.title));
    };

    view! {
        <div class="sheet">
            <header class="sheet-top">
                <button
                    class="plain"
                    aria-label=move || label_text(shared, "common.back")
                    on:click=move |_| address::back()
                >
                    {move || label_text(shared, "common.back")}
                </button>
                <h1>{move || label_text(shared, "notifications.title")}</h1>
            </header>
            {move || {
                let held = items.get();
                if held.is_empty() {
                    let waiting = !reading.with(|reading| reading.started) || reading.with(|reading| reading.asking);
                    // Nothing yet and nothing coming: the words, and no illustration, as the
                    // phone draws it (`Notifications.kt:161-179` draws a bell it then marks as a
                    // decoration, which on a page of text says nothing a reader can use).
                    return (!waiting)
                        .then(|| view! {
                            <p class="sheet-empty">{label_text(shared, "notifications.empty")}</p>
                        })
                        .into_any();
                }
                held.into_iter().map(|item| {
                    let poster = image(&item.tile.images, &[ImageRole::Poster, ImageRole::Backdrop]);
                    let kind = kind_label(item.kind).and_then(|label| maybe_text(shared, label));
                    let now = is_now(item.kind);
                    let unread = item.unread;
                    let name = item.tile.name.clone();
                    let day = shared.digits.with(|digits| in_digits(item.date.as_str(), digits));
                    let chosen = item.clone();
                    view! {
                        <button
                            class="notice-row"
                            aria-describedby=unread.then_some("unread-said")
                            on:click=move |_| open(&chosen)
                        >
                            <span class="notice-art">
                                {poster.map(|url| view! {
                                    // The title's name is drawn as words beside it, so the poster
                                    // is a decoration (`Notifications.kt:254-258`).
                                    <img src=url alt="" aria-hidden="true"/>
                                })}
                            </span>
                            <span class="notice-text">
                                {kind.map(|kind| view! {
                                    <span class="chip" class:chip-now=now>{kind}</span>
                                })}
                                <strong>{name}</strong>
                                <small>{day}</small>
                            </span>
                            {unread.then(|| view! { <span class="dot" aria-hidden="true"></span> })}
                        </button>
                    }
                }).collect_view().into_any()
            }}
            // One copy of the words for "not read yet", named by every unread notice above rather
            // than written into each one's label: the dot carries no words of its own, and the
            // phone says the same thing as the control's state (`Notifications.kt:243-245`).
            <span id="unread-said" hidden>{move || label_text(shared, "say.unread")}</span>
        </div>
    }
}

/// The words for the bell on the top bar: how many notices are waiting, or the page's own name when
/// none are. `browse/Header.kt:100-105`.
pub fn bell_says(shared: Shared, unread: u32) -> String {
    if unread == 0 {
        return label_text(shared, "notifications.title");
    }
    filled(
        shared,
        "say.notifications_unread",
        "n",
        &super::words::number(shared, unread),
    )
    .unwrap_or_else(|| label_text(shared, "notifications.title"))
}

/// The most notices the bell writes out. Past it the count reads as "9+", as the phone's does
/// (`browse/Header.kt:112`): the mark is 18 dp wide there and a browser's is no wider.
const MOST_SHOWN: u32 = 9;

/// What the bell draws over itself: the count, in this language's digits, held to one digit.
#[must_use]
pub fn bell_count(digits: &str, unread: u32) -> String {
    let shown = unread.min(MOST_SHOWN);
    let plus = if unread > MOST_SHOWN { "+" } else { "" };
    format!("{}{plus}", in_digits(&shown.to_string(), digits))
}

#[cfg(test)]
mod tests {
    use super::{MOST_SHOWN, READ_AHEAD, bell_count, is_now, kind_label};
    use crate::surfaces::tests_support::{english, numbers, source};
    use contract::notification::NoticeKind;

    const PHONE: &str =
        "apps/android/surfaces/phone/src/main/kotlin/streams/phone/notifications/Notifications.kt";

    /// Every kind of notice the contract has carries a label, and every label is in the strings
    /// file. A kind with no words is a row of a date and a title and nothing saying why it is there.
    #[test]
    fn every_kind_of_notice_has_words_the_strings_file_carries() {
        let book = english();
        for kind in [
            NoticeKind::Coming,
            NoticeKind::ReminderLive,
            NoticeKind::NewArrival,
            NoticeKind::NewSeason,
            NoticeKind::NewEpisode,
        ] {
            let label = kind_label(kind).unwrap_or_else(|| panic!("{kind:?} has no label"));
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
        // A kind this build does not know draws no chip rather than an empty one.
        assert!(kind_label(NoticeKind::Unknown).is_none());
    }

    /// The same two kinds wear the accent here and on the phone.
    #[test]
    fn the_kinds_that_mean_watch_it_now_are_the_phone_s_two() {
        assert!(is_now(NoticeKind::ReminderLive));
        assert!(is_now(NoticeKind::NewArrival));
        assert!(!is_now(NoticeKind::Coming));
        assert!(!is_now(NoticeKind::NewSeason));
        assert!(!is_now(NoticeKind::NewEpisode));
        let phone = source(PHONE);
        assert!(
            phone.contains(
                "item.kind == NoticeKind.REMINDER_LIVE || item.kind == NoticeKind.NEW_ARRIVAL"
            ),
            "the phone no longer accents those two kinds"
        );
    }

    /// The feed is read the same distance from its end as the phone reads it.
    #[test]
    fn the_next_part_is_read_the_phone_s_distance_from_the_end() {
        let phone = source(PHONE);
        let said = numbers(
            &["lastVisible >= items.size - ", "\n"],
            &phone,
            "the phone's read-ahead",
        )[0];
        assert_eq!(said, READ_AHEAD as f64);
    }

    /// The count on the bell stops at the phone's own ceiling.
    #[test]
    fn the_count_on_the_bell_stops_where_the_phone_s_does() {
        let phone =
            source("apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Header.kt");
        let ceiling = numbers(
            &["if (unreadCount > ", ") \"", "+\""],
            &phone,
            "the phone's ceiling on the bell",
        );
        assert_eq!(
            ceiling[0], ceiling[1],
            "the phone's ceiling and the text it draws past it disagree"
        );
        assert_eq!(ceiling[0], f64::from(MOST_SHOWN));
        assert_eq!(bell_count("latn", 0), "0");
        assert_eq!(bell_count("latn", 9), "9");
        assert_eq!(bell_count("latn", 10), "9+");
        assert_eq!(bell_count("latn", 400), "9+");
        // And in a language that writes its own digits, the count is written in them.
        assert_eq!(bell_count("arab", 3), "٣");
        assert_eq!(bell_count("arab", 10), "٩+");
    }

    #[test]
    fn every_label_this_screen_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.back",
            "notifications.title",
            "notifications.empty",
            "say.unread",
            "say.notifications_unread",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
    }
}
