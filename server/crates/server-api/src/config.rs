//! Settings. Each one is optional: a missing setting turns its feature off, the server says so at
//! start and in `/v1/ready`, and nothing else stops. No address, name or key is built in.

use std::collections::BTreeMap;
use std::path::PathBuf;

use server_rules::Tier;

/// Every setting the server reads, with what a missing one turns off. The loader reads no name
/// this table leaves out, and a test fails on any that is only in one of the two.
pub const SETTINGS: &[(&str, &str)] = &[
    (
        "SERVER_LISTEN",
        "The address to bind. Without it the server does not listen.",
    ),
    (
        "DATA_MANIFEST",
        "Path of the data manifest. Without it titles, pages and search are off.",
    ),
    (
        "DATA_PUBLIC_KEYS",
        "Comma separated paths of the PEM keys that sign a data set. Without them no set can be \
         checked, so titles, pages and search are off.",
    ),
    (
        "INDEX_PATH",
        "Path of the index file. Without it pages use the exact scan.",
    ),
    (
        "DATABASE_URL",
        "Postgres address. Without it accounts, library and play are off.",
    ),
    (
        "IMAGE_BASE",
        "Base address of images. Without it tiles carry no images.",
    ),
    (
        "IMAGE_BASE_ART",
        "Base address for gate art. Without it art defaults to w1280 of IMAGE_BASE.",
    ),
    (
        "IMAGE_SIZE_POSTER",
        "Size of posters. Without it posters use w342.",
    ),
    (
        "IMAGE_SIZE_BACKDROP",
        "Size of backdrops. Without it backdrops use w1280.",
    ),
    (
        "IMAGE_SIZE_STILL",
        "Size of stills. Without it stills use w300.",
    ),
    (
        "IMAGE_SIZE_LOGO",
        "Size of logos. Without it logos use w500.",
    ),
    (
        "IMAGE_SIZE_AVATAR",
        "Size of avatars. Without it avatars use w185.",
    ),
    (
        "FINDER_URLS",
        "Comma separated finder addresses. Without them playback is off.",
    ),
    (
        "FINDER_TOKEN",
        "Service token for finders. Without it playback is off.",
    ),
    (
        "WEB_ROOT",
        "Folder of the built website. Without it this server answers the API only.",
    ),
    (
        "SITE_ADDRESS",
        "Public address of this site, used in mail links and the device page. Without it device \
         sign in is off.",
    ),
    (
        "MAIL_HOOK_SECRET",
        "Shared secret of the mail hook. Without it mail is off.",
    ),
    (
        "MAIL_ADDRESS",
        "Address of the mail provider. Without it mail is off.",
    ),
    (
        "MAIL_TOKEN",
        "Service token of the mail provider. Without it mail is off.",
    ),
    (
        "MAIL_SENDER",
        "The sender address mail goes out as. Without it mail is off.",
    ),
    (
        "HELP_ADDRESS",
        "Address of the help pages. Without it help links are left out.",
    ),
    (
        "BRAND_NAME",
        "The name shown to viewers. Without it the name is empty.",
    ),
    (
        "SUPPORT_EMAIL",
        "The contact shown to viewers. Without it the contact is empty.",
    ),
    (
        "VISITOR_TIER",
        "Age tier of signed out viewers: U, PG, 12 or 15. Without it public titles are off.",
    ),
    (
        "ACCOUNTS_REQUIRED",
        "Set to false, with a visitor tier, to let visitors browse. Without it accounts stay \
         required.",
    ),
    (
        "LIVE_SETTINGS",
        "Path of the live settings file. Without it the defaults apply.",
    ),
    (
        "STRINGS_DIR",
        "Folder of interface text as <lang>.json. Without it strings are off.",
    ),
    (
        "HELP_DIR",
        "Folder of help text as <lang>.json. Without it help is off.",
    ),
    (
        "SUBTITLE_URL",
        "Address of a subtitle program, called with FINDER_TOKEN. Without it subtitle files are off.",
    ),
    (
        "CLIENT_ADDRESS_HEADER",
        "The one header the front proxy sets. Without it the peer address is used.",
    ),
    (
        "BUILD_ID",
        "The build shown in errors and ready. Without it the crate version is used.",
    ),
];

/// One feature the server could not turn on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Off {
    pub feature: &'static str,
    pub reason: String,
}

/// What the server starts with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub listen: Option<String>,
    pub data_manifest: Option<PathBuf>,
    pub data_keys: Vec<PathBuf>,
    pub index_path: Option<PathBuf>,
    pub database_url: Option<String>,
    pub image_base: Option<String>,
    pub image_base_art: Option<String>,
    pub image_size_poster: Option<String>,
    pub image_size_backdrop: Option<String>,
    pub image_size_still: Option<String>,
    pub image_size_logo: Option<String>,
    pub image_size_avatar: Option<String>,
    pub finder_urls: Vec<String>,
    pub finder_token: Option<String>,
    /// A subtitle program. Off without it.
    pub subtitle_url: Option<String>,
    pub site_address: Option<String>,
    pub web_root: Option<String>,
    pub mail_hook_secret: Option<String>,
    pub mail_address: Option<String>,
    pub mail_token: Option<String>,
    pub mail_sender: Option<String>,
    pub help_address: Option<String>,
    pub brand_name: String,
    pub support_email: String,
    pub visitor_tier: Option<Tier>,
    pub accounts_required: bool,
    pub live_settings: Option<PathBuf>,
    pub strings_dir: Option<PathBuf>,
    pub help_dir: Option<PathBuf>,
    pub address_header: Option<String>,
    pub build: String,
    /// Features that are off, each with its reason.
    pub off: Vec<Off>,
}

/// Tier word of a visitor. An adult tier needs a person, so `18` is refused.
fn visitor_tier(word: &str) -> Option<Tier> {
    Tier::from_rating(word).filter(|tier| *tier != Tier::Adult)
}

impl Config {
    /// Read settings from a lookup, normally the environment. Values are never logged.
    #[must_use]
    pub fn read(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let value = |name: &str| {
            lookup(name)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let mut off = Vec::new();
        let mut note = |feature: &'static str, reason: &str| {
            off.push(Off {
                feature,
                reason: reason.to_owned(),
            });
        };
        let listen = value("SERVER_LISTEN");
        if listen.is_none() {
            note("listen", "SERVER_LISTEN is not set");
        }
        let data_manifest = value("DATA_MANIFEST").map(PathBuf::from);
        if data_manifest.is_none() {
            note("titles", "DATA_MANIFEST is not set");
        }
        let data_keys: Vec<PathBuf> = value("DATA_PUBLIC_KEYS")
            .map(|list| {
                list.split(',')
                    .map(|path| PathBuf::from(path.trim()))
                    .collect()
            })
            .unwrap_or_default();
        if data_keys.is_empty() {
            note(
                "titles",
                "DATA_PUBLIC_KEYS is not set, so no set can be checked and none is served",
            );
        }
        let index_path = value("INDEX_PATH").map(PathBuf::from);
        if index_path.is_none() {
            note("index", "INDEX_PATH is not set, pages use the exact scan");
        }
        let database_url = value("DATABASE_URL");
        if database_url.is_none() {
            note("accounts", "DATABASE_URL is not set");
        }
        let image_base = value("IMAGE_BASE");
        if image_base.is_none() {
            note("images", "IMAGE_BASE is not set");
        }
        let image_base_art = value("IMAGE_BASE_ART");
        let image_size_poster = value("IMAGE_SIZE_POSTER");
        let image_size_backdrop = value("IMAGE_SIZE_BACKDROP");
        let image_size_still = value("IMAGE_SIZE_STILL");
        let image_size_logo = value("IMAGE_SIZE_LOGO");
        let image_size_avatar = value("IMAGE_SIZE_AVATAR");
        let finder_urls: Vec<String> = value("FINDER_URLS")
            .map(|list| {
                list.split(',')
                    .map(|url| url.trim().to_owned())
                    .filter(|url| !url.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let finder_token = value("FINDER_TOKEN");
        if finder_urls.is_empty() || finder_token.is_none() {
            note("playback", "FINDER_URLS or FINDER_TOKEN is not set");
        }
        let web_root = value("WEB_ROOT");
        if web_root.is_none() {
            note("website", "WEB_ROOT is not set");
        }
        let site_address = value("SITE_ADDRESS");
        if site_address.is_none() {
            note("device sign in", "SITE_ADDRESS is not set");
        }
        let mail_hook_secret = value("MAIL_HOOK_SECRET");
        let mail_address = value("MAIL_ADDRESS");
        let mail_token = value("MAIL_TOKEN");
        let mail_sender = value("MAIL_SENDER");
        let mail_ready = mail_hook_secret.is_some()
            && mail_address.is_some()
            && mail_token.is_some()
            && mail_sender.is_some();
        if !mail_ready {
            note(
                "mail",
                "MAIL_HOOK_SECRET, MAIL_ADDRESS, MAIL_TOKEN or MAIL_SENDER is not set",
            );
        }
        let brand_name = value("BRAND_NAME").unwrap_or_default();
        if brand_name.is_empty() {
            note("brand", "BRAND_NAME is not set");
        }
        let support_email = value("SUPPORT_EMAIL").unwrap_or_default();
        if support_email.is_empty() {
            note("support", "SUPPORT_EMAIL is not set");
        }
        let tier_word = value("VISITOR_TIER");
        let visitor_tier = tier_word.as_deref().and_then(visitor_tier);
        if visitor_tier.is_none() {
            let reason = if tier_word.is_some() {
                "VISITOR_TIER must be U, PG, 12 or 15"
            } else {
                "VISITOR_TIER is not set"
            };
            note("public titles", reason);
        }
        let asked_open =
            value("ACCOUNTS_REQUIRED").is_some_and(|v| v.eq_ignore_ascii_case("false"));
        if asked_open && visitor_tier.is_none() {
            note("visitors", "ACCOUNTS_REQUIRED=false needs a visitor tier");
        }
        let live_settings = value("LIVE_SETTINGS").map(PathBuf::from);
        let subtitle_url = value("SUBTITLE_URL");
        let help_dir = value("HELP_DIR").map(PathBuf::from);
        if help_dir.is_none() {
            note("help", "HELP_DIR is not set");
        }
        let strings_dir = value("STRINGS_DIR").map(PathBuf::from);
        if strings_dir.is_none() {
            note("strings", "STRINGS_DIR is not set");
        }
        Self {
            listen,
            subtitle_url,
            data_manifest,
            data_keys,
            index_path,
            database_url,
            image_base,
            image_base_art,
            image_size_poster,
            image_size_backdrop,
            image_size_still,
            image_size_logo,
            image_size_avatar,
            finder_urls,
            finder_token,
            site_address,
            web_root,
            mail_hook_secret,
            mail_address,
            mail_token,
            mail_sender,
            help_address: value("HELP_ADDRESS"),
            brand_name,
            support_email,
            visitor_tier,
            accounts_required: !(asked_open && visitor_tier.is_some()),
            live_settings,
            strings_dir,
            help_dir,
            address_header: value("CLIENT_ADDRESS_HEADER").map(|h| h.to_ascii_lowercase()),
            build: value("BUILD_ID").unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned()),
            off,
        }
    }

    /// Settings from the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self::read(|name| std::env::var(name).ok())
    }

    /// Settings from pairs, for tests and fixtures.
    #[must_use]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let map: BTreeMap<String, String> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect();
        Self::read(|name| map.get(name).cloned())
    }

    /// True when playback has what it needs.
    #[must_use]
    pub fn playback_ready(&self) -> bool {
        !self.finder_urls.is_empty() && self.finder_token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    /// Every name the loader asks its lookup for, with the lookup answering nothing and then
    /// answering a value, so a name read only on one branch is still seen.
    fn names_read() -> BTreeSet<String> {
        let seen = std::cell::RefCell::new(BTreeSet::new());
        for answer in [None, Some("x")] {
            let _ = Config::read(|name| {
                seen.borrow_mut().insert(name.to_owned());
                answer.map(str::to_owned)
            });
        }
        seen.into_inner()
    }

    #[test]
    fn s213_the_table_lists_every_setting_the_loader_reads_and_no_other() {
        let read = names_read();
        let listed: BTreeSet<String> = SETTINGS
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .collect();
        let missing: Vec<&String> = read.difference(&listed).collect();
        assert!(
            missing.is_empty(),
            "the loader reads settings the table leaves out: {missing:?}"
        );
        let unread: Vec<&String> = listed.difference(&read).collect();
        assert!(
            unread.is_empty(),
            "the table lists settings the loader never reads: {unread:?}"
        );
        assert_eq!(
            listed.len(),
            SETTINGS.len(),
            "the table names a setting twice"
        );
        for (name, what) in SETTINGS {
            assert!(
                what.len() > 20 && what.contains("Without"),
                "{name} does not say what a missing value turns off"
            );
        }
    }

    #[test]
    fn s213_a_missing_setting_turns_its_feature_off_and_says_so() {
        let config = Config::from_pairs(&[]);
        let names: Vec<&str> = config.off.iter().map(|off| off.feature).collect();
        for feature in [
            "titles",
            "accounts",
            "images",
            "playback",
            "public titles",
            "brand",
        ] {
            assert!(names.contains(&feature), "{feature} is not reported off");
        }
        assert!(
            config.accounts_required,
            "S153: accounts stay required by default"
        );
        assert!(config.listen.is_none(), "no default address exists");
    }

    #[test]
    fn tmk14_only_a_stated_tier_opens_visitors_and_adult_is_never_one() {
        let open = Config::from_pairs(&[("ACCOUNTS_REQUIRED", "false")]);
        assert!(open.accounts_required, "no tier, so accounts stay required");
        let adult = Config::from_pairs(&[("ACCOUNTS_REQUIRED", "false"), ("VISITOR_TIER", "18")]);
        assert!(adult.accounts_required && adult.visitor_tier.is_none());
        let pg = Config::from_pairs(&[("ACCOUNTS_REQUIRED", "false"), ("VISITOR_TIER", "PG")]);
        assert!(!pg.accounts_required);
        assert_eq!(pg.visitor_tier, Some(Tier::OlderKids));
    }

    #[test]
    fn empty_values_count_as_missing() {
        let config = Config::from_pairs(&[("DATA_MANIFEST", "  "), ("FINDER_URLS", " , ")]);
        assert!(config.data_manifest.is_none());
        assert!(config.finder_urls.is_empty());
        assert!(!config.playback_ready());
    }
}
