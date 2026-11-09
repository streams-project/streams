//! The notification feed and unread count for a profile.

use std::collections::{BTreeMap, HashSet};

use contract::TitleKey;
use contract::notification::{NoticeItem, NoticeKind, NotificationCount, Notifications};
use contract::screen::CalendarDay;
use server_catalog::DataSet;
use server_catalog::catalog::item_key;
use server_catalog::fresh::ymd;
use server_catalog::seasons::SeasonStart;
use server_library::NotificationReadState;
use server_rules::answer::{Images, SafeTile, TileExtras};
use server_rules::{Viewer, Visible};

pub const NOTIFICATIONS_CAP: usize = 50;
pub const NEW_ARRIVALS_CAP: usize = 5;
pub const PAGE_SIZE: usize = 20;

/// Inputs for building notifications.
pub struct NotificationFeedAsk<'a> {
    pub data: &'a DataSet,
    pub viewer: &'a Viewer,
    pub today_days: i64,
    pub now_ms: u64,
    pub starts: &'a [SeasonStart],
    pub reminded: &'a [TitleKey],
    pub in_list: &'a [TitleKey],
    pub in_progress: &'a [TitleKey],
    pub read_state: &'a NotificationReadState,
    pub images: &'a Images,
    pub cursor: Option<&'a str>,
}

#[derive(Clone, Debug)]
struct Candidate<'a> {
    id: String,
    kind: NoticeKind,
    date: String,
    title_key: TitleKey,
    visible: Visible<'a>,
}

impl Candidate<'_> {
    fn priority(&self) -> u8 {
        match self.kind {
            NoticeKind::ReminderLive => 4,
            NoticeKind::NewSeason => 3,
            NoticeKind::NewEpisode => 2,
            NoticeKind::NewArrival => 1,
            NoticeKind::Coming => 0,
            _ => 0,
        }
    }
}

fn is_unread(
    notice_id: &str,
    notice_date: &str,
    read_state: &NotificationReadState,
    today: &str,
) -> bool {
    if read_state.dismissed.iter().any(|d| d == notice_id) {
        return false;
    }
    if read_state.read_up_to_ms == 0 {
        return true;
    }
    let read_up_to_days = (read_state.read_up_to_ms / 86_400_000) as i64;
    let read_up_to_date = ymd(read_up_to_days);
    if notice_date < read_up_to_date.as_str() {
        return false;
    }
    // Future dated notices (coming soon) are unread unless dismissed
    if notice_date > today {
        return true;
    }
    notice_date >= read_up_to_date.as_str()
}

/// Build the notification feed for one profile.
#[must_use]
pub fn build_notifications(ask: &NotificationFeedAsk<'_>) -> Notifications {
    let today = ymd(ask.today_days);
    let past_30 = ymd(ask.today_days - 30);
    let next_30 = ymd(ask.today_days + 30);
    let last_14 = ymd(ask.today_days - 14);

    let mut candidates: Vec<Candidate<'_>> = Vec::new();
    let list_set: HashSet<TitleKey> = ask.in_list.iter().copied().collect();
    let progress_set: HashSet<TitleKey> = ask.in_progress.iter().copied().collect();

    // 1. Reminders live (reminded titles whose release date is today or past)
    for &title in ask.reminded {
        let key = item_key(title.kind().word(), title.id());
        if let Some(visible) = ask.viewer.see(ask.data, key) {
            let release_date = ask
                .data
                .fresh
                .items
                .iter()
                .find(|f| f.media_type == title.kind().word() && f.id == title.id())
                .map(|f| f.release_date.clone())
                .unwrap_or_else(|| today.clone());
            if release_date <= today {
                candidates.push(Candidate {
                    id: format!("reminder:{}:{}", title.kind().word(), title.id()),
                    kind: NoticeKind::ReminderLive,
                    date: if release_date.len() == 10 {
                        release_date
                    } else {
                        today.clone()
                    },
                    title_key: title,
                    visible,
                });
            }
        }
    }

    // 2. New season (titles in list or progress whose season air date falls in window)
    for &title in list_set.union(&progress_set) {
        if title.kind() == contract::TitleKind::Tv {
            let key = item_key("tv", title.id());
            if let Some(visible) = ask.viewer.see(ask.data, key) {
                // Check fresh next_season
                if let Some(next) = ask
                    .data
                    .fresh
                    .items
                    .iter()
                    .find(|f| f.media_type == "tv" && f.id == title.id())
                    .and_then(|f| f.next_season.as_ref())
                    .filter(|next| next.air_date >= past_30 && next.air_date <= next_30)
                {
                    candidates.push(Candidate {
                        id: format!("season:tv:{}:{}", title.id(), next.number),
                        kind: NoticeKind::NewSeason,
                        date: next.air_date.clone(),
                        title_key: title,
                        visible,
                    });
                }
                // Check catalogue season starts
                for start in ask.starts.iter().filter(|s| s.series == title.id()) {
                    if start.air_date >= past_30 && start.air_date <= next_30 {
                        candidates.push(Candidate {
                            id: format!("season:tv:{}:{}", title.id(), start.season),
                            kind: NoticeKind::NewSeason,
                            date: start.air_date.clone(),
                            title_key: title,
                            visible,
                        });
                    }
                }
            }
        }
    }

    // 3. New arrivals (fresh titles released in the last 14 days, up to NEW_ARRIVALS_CAP)
    let mut arrivals: Vec<Candidate<'_>> = Vec::new();
    for fresh in &ask.data.fresh.items {
        if fresh.release_date >= last_14 && fresh.release_date <= today {
            let key = item_key(&fresh.media_type, fresh.id);
            if let Some(visible) = ask.viewer.see(ask.data, key) {
                let kind = if fresh.media_type == "tv" {
                    contract::TitleKind::Tv
                } else {
                    contract::TitleKind::Movie
                };
                if let Ok(title_key) = contract::TitleKey::new(kind, fresh.id) {
                    arrivals.push(Candidate {
                        id: format!("arrival:{}:{}", fresh.media_type, fresh.id),
                        kind: NoticeKind::NewArrival,
                        date: fresh.release_date.clone(),
                        title_key,
                        visible,
                    });
                }
            }
        }
    }
    arrivals.sort_by(|a, b| {
        b.date.cmp(&a.date).then_with(|| {
            b.visible
                .item()
                .popularity
                .total_cmp(&a.visible.item().popularity)
        })
    });
    arrivals.truncate(NEW_ARRIVALS_CAP);
    candidates.extend(arrivals);

    // 4. Coming titles (fresh titles with release_date > today and <= next_30)
    for fresh in &ask.data.fresh.items {
        if fresh.release_date > today && fresh.release_date <= next_30 {
            let key = item_key(&fresh.media_type, fresh.id);
            if let Some(visible) = ask.viewer.see(ask.data, key) {
                let kind = if fresh.media_type == "tv" {
                    contract::TitleKind::Tv
                } else {
                    contract::TitleKind::Movie
                };
                if let Ok(title_key) = contract::TitleKey::new(kind, fresh.id) {
                    candidates.push(Candidate {
                        id: format!("coming:{}:{}", fresh.media_type, fresh.id),
                        kind: NoticeKind::Coming,
                        date: fresh.release_date.clone(),
                        title_key,
                        visible,
                    });
                }
            }
        }
    }

    // Dedupe by title keeping highest priority kind
    let mut by_title: BTreeMap<TitleKey, Candidate<'_>> = BTreeMap::new();
    for c in candidates {
        match by_title.get(&c.title_key) {
            Some(existing) if existing.priority() >= c.priority() => {}
            _ => {
                by_title.insert(c.title_key, c);
            }
        }
    }

    // Order by date descending, then popularity, cap to NOTIFICATIONS_CAP (50)
    let mut list: Vec<Candidate<'_>> = by_title.into_values().collect();
    list.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| b.priority().cmp(&a.priority()))
            .then_with(|| {
                b.visible
                    .item()
                    .popularity
                    .total_cmp(&a.visible.item().popularity)
            })
    });
    list.truncate(NOTIFICATIONS_CAP);

    // Paging
    let offset = ask
        .cursor
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let total = list.len();
    let paged: Vec<Candidate<'_>> = list.into_iter().skip(offset).take(PAGE_SIZE).collect();
    let has_more = offset + paged.len() < total;
    let next_cursor = if has_more {
        Some((offset + paged.len()).to_string())
    } else {
        None
    };

    let items: Vec<NoticeItem> = paged
        .into_iter()
        .filter_map(|c| {
            let unread = is_unread(&c.id, &c.date, ask.read_state, &today);
            let safe_tile = SafeTile::new(c.visible, ask.images, TileExtras::default())?;
            let date = CalendarDay::parse(&c.date)
                .unwrap_or_else(|_| CalendarDay::parse(&today).expect("valid today"));
            Some(NoticeItem {
                id: c.id,
                kind: c.kind,
                tile: safe_tile.into_inner(),
                date,
                unread,
            })
        })
        .collect();

    Notifications {
        items,
        cursor: next_cursor,
        has_more,
    }
}

/// Calculate unread notification count for a profile.
#[must_use]
pub fn unread_notification_count(ask: &NotificationFeedAsk<'_>) -> NotificationCount {
    let notifs = build_notifications(&NotificationFeedAsk {
        cursor: None,
        ..*ask
    });
    let unread = notifs.items.iter().filter(|i| i.unread).count() as u32;
    NotificationCount { unread }
}
