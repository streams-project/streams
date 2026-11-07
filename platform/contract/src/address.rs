//! Every address the apps open, in one table. Apps read these names and never type an address, so
//! a change here reaches every family. They follow the addresses viewers already know, and differ
//! only where our data has no such thing: see the kinds in title and watch addresses.

use alloc::format;
use alloc::string::{String, ToString};

use crate::screen::ScreenName;

pub const START: &str = "/";
pub const HOME: &str = "/browse";
pub const KIDS: &str = "/kids";
pub const SERIES: &str = "/browse/genre/83";
pub const FILMS: &str = "/browse/genre/34399";
pub const NEW: &str = "/latest";
pub const GAMES: &str = "/games";
pub const MY_LIST: &str = "/browse/my-list";
pub const MINE: &str = "/mine";
pub const LANGUAGE: &str = "/browse/original-audio";
pub const SEARCH: &str = "/search";
pub const NOTIFICATIONS: &str = "/notifications";
pub const ACTIVITY: &str = "/settings/viewed";
pub const CLIPS: &str = "/clips";
pub const HELP: &str = "/help";
pub const SETTINGS: &str = "/settings";
pub const PROFILES: &str = "/profiles";
pub const SIGN_IN: &str = "/login";
pub const ACTIVATE: &str = "/activate";
/// Followed by a genre.
pub const GENRE: &str = "/browse/genre/";
/// Followed by `kind-id`. Film and series ids overlap, so the kind stays in the address.
pub const TITLE: &str = "/title/";
/// Followed by `kind/id`, then `/season/episode`, as an episode has no id of its own.
pub const WATCH: &str = "/watch/";
/// Details over the current page.
pub const DETAILS_QUERY: &str = "jbv";

/// The table the generated languages are written from.
pub const ALL: &[(&str, &str)] = &[
    ("START", START),
    ("HOME", HOME),
    ("KIDS", KIDS),
    ("SERIES", SERIES),
    ("FILMS", FILMS),
    ("NEW", NEW),
    ("GAMES", GAMES),
    ("MY_LIST", MY_LIST),
    ("MINE", MINE),
    ("LANGUAGE", LANGUAGE),
    ("SEARCH", SEARCH),
    ("NOTIFICATIONS", NOTIFICATIONS),
    ("ACTIVITY", ACTIVITY),
    ("CLIPS", CLIPS),
    ("HELP", HELP),
    ("SETTINGS", SETTINGS),
    ("PROFILES", PROFILES),
    ("SIGN_IN", SIGN_IN),
    ("ACTIVATE", ACTIVATE),
    ("GENRE", GENRE),
    ("TITLE", TITLE),
    ("WATCH", WATCH),
];

/// Where a tab or a screen lives, by the screen's word on the wire.
pub const SCREENS: &[(&str, &str)] = &[
    ("home", HOME),
    ("kids", KIDS),
    ("series", SERIES),
    ("films", FILMS),
    ("new", NEW),
    ("games", GAMES),
    ("my-list", MY_LIST),
    ("mine", MINE),
    ("language", LANGUAGE),
    ("search", SEARCH),
    ("notifications", NOTIFICATIONS),
    ("activity", ACTIVITY),
    ("clips", CLIPS),
    ("help", HELP),
    ("settings", SETTINGS),
];

/// The address of a screen. A genre page needs its genre.
#[must_use]
pub fn of_screen(screen: ScreenName, genre: Option<&str>) -> String {
    let fixed = match screen {
        ScreenName::Genre => return format!("{GENRE}{}", genre.unwrap_or("")),
        ScreenName::Kids => KIDS,
        ScreenName::Series => SERIES,
        ScreenName::Films => FILMS,
        ScreenName::New => NEW,
        ScreenName::Games => GAMES,
        ScreenName::MyList => MY_LIST,
        ScreenName::Mine => MINE,
        ScreenName::Language => LANGUAGE,
        ScreenName::Search => SEARCH,
        ScreenName::Notifications => NOTIFICATIONS,
        ScreenName::Activity => ACTIVITY,
        ScreenName::Clips => CLIPS,
        ScreenName::Help => HELP,
        ScreenName::Settings => SETTINGS,
        _ => HOME,
    };
    fixed.to_string()
}
