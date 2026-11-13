//! The contract's route table against the router: no route the contract names is missing, and the
//! ones still to build are named here so the list can only shrink.

mod common;

use std::collections::BTreeSet;

use axum::http::{Method, StatusCode};
use common::{call, harness};
use contract::api::{Auth, Group, ROUTES};

/// Routes not built yet, each with what it waits for.
const STILL_TO_BUILD: [(&str, &str, &str); 1] = [("POST", "/v1/core/step", "server-hosted")];

fn filled(path: &str) -> String {
    path.replace("{kind}", "movie")
        .replace("{id}", "100")
        .replace("{slug}", "x")
        .replace("{lang}", "en")
        .replace("{name}", "home")
        .replace("{cursor}", "x")
        .replace("{n}", "1")
        .replace("{file}", "x")
}

#[tokio::test]
async fn u21_every_contract_route_answers_and_no_unknown_one_is_left() {
    let server = harness(&[]);
    for route in ROUTES.iter().filter(|route| route.group != Group::Finder) {
        let waiting = STILL_TO_BUILD
            .iter()
            .any(|(method, path, _)| *method == route.method && *path == route.path);
        let method: Method = route.method.parse().unwrap();
        let answer = call(method, &filled(route.path)).on(&server.state).await;
        let found = answer.status != StatusCode::NOT_FOUND
            && answer.status != StatusCode::METHOD_NOT_ALLOWED;
        if route.path.starts_with("/v1/strings") {
            continue;
        }
        if waiting {
            assert!(!found, "{} is built, take it off the list", route.path);
        } else {
            assert!(
                found,
                "{} {} is not mounted: {}",
                route.method, route.path, answer.status
            );
        }
    }
}

#[tokio::test]
async fn s3_a_route_that_needs_a_token_refuses_without_one() {
    let server = harness(&[]);
    for route in ROUTES {
        if !matches!(route.auth, Auth::Member | Auth::Profile) {
            continue;
        }
        let waiting = STILL_TO_BUILD
            .iter()
            .any(|(_, path, _)| *path == route.path);
        if waiting {
            continue;
        }
        let method: Method = route.method.parse().unwrap();
        let mut request = call(method, &filled(route.path));
        if route.request.is_some() {
            request = request.json(serde_json::json!({}));
        }
        let answer = request.on(&server.state).await;
        let refused = matches!(
            answer.status,
            StatusCode::UNAUTHORIZED | StatusCode::BAD_REQUEST
        );
        assert!(
            refused,
            "{} {}: {}",
            route.method, route.path, answer.status
        );
    }
}

#[tokio::test]
async fn tmt19_no_route_sets_a_cookie() {
    let server = harness(&[]);
    for route in ROUTES
        .iter()
        .filter(|route| route.method == "GET" && route.group != Group::Finder)
    {
        let answer = call(Method::GET, &filled(route.path))
            .on(&server.state)
            .await;
        assert!(!answer.headers.contains_key("set-cookie"), "{}", route.path);
    }
}

#[tokio::test]
async fn s51_the_job_deletes_lapsed_sessions_and_a_data_reload_with_no_change_is_quiet() {
    let server = harness(&[]);
    let (_, token) = common::member(&server.door).await;
    assert_eq!(
        common::get("/v1/profiles")
            .token(&token)
            .on(&server.state)
            .await
            .status,
        StatusCode::OK
    );
    server_api::jobs::reload_data(&server.state).await;
    server.clock.advance_ms(31 * 86_400_000);
    assert_eq!(server_api::jobs::prune(&server.state).await, 1);
    assert_eq!(server_api::jobs::prune(&server.state).await, 0);
}

#[tokio::test]
async fn s109_a_row_the_operator_blocked_leaves_every_page_without_a_restart() {
    let dir = common::folder("live");
    let path = dir.join("live.json");
    let path_text = path.to_string_lossy().into_owned();
    let server = harness(&[("LIVE_SETTINGS", path_text.as_str())]);
    let (_, token) = common::member(&server.door).await;
    let list = common::get("/v1/profiles")
        .token(&token)
        .on(&server.state)
        .await;
    let id = list.body["profiles"][0]["id"].as_str().unwrap().to_owned();
    call(Method::POST, &format!("/v1/profiles/{id}/enter"))
        .token(&token)
        .json(serde_json::json!({}))
        .on(&server.state)
        .await;
    let read = || common::get("/v1/screens/home").token(&token);
    let before = read().on(&server.state).await;
    assert!(before.body.to_string().contains("Drama Evenings"));
    std::fs::write(&path, r#"{"blocked_rows":["drama evenings"]}"#).unwrap();
    let after = read().on(&server.state).await;
    assert!(!after.body.to_string().contains("Drama Evenings"));
}

/// A concrete path for a route the contract marks cacheable, on the sample set.
///
/// Every cacheable route needs an entry: a new one the contract adds and nobody maps here fails
/// the two tests below by name, so the cache promise cannot grow unchecked. `cacheable` and
/// `text_form` are fields of the contract that no test read before these.
fn cacheable_path(path: &str) -> String {
    let series = server_catalog::sample::special::SHARED_ID;
    match path {
        "/v1/hello" => String::from("/v1/hello"),
        "/v1/strings/{lang}" => String::from("/v1/strings/en"),
        "/v1/languages" => String::from("/v1/languages"),
        "/v1/help" => String::from("/v1/help"),
        "/v1/help/articles/{slug}" => String::from("/v1/help/articles/first"),
        "/v1/help/policies/{id}" => String::from("/v1/help/policies/terms"),
        "/v1/public/trending" => String::from("/v1/public/trending"),
        // A U title, so the stated visitor tier does not refuse it.
        "/v1/public/titles/{kind}/{id}" => String::from("/v1/public/titles/movie/106"),
        "/v1/nav" => String::from("/v1/nav"),
        "/v1/screens/{name}" => String::from("/v1/screens/home"),
        "/v1/titles/{kind}/{id}" => String::from("/v1/titles/movie/100"),
        "/v1/titles/{kind}/{id}/seasons/{n}" => format!("/v1/titles/tv/{series}/seasons/1"),
        other => panic!(
            "{other} is cacheable in the contract and has no path here: give it one, because a \
             client generated from the contract will hold its answer and ask again with an ETag"
        ),
    }
}

/// A server with text, help and a visitor tier, so every open cacheable route answers rather than
/// reporting its feature off. Accounts stay required, so the profile routes still want a token.
fn full_server() -> (common::Harness, String) {
    let strings = common::folder("strings");
    std::fs::write(strings.join("en.json"), r#"{"language.name":"English"}"#).expect("en");
    let strings = strings.to_string_lossy().into_owned();
    let books = concat!(env!("CARGO_MANIFEST_DIR"), "/../server-help/tests/books");
    let server = harness(&[
        ("STRINGS_DIR", strings.as_str()),
        ("HELP_DIR", books),
        ("VISITOR_TIER", "PG"),
        ("BRAND_NAME", "Sample"),
    ]);
    (server, strings)
}

/// A token whose session has entered a grown up profile.
async fn watching(server: &common::Harness) -> String {
    let (_, token) = common::member(&server.door).await;
    let profiles = common::get("/v1/profiles")
        .token(&token)
        .on(&server.state)
        .await;
    let list = profiles.body["profiles"].as_array().expect("profiles");
    let chosen = list
        .iter()
        .find(|row| row["kids"] == false)
        .expect("a grown up profile")["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let entered = call(Method::POST, &format!("/v1/profiles/{chosen}/enter"))
        .token(&token)
        .json(serde_json::json!({}))
        .on(&server.state)
        .await;
    assert_eq!(entered.status, StatusCode::OK, "{}", entered.body);
    token
}

#[tokio::test]
async fn n17_every_route_the_contract_calls_cacheable_sends_an_etag_and_answers_304_to_it() {
    let (server, _strings) = full_server();
    let token = watching(&server).await;
    let mut checked = 0usize;
    for route in ROUTES.iter().filter(|route| route.cacheable) {
        let path = cacheable_path(route.path);
        assert_eq!(route.method, "GET", "{path} is cacheable and not a GET");
        let first = common::get(&path).token(&token).on(&server.state).await;
        assert_eq!(
            first.status,
            StatusCode::OK,
            "{path} is cacheable and did not answer: {}",
            first.body
        );
        let policy = first.headers["cache-control"]
            .to_str()
            .expect("cache-control")
            .to_owned();
        assert_ne!(
            policy, "no-store",
            "{path} is cacheable in the contract and tells the client not to store it"
        );
        let tag = first
            .headers
            .get("etag")
            .unwrap_or_else(|| panic!("{path} is cacheable in the contract and sent no ETag"))
            .to_str()
            .expect("etag")
            .to_owned();
        let again = common::get(&path)
            .token(&token)
            .header("if-none-match", &tag)
            .on(&server.state)
            .await;
        assert_eq!(
            again.status,
            StatusCode::NOT_MODIFIED,
            "{path} sent an ETag and then ignored it: {}",
            again.status
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        ROUTES.iter().filter(|route| route.cacheable).count(),
        "every cacheable route is checked"
    );
}

#[tokio::test]
async fn n17_no_route_the_contract_leaves_uncacheable_hands_out_an_etag() {
    let (server, _strings) = full_server();
    let token = watching(&server).await;
    // The GETs that are not cacheable and that this harness can answer with a profile token.
    let uncacheable = [
        "/v1/ready",
        "/v1/help/search?q=audio",
        "/v1/profiles",
        "/v1/profiles/me/settings",
        "/v1/notifications",
        "/v1/notifications/count",
        "/v1/changes",
        "/v1/search?q=first",
        "/v1/search/suggest?q=fir",
    ];
    for path in uncacheable {
        let bare = path.split('?').next().expect("path");
        let route = contract::api::find("GET", bare).unwrap_or_else(|| panic!("{bare}"));
        assert!(
            !route.cacheable,
            "{bare} is cacheable in the contract: it belongs in the test above"
        );
        let answer = common::get(path).token(&token).on(&server.state).await;
        assert_eq!(
            answer.status,
            StatusCode::OK,
            "{path} did not answer: {}",
            answer.body
        );
        assert_eq!(
            answer.headers["cache-control"], "no-store",
            "{path} is not cacheable in the contract"
        );
        assert!(
            !answer.headers.contains_key("etag"),
            "{path} is not cacheable in the contract and offered an ETag, so a client would hold \
             an answer the contract says it may not hold"
        );
    }
}

#[tokio::test]
async fn the_one_route_with_a_text_form_answers_it_and_json_without_it() {
    let with_text: Vec<&contract::api::Route> = ROUTES
        .iter()
        .filter(|route| route.text_form.is_some())
        .collect();
    assert_eq!(
        with_text.len(),
        1,
        "the contract declares one text form; a new one needs its own check here"
    );
    let route = with_text[0];
    assert_eq!(route.path, "/v1/subtitles/{file}");
    assert_eq!(route.text_form, Some("text/vtt"));
    // The route is reached only with a subtitle service, which this harness has none of. What is
    // checked here is that the one declared text form is the one the handler looks for, so the
    // declaration and the code cannot drift apart silently.
    let handler =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/routes/play.rs"))
            .expect("play.rs");
    let form = route.text_form.expect("a text form");
    assert!(
        handler.contains(&format!("contains(\"{form}\")")),
        "{} declares {form} and the handler does not read that word",
        route.path
    );
    assert!(
        handler.contains(&format!("\"{form}; charset=utf-8\"")),
        "{} declares {form} and the handler does not answer in it",
        route.path
    );
}

#[tokio::test]
async fn h30_every_route_the_contract_marks_profile_refuses_a_member_with_no_profile() {
    let (server, _strings) = full_server();
    let (_, token) = common::member(&server.door).await;
    let mut refused = 0usize;
    for route in ROUTES.iter().filter(|route| route.auth == Auth::Profile) {
        let waiting = STILL_TO_BUILD
            .iter()
            .any(|(method, path, _)| *method == route.method && *path == route.path);
        if waiting {
            continue;
        }
        let method: Method = route.method.parse().expect("method");
        let mut request = call(method, &filled(route.path)).token(&token);
        if route.request.is_some() {
            request = request.json(serde_json::json!({}));
        }
        let answer = request.on(&server.state).await;
        assert_ne!(
            answer.status,
            StatusCode::OK,
            "{} {} needs an active profile and served a session that has none",
            route.method,
            route.path
        );
        assert_eq!(
            answer.status,
            StatusCode::FORBIDDEN,
            "{} {} needs an active profile: {} {}",
            route.method,
            route.path,
            answer.status,
            answer.body
        );
        assert_eq!(
            answer.code(),
            "profile_required",
            "{} {} refused for another reason than the missing profile",
            route.method,
            route.path
        );
        refused += 1;
    }
    assert_eq!(
        refused,
        ROUTES
            .iter()
            .filter(|route| route.auth == Auth::Profile)
            .count(),
        "every profile route is checked"
    );
}

/// Every route the router mounts outside `/v1`, with why the contract does not name it. The
/// promise in `api.rs` is bounded to the `/v1` surface a client speaks, and this list is the whole
/// of what that bound lets through.
const NOT_CLIENT_SURFACE: [(&str, &str, &str); 1] = [(
    "POST",
    "/internal/mail",
    "the auth service's send mail hook: it proves itself by signature, is off without \
     mail_hook_secret, and no generated client calls it",
)];

/// The HTTP methods one `.route(...)` call mounts, from the `get(..)`, `post(..)` and friends in
/// its handler argument. A verb that is part of a longer name, such as `account::delete`, is not
/// one of them.
fn methods(call: &str) -> Vec<String> {
    let mut list = Vec::new();
    for verb in ["get", "post", "put", "patch", "delete"] {
        let wanted = format!("{verb}(");
        let mut rest = call;
        while let Some(at) = rest.find(&wanted) {
            let before = rest[..at].chars().next_back();
            let inside_a_name = before
                .is_some_and(|letter| letter.is_alphanumeric() || letter == '_' || letter == ':');
            if !inside_a_name {
                list.push(verb.to_uppercase());
                break;
            }
            rest = &rest[at + wanted.len()..];
        }
    }
    list
}

/// Every `(method, path)` the one router mounts, read from the router's own source.
///
/// Axum hands out no list of what a `Router` holds, so the source is the only place this can be
/// read, as the text form test in this file reads `play.rs`. The parse proves itself twice: the
/// equality in the test wants every `/v1` route the contract names, so a parse that found nothing
/// cannot pass, and the ways of mounting a route that this parse cannot read are refused here, so
/// it cannot start lying by omission the day the router grows one.
fn mounted() -> BTreeSet<(String, String)> {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/router.rs"))
        .expect("the router's own source");
    for way in [".nest(", ".nest_service(", ".merge(", ".route_service("] {
        assert!(
            !source.contains(way),
            "the router mounts with {way}, which this parse does not read: teach it that before \
             trusting what it finds"
        );
    }
    let mut found = BTreeSet::new();
    for rest in source.split(".route(").skip(1) {
        let mut depth = 1usize;
        let call: String = rest
            .chars()
            .take_while(|letter| {
                match *letter {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                depth > 0
            })
            .collect();
        let path = call
            .split('"')
            .nth(1)
            .expect("the first argument of .route is its path")
            .to_owned();
        for method in methods(&call) {
            found.insert((method, path.clone()));
        }
    }
    found
}

/// The other direction of `u21`: not every declared route is mounted, but every mounted route is
/// declared. Nothing checked this way round before, which is how `POST /internal/mail` came to be
/// served without the contract naming it, for however long it has been there.
#[test]
fn u21_every_route_the_router_mounts_is_named_by_the_contract_or_listed_as_internal() {
    let mounted = mounted();
    assert!(
        mounted.len() > 60,
        "the router parse found {} routes, so it is not reading what it thinks it is",
        mounted.len()
    );
    let declared: BTreeSet<(String, String)> = ROUTES
        .iter()
        .filter(|route| route.group != Group::Finder)
        .filter(|route| {
            !STILL_TO_BUILD
                .iter()
                .any(|(method, path, _)| *method == route.method && *path == route.path)
        })
        .map(|route| (route.method.to_string(), route.path.to_string()))
        .collect();
    let surface: BTreeSet<(String, String)> = mounted
        .iter()
        .filter(|(_, path)| path.starts_with("/v1"))
        .cloned()
        .collect();
    let unnamed: Vec<&(String, String)> = surface.difference(&declared).collect();
    assert!(
        unnamed.is_empty(),
        "the router mounts {unnamed:?} under /v1 and the contract names none of them, so a \
         generated client cannot know they exist: name them in contract/src/api.rs"
    );
    let unmounted: Vec<&(String, String)> = declared.difference(&surface).collect();
    assert!(
        unmounted.is_empty(),
        "the contract names {unmounted:?} and the router mounts none of them: mount them, or put \
         them on STILL_TO_BUILD"
    );
    for (method, path) in mounted.iter().filter(|(_, path)| !path.starts_with("/v1")) {
        assert!(
            NOT_CLIENT_SURFACE
                .iter()
                .any(|(listed_method, listed_path, _)| listed_method == method
                    && listed_path == path),
            "the router mounts {method} {path}, which is neither named by the contract nor listed \
             as internal: say what it is in NOT_CLIENT_SURFACE, or name it in the contract"
        );
    }
    for (method, path, _) in NOT_CLIENT_SURFACE {
        assert!(
            !path.starts_with("/v1"),
            "{path} is on the internal list and is client surface: the contract must name it"
        );
        assert!(
            mounted.contains(&(method.to_owned(), path.to_owned())),
            "{method} {path} is listed as internal and is not mounted: take it off the list"
        );
    }
}

/// A concrete call for a route the contract opens to visitors, on the sample set. A route the
/// contract opens and nobody maps here fails the sweep below by name, as the cacheable set does
/// above. Every one of them owes a visitor an answer, so they all expect 200.
fn visitor_call(path: &str, cursor: &str) -> String {
    let series = server_catalog::sample::special::SHARED_ID;
    match path {
        "/v1/nav" => String::from("/v1/nav"),
        "/v1/screens/{name}" => String::from("/v1/screens/home"),
        "/v1/more/{cursor}" => format!("/v1/more/{cursor}"),
        // A U title, so the stated visitor tier does not refuse it for its age.
        "/v1/titles/{kind}/{id}" => String::from("/v1/titles/movie/106"),
        // The series of the shared id is rated U, so the same holds.
        "/v1/titles/{kind}/{id}/seasons/{n}" => format!("/v1/titles/tv/{series}/seasons/1"),
        "/v1/search" => String::from("/v1/search?q=first"),
        "/v1/search/suggest" => String::from("/v1/search/suggest?q=fir"),
        other => panic!(
            "{other} is open to visitors in the contract and has no call here: give it one, \
             because a client generated from the contract will call it with no token at all"
        ),
    }
}

/// The deployment the contract's visitor flag describes: no accounts required, at a tier the
/// deployer stated. The tier is 15 and not a children's tier because the sample has no series a
/// children's tier may see, and the season route is one of the seven.
#[tokio::test]
async fn a_visitor_is_served_every_route_the_contract_opens_and_no_other_profile_route() {
    let open = harness(&[("ACCOUNTS_REQUIRED", "false"), ("VISITOR_TIER", "15")]);
    let home = common::get("/v1/screens/home").on(&open.state).await;
    assert_eq!(home.status, StatusCode::OK, "{}", home.body);
    let cursor = home.body["cursor"]
        .as_str()
        .expect("a visitor's home screen pages, so /v1/more has a cursor to ask with")
        .to_owned();
    let mut served = 0usize;
    for route in ROUTES.iter().filter(|route| route.auth == Auth::Profile) {
        let waiting = STILL_TO_BUILD
            .iter()
            .any(|(method, path, _)| *method == route.method && *path == route.path);
        if waiting {
            continue;
        }
        if route.open_to_visitors {
            let path = visitor_call(route.path, &cursor);
            let answer = common::get(&path).on(&open.state).await;
            assert_eq!(
                answer.status,
                StatusCode::OK,
                "{path} is open to visitors in the contract and answered one {}: {}",
                answer.status,
                answer.body
            );
            served += 1;
        } else {
            let method: Method = route.method.parse().expect("method");
            let mut request = call(method, &filled(route.path));
            if route.request.is_some() {
                request = request.json(serde_json::json!({}));
            }
            let answer = request.on(&open.state).await;
            assert_ne!(
                answer.status,
                StatusCode::OK,
                "{} {} is not open to visitors in the contract and served one",
                route.method,
                route.path
            );
            assert!(
                matches!(
                    answer.status,
                    StatusCode::UNAUTHORIZED | StatusCode::BAD_REQUEST
                ),
                "{} {} refused a visitor for another reason than the missing token: {} {}",
                route.method,
                route.path,
                answer.status,
                answer.body
            );
        }
    }
    assert_eq!(
        served,
        ROUTES.iter().filter(|route| route.open_to_visitors).count(),
        "every route the contract opens to visitors is checked"
    );
}
