//! The one list of what an account owns.
//!
//! Every table of `server/migrations` is named here exactly once, with the four facts a person is
//! entitled to know about it: what it is for, how long it lives, whether their answer about
//! personal rows governs the write, and whether the export carries it. A fifth fact, how deletion
//! reaches it, is here too, because a promise of deletion that is not in the same place as the
//! list of what we hold is a promise nobody can check.
//!
//! Two readers and one check keep this honest, and they are the reason this is code and not a
//! document:
//!
//! * `GET /v1/account/export` fills one section per [`Export::Under`] entry, and its own test
//!   fails when an entry has no section.
//! * the deletion path answers for every [`Removal::Erase`] entry, each of which names the
//!   function that removes it.
//! * [`tests`] reads the migration files: a table that is not in this list fails, an entry whose
//!   table does not exist fails, and a [`Removal::Cascade`] claim fails unless the foreign keys
//!   really do carry the row away with `auth.users`.
//!
//! The inventory this was written from, with the evidence per row, is `board/data.md`
//! (2026-11-11).

/// What names a person in a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyed {
    /// The account row itself, in `auth.users`.
    Account,
    /// A profile of an account.
    Profile,
    /// A device session of an account.
    Session,
    /// An e-mail address, with no account row behind it.
    Address,
    /// Nothing about a person.
    NoOne,
}

/// How long a row lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lives {
    /// Until the account or profile it belongs to goes. No prune exists.
    WhileHeld,
    /// Deleted this many days after its own time, by the hourly job.
    Days(u32),
    /// Kept to this many rows per profile, newest first.
    Newest(u32),
    /// Until it is spent or lapses.
    UntilSpent,
    /// Reference data of the build, not of a person.
    WithTheBuild,
}

/// Whether the person's answer about personal rows governs the write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    /// Written only while consent for personal rows is on, and removed when it is withdrawn.
    Consent,
    /// Written to do what the person asked for, whatever that answer is.
    Working,
}

/// Whether `GET /v1/account/export` carries the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Export {
    /// Carried, under this name in the body.
    Under(&'static str),
    /// Deliberately left out, with the reason. These are the only honest omissions: a secret we
    /// hold on the person's behalf, and rows that are not about a person at all.
    Never(&'static str),
    /// Held, and not carried yet, with what is missing. A person is entitled to these, so each
    /// one is an open item and not a decision.
    NotYet(&'static str),
}

/// How deletion of the account reaches the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    /// The foreign keys carry it away with the `auth.users` row, in the same statement. The test
    /// below proves the chain really cascades.
    Cascade,
    /// No key reaches it, so the deletion path removes it by name. The text names the function
    /// that does it, so a reader can check the claim.
    Erase(&'static str),
    /// Not a person's row. Deletion does not touch it.
    Keeps,
}

/// One table and what we hold in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    /// The table, as the migration names it.
    pub table: &'static str,
    /// What it is for, in the words a policy could reuse.
    pub purpose: &'static str,
    pub keyed_by: Keyed,
    pub lives: Lives,
    pub consent: Asked,
    pub export: Export,
    pub removal: Removal,
}

/// Every table the server holds, in migration order.
///
/// Adding a table to `server/migrations` without adding it here fails `cargo test -p
/// server-accounts`, which is what `./check` runs.
pub const HELD: &[Held] = &[
    // ---- 0000_auth.sql
    Held {
        table: "auth.users",
        purpose: "the account: its e-mail address, its phone number and the hash of its password",
        keyed_by: Keyed::Account,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("account"),
        removal: Removal::Cascade,
    },
    Held {
        table: "auth.login_codes",
        purpose: "the live sign-in or recovery code for one address, kept as a hash",
        keyed_by: Keyed::Address,
        lives: Lives::UntilSpent,
        consent: Asked::Working,
        export: Export::Never("the hash of a one-time code, which is a secret and not a record"),
        // The row is keyed by address and has no foreign key, so nothing carries it away.
        removal: Removal::Erase("BuiltInAuth::delete"),
    },
    // ---- 0001_accounts.sql
    Held {
        table: "account_details",
        purpose: "the names of the account holder, their answer about moving a profile to \
                  another account (profile_transfer, read by Door::transfer_profile, which no \
                  route writes yet), and two columns nothing reads at all: test_participation, \
                  an experiment opt-in that defaults to true, and rating_system",
        keyed_by: Keyed::Account,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        // The names go out inside `account`, beside the address they belong to. The two columns
        // nothing reads are not carried: their value is the schema default for every account, so
        // they would say nothing about this person. Either something reads them or a migration
        // drops them; that decision is in this package's report.
        export: Export::Under("account"),
        removal: Removal::Cascade,
    },
    Held {
        table: "rating_system_by_country",
        purpose: "which certificate system a country uses",
        keyed_by: Keyed::NoOne,
        lives: Lives::WithTheBuild,
        consent: Asked::Working,
        export: Export::Never("reference data of the deployment, the same for everyone"),
        removal: Removal::Keeps,
    },
    Held {
        table: "profiles",
        purpose: "the profiles of the account: name, picture, kind and maturity level",
        keyed_by: Keyed::Account,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("profiles"),
        removal: Removal::Cascade,
    },
    Held {
        table: "profile_pins",
        purpose: "the PIN of a profile, kept as a hash, with its lock counters",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Never("the hash of a secret the person holds, never handed back"),
        removal: Removal::Cascade,
    },
    Held {
        table: "profile_limits",
        purpose: "the age limit of a profile and the title names it never sees",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("profile_limits"),
        removal: Removal::Cascade,
    },
    Held {
        table: "account_events",
        purpose: "the audit log of the account: one row per sign-in or change, with its time",
        keyed_by: Keyed::Account,
        lives: Lives::Days(365),
        consent: Asked::Working,
        export: Export::Under("account_events"),
        removal: Removal::Cascade,
    },
    Held {
        table: "account_devices",
        purpose: "the key a device keeps for itself, which names it across sessions",
        keyed_by: Keyed::Account,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::NotYet(
            "written by one statement and read by no code, with 11 of its 15 columns never \
             written; narrow it to what is used or export it, which is an open decision",
        ),
        removal: Removal::Cascade,
    },
    Held {
        table: "device_sessions",
        purpose: "one row per signed-in device: its name, the hashes of its tokens, the active \
                  profile, the last step up, the consent answer and the capability card the \
                  device sent",
        keyed_by: Keyed::Account,
        lives: Lives::Days(30),
        consent: Asked::Working,
        export: Export::Under("devices"),
        removal: Removal::Cascade,
    },
    Held {
        table: "device_codes",
        purpose: "a television waiting to be let in: the code it shows and the name it gave",
        keyed_by: Keyed::Account,
        lives: Lives::UntilSpent,
        consent: Asked::Working,
        export: Export::Never("a one-time code in flight, gone when it is used or lapses"),
        removal: Removal::Cascade,
    },
    // ---- 0100_library.sql
    Held {
        table: "title_state",
        purpose: "My List and every rating, each with its own version",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("library"),
        removal: Removal::Cascade,
    },
    Held {
        table: "episode_progress",
        purpose: "what was watched and how far, per film and per episode",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("progress"),
        removal: Removal::Cascade,
    },
    Held {
        table: "hidden_titles",
        purpose: "titles hidden from a profile's rows",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("hidden"),
        removal: Removal::Cascade,
    },
    Held {
        table: "profile_library",
        purpose: "the change number of a profile, so devices can catch up, and the single writer \
                  lock",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Never("a counter of the sync mechanism, with nothing about the person"),
        removal: Removal::Cascade,
    },
    Held {
        table: "applied_ops",
        purpose: "the id of each change a device sent, so sending it twice changes nothing once",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::NotYet(
            "op ids with their times, which is an activity trail; no read exists and no prune \
             exists either, and both are open items",
        ),
        removal: Removal::Cascade,
    },
    Held {
        table: "profile_settings",
        purpose: "autoplay, languages, subtitle style, and the gender and handle a profile chose",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("settings"),
        removal: Removal::Cascade,
    },
    Held {
        table: "refused_trailers",
        purpose: "trailers that failed to play for this profile, so the next choice skips them",
        keyed_by: Keyed::Profile,
        lives: Lives::Newest(300),
        // Ledger P43: a lasting refusal code from the player reaches the server as this command.
        // It is the record of a failure, not of a choice, so the consent answer does not govern
        // it and turning consent off would only make trailers fail again.
        consent: Asked::Working,
        export: Export::NotYet(
            "a technical record keyed to the profile; no read exists, and whether a person wants \
             it in their file is an open decision",
        ),
        removal: Removal::Cascade,
    },
    Held {
        table: "profile_taste",
        purpose: "the taste the server inferred from what the profile watched",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Consent,
        export: Export::Under("taste"),
        removal: Removal::Cascade,
    },
    Held {
        table: "feed_events",
        purpose: "what a profile was shown and what it did with it: impression, click, start, \
                  complete, like, skip and list changes",
        keyed_by: Keyed::Profile,
        lives: Lives::Days(180),
        consent: Asked::Consent,
        export: Export::NotYet(
            "impression-level rows kept 180 days; no read exists on either store, and no policy \
             text describes them, so the disclosure comes first",
        ),
        removal: Removal::Cascade,
    },
    // ---- 0110_play.sql
    Held {
        table: "reachability",
        purpose: "how often playback reached a host, by client class",
        keyed_by: Keyed::NoOne,
        lives: Lives::WithTheBuild,
        consent: Asked::Working,
        export: Export::Never("counts per host and class, with no person in them"),
        removal: Removal::Keeps,
    },
    Held {
        table: "play_events",
        purpose: "what happened during a play: the event, its time, the attempt, how long the \
                  first frame took, any failure, and the host it came from",
        keyed_by: Keyed::Profile,
        lives: Lives::Days(30),
        consent: Asked::Consent,
        export: Export::NotYet(
            "timing and failure samples keyed to the profile; the log lives in server-play, \
             which this package does not own, so the read is a handoff",
        ),
        removal: Removal::Cascade,
    },
    Held {
        table: "client_reports",
        purpose: "a fault a client reported: its build, a code and its own words",
        keyed_by: Keyed::NoOne,
        lives: Lives::Days(30),
        consent: Asked::Working,
        export: Export::Never(
            "no account or profile column exists, so no row can be matched to a \
                               person",
        ),
        removal: Removal::Keeps,
    },
    // ---- 0120_notifications.sql
    Held {
        table: "profile_reminders",
        purpose: "titles a profile asked to be reminded about",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("reminders"),
        removal: Removal::Cascade,
    },
    Held {
        table: "notification_reads",
        purpose: "how far a profile has read its notifications, and the ones it dismissed",
        keyed_by: Keyed::Profile,
        lives: Lives::WhileHeld,
        consent: Asked::Working,
        export: Export::Under("notification_reads"),
        removal: Removal::Cascade,
    },
];

/// The tables the export carries, with the name each one goes out under.
#[must_use]
pub fn exported() -> Vec<(&'static str, &'static str)> {
    HELD.iter()
        .filter_map(|held| match held.export {
            Export::Under(name) => Some((held.table, name)),
            Export::Never(_) | Export::NotYet(_) => None,
        })
        .collect()
}

/// The names the export body carries, each from one entry of [`HELD`]. The export route builds
/// exactly these keys, and its test compares the two sets, so a table added to the list without a
/// section fails.
#[must_use]
pub fn export_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = exported().into_iter().map(|(_, name)| name).collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// The tables no key carries away, with the function that removes each one. The deletion path
/// must answer for every row of this list.
#[must_use]
pub fn erased_by_hand() -> Vec<(&'static str, &'static str)> {
    HELD.iter()
        .filter_map(|held| match held.removal {
            Removal::Erase(by) => Some((held.table, by)),
            Removal::Cascade | Removal::Keeps => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::{Asked, Export, HELD, Keyed, Removal};

    /// Where the migration chain lives, relative to this crate.
    fn migrations() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
    }

    /// One foreign key: the parent table and what `on delete` says.
    #[derive(Clone, Debug)]
    struct Link {
        name: String,
        parent: String,
        cascades: bool,
    }

    /// The whole chain in file order, as a map from table to its foreign keys. Column level keys
    /// take the name Postgres gives them, so a later `alter table ... drop constraint` removes
    /// the right one.
    fn schema() -> BTreeMap<String, Vec<Link>> {
        let mut files: Vec<PathBuf> = std::fs::read_dir(migrations())
            .expect("the migrations folder")
            .filter_map(|entry| {
                let path = entry.expect("a migration").path();
                (path.extension()?.to_str()? == "sql").then_some(path)
            })
            .collect();
        files.sort();
        let mut tables: BTreeMap<String, Vec<Link>> = BTreeMap::new();
        for path in files {
            let text = std::fs::read_to_string(&path).expect("a migration");
            read_file(&strip_comments(&text), &mut tables);
        }
        tables
    }

    fn strip_comments(text: &str) -> String {
        text.lines()
            .map(|line| line.split("--").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Every statement of one file, in order.
    fn read_file(text: &str, tables: &mut BTreeMap<String, Vec<Link>>) {
        for statement in text.split(';') {
            let flat = statement.split_whitespace().collect::<Vec<_>>().join(" ");
            let lower = flat.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("create table ") {
                let rest = rest.strip_prefix("if not exists ").unwrap_or(rest);
                let name = rest
                    .split(['(', ' '])
                    .next()
                    .expect("a table name")
                    .to_owned();
                let body = &lower[lower.find('(').expect("a table body") + 1..];
                tables.insert(name.clone(), links_of(&name, body));
            } else if let Some(rest) = lower.strip_prefix("alter table ") {
                let mut words = rest.split_whitespace();
                let name = words.next().expect("a table name").to_owned();
                let held = tables.entry(name.clone()).or_default();
                if let Some(at) = rest.find("drop constraint ") {
                    let after = &rest[at + "drop constraint ".len()..];
                    let after = after.strip_prefix("if exists ").unwrap_or(after);
                    let dropped = after
                        .split_whitespace()
                        .next()
                        .expect("a constraint name")
                        .to_owned();
                    held.retain(|link| link.name != dropped);
                }
                if rest.contains(" foreign key ") {
                    let named = rest.find("add constraint ").map(|at| {
                        rest[at + "add constraint ".len()..]
                            .split_whitespace()
                            .next()
                            .expect("a constraint name")
                            .to_owned()
                    });
                    if let Some(link) = reference_in(rest) {
                        held.push(Link {
                            name: named.unwrap_or_else(|| link.0.clone()),
                            parent: link.0,
                            cascades: link.1,
                        });
                    }
                }
            }
        }
    }

    /// The column level foreign keys inside a `create table` body.
    fn links_of(table: &str, body: &str) -> Vec<Link> {
        let mut found = Vec::new();
        for line in body.split(',') {
            if let Some(reference) = reference_in(line) {
                let column = line.split_whitespace().next().unwrap_or("column");
                found.push(Link {
                    name: format!("{}_{column}_fkey", table.replace('.', "_")),
                    parent: reference.0,
                    cascades: reference.1,
                });
            }
        }
        found
    }

    /// The parent table and whether the row goes with it, for one fragment holding `references`.
    fn reference_in(fragment: &str) -> Option<(String, bool)> {
        let at = fragment.find("references ")?;
        let rest = &fragment[at + "references ".len()..];
        let parent = rest.split(['(', ' ']).next()?.to_owned();
        Some((parent, rest.contains("on delete cascade")))
    }

    /// True when the row is carried away with the account row by foreign keys alone.
    fn cascades_to_account(tables: &BTreeMap<String, Vec<Link>>, table: &str) -> bool {
        let Some(links) = tables.get(table) else {
            return false;
        };
        links.iter().any(|link| {
            link.cascades
                && (link.parent == "auth.users" || cascades_to_account(tables, &link.parent))
        })
    }

    #[test]
    fn every_table_in_the_schema_is_named_once_in_the_list() {
        let tables = schema();
        let listed: Vec<&str> = HELD.iter().map(|held| held.table).collect();
        for table in tables.keys() {
            assert!(
                listed.contains(&table.as_str()),
                "{table} is in the migrations and in no entry of HELD: say what it holds, how \
                 long it lives, whether consent governs it, whether the export carries it, and \
                 how deletion reaches it"
            );
        }
        for held in HELD {
            assert!(
                tables.contains_key(held.table),
                "HELD names {}, which no migration creates",
                held.table
            );
        }
        let mut once = listed.clone();
        once.sort_unstable();
        once.dedup();
        assert_eq!(once.len(), listed.len(), "a table is listed twice");
        assert_eq!(
            listed.len(),
            tables.len(),
            "the list and the schema hold a different number of tables"
        );
    }

    #[test]
    fn a_cascade_claim_is_true_of_the_foreign_keys() {
        let tables = schema();
        for held in HELD {
            match held.removal {
                Removal::Cascade => assert!(
                    held.table == "auth.users" || cascades_to_account(&tables, held.table),
                    "{} claims deletion reaches it through the foreign keys, and no cascading \
                     chain from it to auth.users exists: either the key is `on delete cascade` or \
                     the entry says Erase and the deletion path removes the rows by name",
                    held.table
                ),
                Removal::Erase(by) => {
                    assert!(
                        !cascades_to_account(&tables, held.table),
                        "{} is removed by {by} and the foreign keys already carry it away, so the \
                         entry should say Cascade",
                        held.table
                    );
                    assert!(!by.is_empty(), "{} names no remover", held.table);
                }
                Removal::Keeps => {
                    assert_eq!(
                        held.keyed_by,
                        Keyed::NoOne,
                        "{} is kept past deletion, so it must hold nothing about a person",
                        held.table
                    );
                    assert!(
                        !cascades_to_account(&tables, held.table),
                        "{} is kept past deletion and its rows are carried away anyway",
                        held.table
                    );
                }
            }
        }
    }

    #[test]
    fn every_entry_says_something_in_every_field() {
        for held in HELD {
            assert!(
                held.purpose.len() > 20,
                "{} does not say what it is for",
                held.table
            );
            match held.export {
                Export::Under(name) => assert!(!name.is_empty()),
                Export::Never(reason) | Export::NotYet(reason) => assert!(
                    reason.len() > 20,
                    "{} leaves the export with no reason",
                    held.table
                ),
            }
            if held.consent == Asked::Consent {
                assert_ne!(
                    held.keyed_by,
                    Keyed::NoOne,
                    "{} cannot be governed by consent and hold nobody",
                    held.table
                );
            }
        }
    }
}
