//! Notifications and reminders.
//!
//! A feed of arrival, coming, season and reminder notices for one profile.

use alloc::string::String;
use alloc::vec::Vec;

use crate::screen::{CalendarDay, Tile};

words! {
    /// What kind of notice this is.
    pub enum NoticeKind {
        NewArrival,
        Coming,
        ReminderLive,
        NewSeason,
        NewEpisode,
    }
}

wire! {
    /// One item in the notification feed.
    pub struct NoticeItem {
        pub id: String,
        pub kind: NoticeKind,
        pub tile: Tile,
        pub date: CalendarDay,
        pub unread: bool,
    }

    /// `GET /v1/notifications`.
    pub struct Notifications {
        pub items: Vec<NoticeItem>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub cursor: Option<String>,
        pub has_more: bool,
    }

    /// `GET /v1/notifications/count`.
    pub struct NotificationCount {
        pub unread: u32,
    }

    /// `POST /v1/notifications/read`.
    pub struct NotificationRead {
        pub up_to: String,
    }
}
