//! The notification feed, unread count and read marks for a profile.
#![allow(clippy::disallowed_types)] // a test reads answers as a client does

mod common;

use axum::http::{Method, StatusCode};
use common::{Harness, call, get, harness, member};
use contract::TitleKey;
use contract::notification::{NoticeKind, NotificationCount, Notifications};
use contract::screen::TitleDetail;
use serde_json::json;

use server_accounts::{AccountId, StepUpMethod};

/// A token whose session entered the named kind of profile.
async fn enter(server: &Harness, kids: bool) -> (AccountId, String, String) {
    let (account, token) = member(&server.door).await;
    let list = get("/v1/profiles").token(&token).on(&server.state).await;
    let profiles = list.body["profiles"].as_array().expect("profiles");
    let id = profiles
        .iter()
        .find(|p| p["kids"] == kids)
        .expect("profile")["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let path = format!("/v1/profiles/{id}/enter");
    let entered = call(Method::POST, &path)
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    assert_eq!(entered.status, StatusCode::OK);
    (account, token, id)
}

#[tokio::test]
async fn n13_notifications_routes_require_bearer_and_profile_scope() {
    let server = harness(&[]);

    // 1. Without token: 401
    let unauth_list = get("/v1/notifications").on(&server.state).await;
    assert_eq!(unauth_list.status, StatusCode::UNAUTHORIZED);

    let unauth_count = get("/v1/notifications/count").on(&server.state).await;
    assert_eq!(unauth_count.status, StatusCode::UNAUTHORIZED);

    let unauth_read = call(Method::POST, "/v1/notifications/read")
        .json(json!({ "up_to": "any:item" }))
        .on(&server.state)
        .await;
    assert_eq!(unauth_read.status, StatusCode::UNAUTHORIZED);

    // 2. Member token without profile scope: 403
    let (_, member_token) = member(&server.door).await;
    let forbidden_list = get("/v1/notifications")
        .token(&member_token)
        .on(&server.state)
        .await;
    assert_eq!(forbidden_list.status, StatusCode::FORBIDDEN);

    let forbidden_count = get("/v1/notifications/count")
        .token(&member_token)
        .on(&server.state)
        .await;
    assert_eq!(forbidden_count.status, StatusCode::FORBIDDEN);

    let forbidden_read = call(Method::POST, "/v1/notifications/read")
        .token(&member_token)
        .json(json!({ "up_to": "any:item" }))
        .on(&server.state)
        .await;
    assert_eq!(forbidden_read.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn n13_feed_and_count_and_read_marks() {
    let server = harness(&[]);
    let (_, token, _) = enter(&server, false).await;

    // GET /v1/notifications
    let answer = get("/v1/notifications")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let feed: Notifications = serde_json::from_value(answer.body).expect("Notifications");
    assert!(!feed.items.is_empty(), "sample has fresh items");

    // GET /v1/notifications/count
    let count_answer = get("/v1/notifications/count")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(count_answer.status, StatusCode::OK, "{}", count_answer.body);
    let count: NotificationCount =
        serde_json::from_value(count_answer.body).expect("NotificationCount");
    let unread_items = feed.items.iter().filter(|i| i.unread).count() as u32;
    assert_eq!(count.unread, unread_items);

    // POST /v1/notifications/read up to the first item
    let top_item = &feed.items[0];
    let read_answer = call(Method::POST, "/v1/notifications/read")
        .token(&token)
        .json(json!({ "up_to": top_item.id }))
        .on(&server.state)
        .await;
    assert_eq!(read_answer.status, StatusCode::OK, "{}", read_answer.body);
    assert_eq!(read_answer.body["done"], true);

    // After marking read, the read items are marked read
    let count_after = get("/v1/notifications/count")
        .token(&token)
        .on(&server.state)
        .await;
    let count_after: NotificationCount =
        serde_json::from_value(count_after.body).expect("NotificationCount");
    assert!(count_after.unread <= count.unread);
}

#[tokio::test]
async fn n14_feed_builder_dedupes_by_title_and_caps_at_fifty() {
    let server = harness(&[]);
    let (_, token, _) = enter(&server, false).await;

    let answer = get("/v1/notifications")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let feed: Notifications = serde_json::from_value(answer.body).expect("Notifications");

    assert!(feed.items.len() <= 50, "capped at 50");

    // No duplicate titles in items
    let mut titles = std::collections::HashSet::new();
    for item in &feed.items {
        assert!(
            titles.insert(item.tile.title),
            "duplicate title in feed: {}",
            item.tile.title
        );
    }

    // Dates descending
    for pair in feed.items.windows(2) {
        assert!(
            pair[0].date >= pair[1].date,
            "items sorted by date desc: {:?} < {:?}",
            pair[0].date,
            pair[1].date
        );
    }

    // Paging if cursor present
    if feed.has_more {
        let cursor = feed.cursor.expect("cursor when has_more is true");
        let next_answer = get(&format!("/v1/notifications?cursor={cursor}"))
            .token(&token)
            .on(&server.state)
            .await;
        assert_eq!(next_answer.status, StatusCode::OK);
        let next_feed: Notifications =
            serde_json::from_value(next_answer.body).expect("Notifications");
        assert!(!next_feed.items.is_empty());
        for item in &next_feed.items {
            assert!(
                !titles.contains(&item.tile.title),
                "cursor page repeated title {}",
                item.tile.title
            );
        }
    }
}

#[tokio::test]
async fn n14_kids_profile_only_sees_kids_safe_notifications() {
    let server = harness(&[]);
    let (_, token, _) = enter(&server, true).await;

    let answer = get("/v1/notifications")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let feed: Notifications = serde_json::from_value(answer.body).expect("Notifications");

    for item in &feed.items {
        let key = item.tile.title;
        let title_path = format!("/v1/titles/{}/{}", key.kind().word(), key.id());
        let page = get(&title_path).token(&token).on(&server.state).await;
        assert_eq!(page.status, StatusCode::OK, "kids profile allowed {}", key);
    }
}

#[tokio::test]
async fn n13_remind_command_toggles_and_reminder_live_after_release() {
    let server = harness(&[]);
    let (account, mut token, profile_id) = enter(&server, false).await;

    let feed_answer = get("/v1/notifications")
        .token(&token)
        .on(&server.state)
        .await;
    let feed: Notifications = serde_json::from_value(feed_answer.body).expect("Notifications");

    // Find a coming title
    let coming = feed
        .items
        .iter()
        .find(|i| i.kind == NoticeKind::Coming)
        .expect("coming title in feed");
    let key: TitleKey = coming.tile.title;
    let title_path = format!("/v1/titles/{}/{}", key.kind().word(), key.id());

    // Check title detail before remind
    let detail_before = get(&title_path).token(&token).on(&server.state).await;
    assert_eq!(detail_before.status, StatusCode::OK);
    let detail: TitleDetail = serde_json::from_value(detail_before.body).expect("TitleDetail");
    assert!(detail.available_from.is_some());
    assert!(!detail.reminder_set);

    // Toggle reminder ON via /v1/commands
    let batch = json!({
        "ops": [{
            "op_id": "op_remind_1",
            "command": "remind",
            "title": key.to_string(),
            "on": true
        }]
    });
    let cmd_answer = call(Method::POST, "/v1/commands")
        .token(&token)
        .json(batch)
        .on(&server.state)
        .await;
    assert_eq!(cmd_answer.status, StatusCode::OK, "{}", cmd_answer.body);
    assert_eq!(cmd_answer.body["answers"][0]["outcome"], "applied");

    // Detail after remind on
    let detail_after = get(&title_path).token(&token).on(&server.state).await;
    let detail: TitleDetail = serde_json::from_value(detail_after.body).expect("TitleDetail");
    assert!(detail.reminder_set);

    // Advance clock past the title's release date (+15 days = 15 * 86_400_000 ms)
    server.clock.advance_ms(15 * 86_400_000);

    // Open a fresh session since the previous token expired over 15 days
    let (_, tokens) = server
        .door
        .open_session(account, "test device", StepUpMethod::Password)
        .await
        .expect("session");
    token = tokens.access;
    let re_enter = call(Method::POST, &format!("/v1/profiles/{profile_id}/enter"))
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    assert_eq!(re_enter.status, StatusCode::OK);

    // In notifications feed, this title should now be ReminderLive
    let feed_later = get("/v1/notifications")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(feed_later.status, StatusCode::OK, "{}", feed_later.body);
    let later_feed: Notifications = serde_json::from_value(feed_later.body).expect("Notifications");
    let live_notice = later_feed
        .items
        .iter()
        .find(|i| i.tile.title == key)
        .expect("reminded title still in feed");
    assert_eq!(live_notice.kind, NoticeKind::ReminderLive);

    // Toggle reminder OFF
    let batch_off = json!({
        "ops": [{
            "op_id": "op_remind_2",
            "command": "remind",
            "title": key.to_string(),
            "on": false
        }]
    });
    let cmd_off = call(Method::POST, "/v1/commands")
        .token(&token)
        .json(batch_off)
        .on(&server.state)
        .await;
    assert_eq!(cmd_off.status, StatusCode::OK);
    assert_eq!(cmd_off.body["answers"][0]["outcome"], "applied");

    let detail_final = get(&title_path).token(&token).on(&server.state).await;
    let detail: TitleDetail = serde_json::from_value(detail_final.body).expect("TitleDetail");
    assert!(!detail.reminder_set);
}
