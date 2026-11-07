//! What an address or a deep link opens. The order is fixed: the moved table as data, then an
//! address that does not exist, then who may open it. A semantic target comes out and each device
//! maps it to a screen. There is no audience list: the server answers each screen call allowed or
//! refused, and the core maps that code to a target.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use contract::core_msg::{RouteTarget, Target};
use contract::hello::MovedAddress;
use contract::screen::ScreenName;
use contract::{EpisodeRef, TitleKey};

/// Where a member lands when nothing else was asked.
pub const HOME_ADDRESS: &str = "/home";

/// What the decision needs to know.
#[derive(Clone, Copy, Debug)]
pub struct Who<'a> {
    pub member: bool,
    /// `None` while the answer is unknown.
    pub accounts_required: Option<bool>,
    pub moved: &'a [MovedAddress],
    /// Where this profile starts, as the server named it.
    pub home: Target,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Place {
    Start,
    SignIn,
    Profiles,
    Home,
    Kids,
    Screen(ScreenName, Option<String>),
    Title(TitleKey),
    Watch(TitleKey, Option<EpisodeRef>),
    Activate(Option<String>),
}

impl Place {
    const fn open_to_visitors(&self) -> bool {
        matches!(
            self,
            Self::Start | Self::SignIn | Self::Title(_) | Self::Screen(ScreenName::Help, _)
        )
    }

    /// Approving a device gives it this account, so a visitor who may browse still signs in.
    const fn needs_account(&self) -> bool {
        matches!(self, Self::Activate(_))
    }
}

/// A target with nothing else set.
#[must_use]
pub const fn target(target: Target) -> RouteTarget {
    RouteTarget {
        target,
        screen: None,
        genre: None,
        title: None,
        episode: None,
        next: None,
        code: None,
    }
}

fn split(address: &str) -> (&str, &str) {
    let address = address.split('#').next().unwrap_or(address);
    let (path, query) = address.split_once('?').unwrap_or((address, ""));
    let path = path.trim_end_matches('/');
    (if path.is_empty() { "/" } else { path }, query)
}

fn query_value<'a>(query: &'a str, names: &[&str]) -> Option<&'a str> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| names.contains(name))
        .map(|(_, value)| value)
}

fn hex(byte: u8) -> Option<u8> {
    char::from(byte)
        .to_digit(16)
        .and_then(|digit| u8::try_from(digit).ok())
}

fn decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut kept = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' {
            let high = hex(*bytes.get(at + 1)?)?;
            let low = hex(*bytes.get(at + 2)?)?;
            kept.push(high * 16 + low);
            at += 3;
        } else {
            kept.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8(kept).ok()
}

fn title_key(kind: &str, id: &str) -> Option<TitleKey> {
    TitleKey::parse(&format!("{kind}:{id}")).ok()
}

const fn screen_named(word: &str) -> Option<ScreenName> {
    match word.as_bytes() {
        b"films" => Some(ScreenName::Films),
        b"series" => Some(ScreenName::Series),
        b"new" => Some(ScreenName::New),
        b"my-list" => Some(ScreenName::MyList),
        b"mine" => Some(ScreenName::Mine),
        b"search" => Some(ScreenName::Search),
        b"notifications" => Some(ScreenName::Notifications),
        b"activity" => Some(ScreenName::Activity),
        b"games" => Some(ScreenName::Games),
        b"clips" => Some(ScreenName::Clips),
        b"language" => Some(ScreenName::Language),
        b"help" => Some(ScreenName::Help),
        b"settings" => Some(ScreenName::Settings),
        _ => None,
    }
}

/// Season and episode of a watch address, both required.
fn episode_of(query: &str) -> Option<EpisodeRef> {
    let season = query_value(query, &["season", "s"])?.parse().ok()?;
    let episode = query_value(query, &["episode", "e"])?.parse().ok()?;
    Some(EpisodeRef { season, episode })
}

fn place(path: &str, query: &str) -> Option<Place> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [""] => Some(Place::Start),
        ["sign-in"] => Some(Place::SignIn),
        ["profiles"] => Some(Place::Profiles),
        ["home"] => Some(Place::Home),
        ["kids"] => Some(Place::Kids),
        ["activate"] => Some(Place::Activate(
            query_value(query, &["code"]).and_then(decode),
        )),
        ["genre", id] if !id.is_empty() => {
            Some(Place::Screen(ScreenName::Genre, Some((*id).to_string())))
        }
        ["title", kind, id] => title_key(kind, id).map(Place::Title),
        ["watch", kind, id] => {
            let title = title_key(kind, id)?;
            Some(Place::Watch(title, episode_of(query)))
        }
        ["watch", kind, id, season, episode] => {
            let title = title_key(kind, id)?;
            let pair = EpisodeRef {
                season: season.parse().ok()?,
                episode: episode.parse().ok()?,
            };
            Some(Place::Watch(title, Some(pair)))
        }
        [word] => screen_named(word).map(|name| Place::Screen(name, None)),
        _ => None,
    }
}

fn moved_path<'a>(path: &str, moved: &'a [MovedAddress]) -> Option<&'a str> {
    moved
        .iter()
        .find(|row| row.from.eq_ignore_ascii_case(path))
        .map(|row| row.to.as_str())
}

/// A title waiting in the address goes along. A bare id has no kind and is left behind.
fn pending_title(query: &str) -> Option<TitleKey> {
    TitleKey::parse(&decode(query_value(query, &["title"])?)?).ok()
}

fn member_target(place: Place, query: &str) -> RouteTarget {
    let mut route = target(Target::NotFound);
    match place {
        Place::Profiles => route.target = Target::Profiles,
        Place::Kids => {
            route.target = Target::KidsHome;
            route.title = pending_title(query);
        }
        Place::Screen(name, genre) => {
            route.target = Target::Screen;
            route.screen = Some(name);
            route.genre = genre;
            route.title = pending_title(query);
        }
        Place::Watch(title, episode) => {
            route.target = Target::Watch;
            route.title = Some(title);
            route.episode = episode;
        }
        Place::Title(title) => {
            route.target = Target::Title;
            route.title = Some(title);
        }
        Place::Start => route.target = Target::Start,
        Place::SignIn => route.target = Target::SignIn,
        Place::Home => {
            route.target = Target::Home;
            route.title = pending_title(query);
        }
        Place::Activate(code) => {
            route.target = Target::Activate;
            route.code = code;
        }
    }
    route
}

/// `next` must be one local address that only a member opens, or nothing.
#[must_use]
pub fn safe_next(next: &str) -> Option<String> {
    let local = next.starts_with('/')
        && !next.starts_with("//")
        && !next.contains('\\')
        && !next.chars().any(|c| c.is_control() || c.is_whitespace());
    if !local {
        return None;
    }
    let (path, query) = split(next);
    let found = place(path, query)?;
    (!found.open_to_visitors()).then(|| next.to_string())
}

fn next_of(query: &str) -> Option<String> {
    safe_next(&decode(query_value(query, &["next"])?)?)
}

/// Decide what an address opens.
#[must_use]
pub fn decide(address: &str, who: &Who<'_>) -> RouteTarget {
    decide_from(address, who, true)
}

/// `next` is followed once. The moved table is data from a server, and a row that leads back to
/// itself must not hang a device.
fn decide_from(address: &str, who: &Who<'_>, follow_next: bool) -> RouteTarget {
    let (path, query) = split(address);
    // 1. an address that moved. The query stays with it.
    let moved = moved_path(path, who.moved);
    let path = moved.unwrap_or(path);
    let (path, own_query) = split(path);
    let query = if own_query.is_empty() {
        query
    } else {
        own_query
    };
    let full = if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    };
    // 2. an address that does not exist
    let Some(found) = place(path, query) else {
        return target(Target::NotFound);
    };
    // 3. who may open it
    match found {
        Place::Start if who.member => target(who.home),
        Place::SignIn if who.member => next_of(query)
            .filter(|_| follow_next)
            .map_or(target(who.home), |next| decide_from(&next, who, false)),
        Place::SignIn => {
            let mut route = target(Target::SignIn);
            route.next = next_of(query);
            route
        }
        place if place.open_to_visitors() => member_target(place, query),
        _ if who.member => member_target(found, query),
        _ if who.accounts_required == Some(false) && !found.needs_account() => {
            member_target(found, query)
        }
        _ if who.accounts_required.is_none() => target(Target::Wait),
        _ => {
            let mut route = target(Target::SignIn);
            route.next = safe_next(&full);
            route
        }
    }
}
