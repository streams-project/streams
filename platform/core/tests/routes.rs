//! What an address opens. Each test names the behaviour it covers.

use contract::core_msg::Target;
use contract::hello::MovedAddress;
use contract::screen::ScreenName;
use contract::{EpisodeRef, TitleKey, TitleKind};
use platform_core::routes::{Who, decide, safe_next};

const VISITOR: Who<'_> = Who {
    member: false,
    accounts_required: Some(true),
    moved: &[],
    home: Target::Home,
};
const MEMBER: Who<'_> = Who {
    member: true,
    accounts_required: Some(true),
    moved: &[],
    home: Target::Home,
};

fn key(kind: TitleKind, id: u32) -> TitleKey {
    TitleKey::new(kind, id).unwrap()
}

#[test]
fn c131_a_moved_address_comes_first_then_unknown_then_access() {
    let moved = [MovedAddress {
        from: "/browse/latest".to_string(),
        to: "/new".to_string(),
    }];
    let who = Who {
        member: false,
        accounts_required: Some(true),
        moved: &moved,
        home: Target::Home,
    };
    // moved, then the visitor rule sends the final address on to sign in
    let route = decide("/browse/latest", &who);
    assert_eq!(route.target, Target::SignIn);
    assert_eq!(route.next.as_deref(), Some("/new"));
    // trailing slash and capitals do not matter to the table
    assert_eq!(
        decide("/Browse/Latest/", &who).next.as_deref(),
        Some("/new")
    );
    assert_eq!(decide("/nowhere", &who).target, Target::NotFound);
    let member = Who {
        member: true,
        ..who
    };
    let route = decide("/browse/latest", &member);
    assert_eq!(route.target, Target::Screen);
    assert_eq!(route.screen, Some(ScreenName::New));
}

#[test]
fn c131_an_unknown_answer_waits_and_nothing_is_drawn() {
    let who = Who {
        member: false,
        accounts_required: None,
        moved: &[],
        home: Target::Home,
    };
    assert_eq!(decide("/home", &who).target, Target::Wait);
    assert_eq!(decide("/", &who).target, Target::Start);
    assert_eq!(decide("/nowhere", &who).target, Target::NotFound);
}

#[test]
fn c108_with_accounts_off_a_visitor_stays_on_a_members_page() {
    let who = Who {
        member: false,
        accounts_required: Some(false),
        moved: &[],
        home: Target::Home,
    };
    assert_eq!(decide("/home", &who).target, Target::Home);
}

#[test]
fn c131_c102_a_visitor_goes_to_sign_in_with_next_and_a_member_leaves_sign_in() {
    let route = decide("/my-list?x=1", &VISITOR);
    assert_eq!(route.target, Target::SignIn);
    assert_eq!(route.next.as_deref(), Some("/my-list?x=1"));
    assert_eq!(decide("/sign-in", &VISITOR).target, Target::SignIn);
    assert_eq!(decide("/sign-in", &MEMBER).target, Target::Home);
    let route = decide("/sign-in?next=%2Fmy-list", &MEMBER);
    assert_eq!(route.target, Target::Screen);
    assert_eq!(route.screen, Some(ScreenName::MyList));
}

#[test]
fn c102_next_must_be_one_local_members_address() {
    assert_eq!(safe_next("/home").as_deref(), Some("/home"));
    assert_eq!(safe_next("/my-list?x=1").as_deref(), Some("/my-list?x=1"));
    // Each of these names a members page the shape rule alone refuses, so none of them can pass
    // for another reason: drop the guard and the address comes back.
    for bad in [
        "home",          // no leading slash, so not local
        "//home",        // two slashes name another host
        "/home?x=a\\b",  // a backslash, which some clients read as a slash
        "/home?x=a b",   // a space
        "/home?x=a\nme", // a control character
    ] {
        assert_eq!(safe_next(bad), None, "{bad}");
    }
    // these are local and well shaped, and refused because they are open to visitors or nothing
    for open in ["/", "/sign-in", "/title/movie/7", "/nowhere"] {
        assert_eq!(safe_next(open), None, "{open}");
    }
    assert_eq!(
        decide("/sign-in?next=//host.example", &MEMBER).target,
        Target::Home
    );
}

#[test]
fn the_root_is_the_start_page_for_a_visitor_and_home_for_a_member() {
    assert_eq!(decide("/", &VISITOR).target, Target::Start);
    assert_eq!(decide("", &VISITOR).target, Target::Start);
    assert_eq!(decide("/", &MEMBER).target, Target::Home);
}

#[test]
fn c35_no_request_without_a_kind() {
    assert_eq!(decide("/watch/7", &MEMBER).target, Target::NotFound);
    assert_eq!(decide("/watch/film/7", &MEMBER).target, Target::NotFound);
    assert_eq!(decide("/title/7", &MEMBER).target, Target::NotFound);
    let route = decide("/watch/movie/7", &MEMBER);
    assert_eq!(route.target, Target::Watch);
    assert_eq!(route.title, Some(key(TitleKind::Movie, 7)));
    assert_eq!(route.episode, None);
}

#[test]
fn c34_season_and_episode_come_from_the_address_both_required_with_short_names() {
    let tv = key(TitleKind::Tv, 7);
    let pair = Some(EpisodeRef {
        season: 2,
        episode: 3,
    });
    assert_eq!(decide("/watch/tv/7/2/3", &MEMBER).episode, pair);
    assert_eq!(
        decide("/watch/tv/7?season=2&episode=3", &MEMBER).episode,
        pair
    );
    assert_eq!(decide("/watch/tv/7?s=2&e=3", &MEMBER).episode, pair);
    let route = decide("/watch/tv/7?season=2", &MEMBER);
    assert_eq!(route.title, Some(tv));
    assert_eq!(route.episode, None);
    // an episode with no season names nothing: no season is assumed
    let route = decide("/watch/tv/7?episode=3", &MEMBER);
    assert_eq!(route.title, Some(tv));
    assert_eq!(route.episode, None);
    assert_eq!(decide("/watch/tv/7?e=3", &MEMBER).episode, None);
}

#[test]
fn every_screen_name_has_an_address_a_genre_carries_its_id() {
    for (address, name) in [
        ("/films", ScreenName::Films),
        ("/series", ScreenName::Series),
        ("/new", ScreenName::New),
        ("/my-list", ScreenName::MyList),
        ("/search", ScreenName::Search),
        ("/notifications", ScreenName::Notifications),
        ("/activity", ScreenName::Activity),
        ("/games", ScreenName::Games),
        ("/clips", ScreenName::Clips),
        ("/language", ScreenName::Language),
    ] {
        let route = decide(address, &MEMBER);
        assert_eq!((route.target, route.screen), (Target::Screen, Some(name)));
    }
    let route = decide("/genre/28", &MEMBER);
    assert_eq!(route.screen, Some(ScreenName::Genre));
    assert_eq!(route.genre.as_deref(), Some("28"));
    assert_eq!(decide("/kids", &MEMBER).target, Target::KidsHome);
    assert_eq!(decide("/profiles", &MEMBER).target, Target::Profiles);
}

#[test]
fn a_title_page_is_open_to_a_visitor() {
    let route = decide("/title/tv/9", &VISITOR);
    assert_eq!(route.target, Target::Title);
    assert_eq!(route.title, Some(key(TitleKind::Tv, 9)));
}

#[test]
fn p35_a_pending_title_goes_along() {
    let route = decide("/home?title=tv:9", &MEMBER);
    assert_eq!(route.target, Target::Home);
    assert_eq!(route.title, Some(key(TitleKind::Tv, 9)));
    // the kind separator is often escaped by whoever wrote the address
    assert_eq!(
        decide("/home?title=tv%3A9", &MEMBER).title,
        Some(key(TitleKind::Tv, 9))
    );
    // a bare id has no kind, so it is left behind
    assert_eq!(decide("/home?title=9", &MEMBER).title, None);
}

#[test]
fn p35_the_root_and_sign_in_lead_a_member_to_the_profiles_own_home() {
    let kids = Who {
        home: Target::KidsHome,
        ..MEMBER
    };
    assert_eq!(decide("/", &kids).target, Target::KidsHome);
    assert_eq!(decide("/sign-in", &kids).target, Target::KidsHome);
}

#[test]
fn c131_a_moved_table_that_leads_back_to_itself_ends_at_home() {
    let moved = [MovedAddress {
        from: "/home".to_string(),
        to: "/sign-in?next=/home".to_string(),
    }];
    let who = Who {
        member: true,
        accounts_required: Some(true),
        moved: &moved,
        home: Target::Home,
    };
    assert_eq!(decide("/home", &who).target, Target::Home);
}
