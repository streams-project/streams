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
        to: "/latest".to_string(),
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
    assert_eq!(route.next.as_deref(), Some("/latest"));
    // trailing slash and capitals do not matter to the table
    assert_eq!(
        decide("/Browse/Latest/", &who).next.as_deref(),
        Some("/latest")
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
fn n10_an_approve_address_keeps_its_code_through_sign_in() {
    let address = "/activate?code=BCDFGHJK";
    assert_eq!(safe_next(address).as_deref(), Some(address));
    let route = decide(address, &VISITOR);
    assert_eq!(route.target, Target::SignIn);
    assert_eq!(route.next.as_deref(), Some(address));
    assert_eq!(route.code, None, "the code travels in next until sign in");
    let route = decide(address, &MEMBER);
    assert_eq!(route.target, Target::Activate);
    assert_eq!(route.code.as_deref(), Some("BCDFGHJK"));
    let route = decide("/login?next=%2Factivate%3Fcode%3DBCDFGHJK", &MEMBER);
    assert_eq!(route.target, Target::Activate);
    assert_eq!(route.code.as_deref(), Some("BCDFGHJK"));
}

#[test]
fn n10_an_approve_address_may_hold_no_code_or_an_odd_one() {
    let plain = decide("/activate", &MEMBER);
    assert_eq!((plain.target, plain.code), (Target::Activate, None));
    assert_eq!(
        decide("/activate/?code=BCDF%20GHJK", &MEMBER)
            .code
            .as_deref(),
        Some("BCDF GHJK")
    );
    let broken = decide("/activate?code=%zz", &MEMBER);
    assert_eq!((broken.target, broken.code), (Target::Activate, None));
    assert_eq!(decide("/activate/more", &MEMBER).target, Target::NotFound);
}

#[test]
fn n10_a_visitor_who_may_browse_still_signs_in_to_approve_a_device() {
    let open = Who {
        accounts_required: Some(false),
        ..VISITOR
    };
    assert_eq!(decide("/browse", &open).target, Target::Home);
    let route = decide("/activate?code=BCDFGHJK", &open);
    assert_eq!(route.target, Target::SignIn);
    assert_eq!(route.next.as_deref(), Some("/activate?code=BCDFGHJK"));
    let unknown = Who {
        accounts_required: None,
        ..VISITOR
    };
    assert_eq!(
        decide("/activate?code=BCDFGHJK", &unknown).target,
        Target::Wait
    );
}

#[test]
fn c131_an_unknown_answer_waits_and_nothing_is_drawn() {
    let who = Who {
        member: false,
        accounts_required: None,
        moved: &[],
        home: Target::Home,
    };
    assert_eq!(decide("/browse", &who).target, Target::Wait);
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
    assert_eq!(decide("/browse", &who).target, Target::Home);
}

#[test]
fn c131_c102_a_visitor_goes_to_sign_in_with_next_and_a_member_leaves_sign_in() {
    let route = decide("/browse/my-list?x=1", &VISITOR);
    assert_eq!(route.target, Target::SignIn);
    assert_eq!(route.next.as_deref(), Some("/browse/my-list?x=1"));
    assert_eq!(decide("/login", &VISITOR).target, Target::SignIn);
    assert_eq!(decide("/login", &MEMBER).target, Target::Home);
    let route = decide("/login?next=%2Fbrowse%2Fmy-list", &MEMBER);
    assert_eq!(route.target, Target::Screen);
    assert_eq!(route.screen, Some(ScreenName::MyList));
}

#[test]
fn c102_next_must_be_one_local_members_address() {
    assert_eq!(safe_next("/browse").as_deref(), Some("/browse"));
    assert_eq!(
        safe_next("/browse/my-list?x=1").as_deref(),
        Some("/browse/my-list?x=1")
    );
    // Each of these names a members page the shape rule alone refuses, so none of them can pass
    // for another reason: drop the guard and the address comes back.
    for bad in [
        "home",            // no leading slash, so not local
        "//home",          // two slashes name another host
        "/browse?x=a\\b",  // a backslash, which some clients read as a slash
        "/browse?x=a b",   // a space
        "/browse?x=a\nme", // a control character
    ] {
        assert_eq!(safe_next(bad), None, "{bad}");
    }
    // these are local and well shaped, and refused because they are open to visitors or nothing
    for open in ["/", "/login", "/title/movie-7", "/nowhere"] {
        assert_eq!(safe_next(open), None, "{open}");
    }
    assert_eq!(
        decide("/login?next=//host.example", &MEMBER).target,
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
        ("/browse/genre/34399", ScreenName::Films),
        ("/browse/genre/83", ScreenName::Series),
        ("/latest", ScreenName::New),
        ("/browse/my-list", ScreenName::MyList),
        ("/search", ScreenName::Search),
        ("/notifications", ScreenName::Notifications),
        ("/settings/viewed", ScreenName::Activity),
        ("/games", ScreenName::Games),
        ("/clips", ScreenName::Clips),
        ("/browse/original-audio", ScreenName::Language),
    ] {
        let route = decide(address, &MEMBER);
        assert_eq!((route.target, route.screen), (Target::Screen, Some(name)));
    }
    let route = decide("/browse/genre/28", &MEMBER);
    assert_eq!(route.screen, Some(ScreenName::Genre));
    assert_eq!(route.genre.as_deref(), Some("28"));
    assert_eq!(decide("/kids", &MEMBER).target, Target::KidsHome);
    assert_eq!(decide("/profiles", &MEMBER).target, Target::Profiles);
}

#[test]
fn a_title_page_is_open_to_a_visitor() {
    let route = decide("/title/tv-9", &VISITOR);
    assert_eq!(route.target, Target::Title);
    assert_eq!(route.title, Some(key(TitleKind::Tv, 9)));
}

#[test]
fn p35_a_pending_title_goes_along() {
    let route = decide("/browse?title=tv:9", &MEMBER);
    assert_eq!(route.target, Target::Home);
    assert_eq!(route.title, Some(key(TitleKind::Tv, 9)));
    // the kind separator is often escaped by whoever wrote the address
    assert_eq!(
        decide("/browse?title=tv%3A9", &MEMBER).title,
        Some(key(TitleKind::Tv, 9))
    );
    // a bare id has no kind, so it is left behind
    assert_eq!(decide("/browse?title=9", &MEMBER).title, None);
}

#[test]
fn p35_the_root_and_sign_in_lead_a_member_to_the_profiles_own_home() {
    let kids = Who {
        home: Target::KidsHome,
        ..MEMBER
    };
    assert_eq!(decide("/", &kids).target, Target::KidsHome);
    assert_eq!(decide("/login", &kids).target, Target::KidsHome);
}

#[test]
fn c131_a_moved_table_that_leads_back_to_itself_ends_at_home() {
    let moved = [MovedAddress {
        from: "/browse".to_string(),
        to: "/login?next=/home".to_string(),
    }];
    let who = Who {
        member: true,
        accounts_required: Some(true),
        moved: &moved,
        home: Target::Home,
    };
    assert_eq!(decide("/browse", &who).target, Target::Home);
}

#[test]
fn help_opens_for_a_visitor_and_settings_asks_to_sign_in_first() {
    let open = decide("/help", &VISITOR);
    assert_eq!(open.target, Target::Screen);
    assert_eq!(open.screen, Some(ScreenName::Help));
    let closed = decide("/settings", &VISITOR);
    assert_eq!(closed.target, Target::SignIn);
    assert_eq!(closed.next.as_deref(), Some("/settings"));
    assert_eq!(
        decide("/settings", &MEMBER).screen,
        Some(ScreenName::Settings)
    );
}

#[test]
fn the_address_table_opens_every_page_it_names() {
    for (address, screen) in [
        (contract::address::SERIES, ScreenName::Series),
        (contract::address::FILMS, ScreenName::Films),
        (contract::address::NEW, ScreenName::New),
        (contract::address::MY_LIST, ScreenName::MyList),
        (contract::address::LANGUAGE, ScreenName::Language),
        (contract::address::ACTIVITY, ScreenName::Activity),
        (contract::address::MINE, ScreenName::Mine),
        (contract::address::GAMES, ScreenName::Games),
    ] {
        assert_eq!(decide(address, &MEMBER).screen, Some(screen), "{address}");
        assert_eq!(
            contract::address::of_screen(screen, None),
            address,
            "the table and the rule agree"
        );
    }
    assert_eq!(
        decide(contract::address::HOME, &MEMBER).target,
        Target::Home
    );
    assert_eq!(decide("/", &VISITOR).target, Target::Start);
}

#[test]
fn details_open_over_any_page_and_a_genre_ignores_its_parent() {
    let title = key(TitleKind::Tv, 1399);
    assert_eq!(decide("/title/tv-1399", &MEMBER).title, Some(title));
    assert_eq!(decide("/browse?jbv=tv-1399", &MEMBER).title, Some(title));
    assert_eq!(decide("/latest?jbv=tv-1399", &MEMBER).title, Some(title));
    let genre = decide("/browse/genre/comedy?bc=83", &MEMBER);
    assert_eq!(genre.screen, Some(ScreenName::Genre));
    assert_eq!(genre.genre.as_deref(), Some("comedy"));
    assert_eq!(decide("/title/1399", &MEMBER).target, Target::NotFound);
    assert_eq!(decide("/home", &MEMBER).target, Target::NotFound);
    assert_eq!(decide("/tv2", &MEMBER).target, Target::Activate);
}
