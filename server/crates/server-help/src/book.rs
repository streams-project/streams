//! The files of a help folder. English holds the tree and the order. Another language gives its
//! words where it has them, so a missing translation shows English instead of a gap.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use contract::hello::{PLACEHOLDER_BRAND, PLACEHOLDER_SUPPORT_EMAIL};
use contract::help::{
    HelpArticle, HelpBlock, HelpBlockKind, HelpGroup, HelpHome, HelpLine, HelpLink, HelpPolicy,
    HelpPolicyLink, HelpResults, HelpSection, HelpTopic,
};
use serde::Deserialize;

use crate::markup::{parse, plain, runs};

const FALLBACK: &str = "en";
const RESULTS: usize = 20;

#[derive(Debug, thiserror::Error)]
pub enum HelpError {
    #[error("the help folder could not be read: {0}")]
    Folder(#[from] std::io::Error),
    #[error("the help file {0} does not read: {1}")]
    File(String, serde_json::Error),
    #[error("the help folder has no English file")]
    NoEnglish,
    #[error("{0}: policy {1}, section {2} has {3} bullets and {4} table lists")]
    Tables(String, String, String, usize, usize),
}

#[derive(Debug, Default, Deserialize)]
struct Group {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    articles: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Topic {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    groups: Vec<Group>,
}

#[derive(Debug, Default, Deserialize)]
struct Article {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    keywords: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    related: Vec<String>,
    /// The fingerprint of the English body this translation was written from. A translated body
    /// is served while it matches, and the English one is served when it does not, because a
    /// correction to the English text is usually a correction of fact: the old translation would
    /// go on telling a reader what the product no longer does. A book that carries no fingerprint
    /// at all is read as it always was.
    #[serde(default, rename = "from")]
    from_english: Option<String>,
}

/// The fingerprint of one body: FNV-1a over its bytes. Short, stable, and written by
/// `help/` tooling in the same shape, so a translator can copy the English one.
fn fingerprint(text: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[derive(Debug, Default, Deserialize)]
struct PolicySection {
    /// The name this section is known by in every book. Required, because a section is paired
    /// across books by it: a file that leaves it out does not read at all.
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    bullets: Vec<String>,
    /// Which database tables each bullet accounts for, bullet by bullet. Only the English
    /// retention section carries it, and only a check reads it: it is what ties the words to
    /// `server-accounts::holdings::HELD` so neither can move without the other.
    #[serde(default)]
    tables: Vec<Vec<String>>,
}

impl PolicySection {
    fn says(text: Option<&String>) -> bool {
        text.is_some_and(|words| !words.trim().is_empty())
    }

    /// Whether this section can be served on its own, without borrowing from the English one of
    /// the same id. A heading with no words under it, or a list the English section has and this
    /// one does not, is half a section, and half a section is where the scrambling came from.
    fn whole(&self, english: &Self) -> bool {
        Self::says(self.title.as_ref())
            && Self::says(self.body.as_ref())
            && (english.bullets.is_empty() || !self.bullets.is_empty())
    }
}

/// A policy section a book does not carry, or carries only half of. Named rather than patched
/// over: a reader gets the whole English section, and a test says which books are short.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub language: String,
    pub policy: String,
    /// The section id, or the policy id itself when the whole policy is absent.
    pub section: String,
    /// What is wrong: `missing`, `half written`, or `not in English`.
    pub reason: &'static str,
}

#[derive(Debug, Default, Deserialize)]
struct Policy {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    sections: Vec<PolicySection>,
}

#[derive(Debug, Default, Deserialize)]
struct Book {
    #[serde(default)]
    topics: Vec<Topic>,
    #[serde(default)]
    popular: Vec<String>,
    #[serde(default)]
    articles: BTreeMap<String, Article>,
    #[serde(default)]
    policies: BTreeMap<String, Policy>,
}

/// What the text says the product and its address are.
#[derive(Clone, Copy, Debug)]
pub struct Named<'a> {
    pub brand: &'a str,
    pub support_email: &'a str,
}

impl Named<'_> {
    fn fill(&self, text: &str) -> String {
        text.replace(PLACEHOLDER_BRAND, self.brand)
            .replace(PLACEHOLDER_SUPPORT_EMAIL, self.support_email)
    }
}

/// Every language of a help folder, held in memory.
#[derive(Debug)]
pub struct Help {
    books: HashMap<String, Book>,
}

/// One reader's view: a language and the names to fill in.
struct View<'a> {
    english: &'a Book,
    book: &'a Book,
    named: Named<'a>,
}

impl Help {
    /// Reads every `<lang>.json` in a folder.
    ///
    /// # Errors
    /// A folder or file that does not read, or a folder with no English file.
    pub fn open(folder: &Path) -> Result<Self, HelpError> {
        let mut books = HashMap::new();
        for entry in std::fs::read_dir(folder)? {
            let path = entry?.path();
            if path.extension().is_none_or(|end| end != "json") {
                continue;
            }
            let Some(code) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let bytes = std::fs::read(&path)?;
            let book: Book = serde_json::from_slice(&bytes)
                .map_err(|error| HelpError::File(code.to_owned(), error))?;
            books.insert(code.to_owned(), book);
        }
        if !books.contains_key(FALLBACK) {
            return Err(HelpError::NoEnglish);
        }
        // A bullet's tables are read bullet by bullet, so a list of a different length would tie
        // a sentence to the wrong rows. The folder does not open at all in that case.
        for (code, book) in &books {
            for (policy, held) in &book.policies {
                for section in &held.sections {
                    if !section.tables.is_empty() && section.tables.len() != section.bullets.len() {
                        return Err(HelpError::Tables(
                            code.clone(),
                            policy.clone(),
                            section.id.clone(),
                            section.bullets.len(),
                            section.tables.len(),
                        ));
                    }
                }
            }
        }
        Ok(Self { books })
    }

    /// Every policy section a book other than English does not carry whole, and every section a
    /// book carries that English does not. Empty means every reader sees their own language.
    ///
    /// This is the list `policy` acts on: a gap is served as the whole English section, never as
    /// one book's heading over another book's words.
    #[must_use]
    pub fn gaps(&self) -> Vec<Gap> {
        let Some(english) = self.books.get(FALLBACK) else {
            return Vec::new();
        };
        let mut codes: Vec<&String> = self.books.keys().filter(|code| *code != FALLBACK).collect();
        codes.sort();
        let mut gaps = Vec::new();
        for code in codes {
            let book = &self.books[code];
            let mut gap = |policy: &str, section: &str, reason| {
                gaps.push(Gap {
                    language: code.clone(),
                    policy: policy.to_owned(),
                    section: section.to_owned(),
                    reason,
                });
            };
            for (id, policy) in &english.policies {
                let Some(local) = book.policies.get(id) else {
                    gap(id, id, "missing");
                    continue;
                };
                for section in &policy.sections {
                    match local.sections.iter().find(|held| held.id == section.id) {
                        None => gap(id, &section.id, "missing"),
                        Some(held) if !held.whole(section) => gap(id, &section.id, "half written"),
                        Some(_) => {}
                    }
                }
                for held in &local.sections {
                    if !policy.sections.iter().any(|known| known.id == held.id) {
                        gap(id, &held.id, "not in English");
                    }
                }
            }
            for id in book.policies.keys() {
                if !english.policies.contains_key(id) {
                    gap(id, id, "not in English");
                }
            }
        }
        gaps
    }

    /// The tables each bullet of one English policy section accounts for, in bullet order.
    ///
    /// The retention section of the privacy policy is written against
    /// `server-accounts::holdings::HELD`, and this is how a check reads the tie: every table that
    /// package holds is named by exactly one bullet here, and the bullet says that table's life.
    #[must_use]
    pub fn tables(&self, policy: &str, section: &str) -> Vec<Vec<String>> {
        self.books
            .get(FALLBACK)
            .and_then(|book| book.policies.get(policy))
            .map(|held| {
                held.sections
                    .iter()
                    .find(|one| one.id == section)
                    .map(|one| one.tables.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// Every article whose translation names an English body that has since changed, as
    /// (language, slug). Those articles are served in English until the words catch up.
    #[must_use]
    pub fn stale(&self) -> Vec<(String, String)> {
        let Some(english) = self.books.get(FALLBACK) else {
            return Vec::new();
        };
        let mut codes: Vec<&String> = self.books.keys().filter(|code| *code != FALLBACK).collect();
        codes.sort();
        let mut found = Vec::new();
        for code in codes {
            let book = &self.books[code];
            for (slug, article) in &book.articles {
                let Some(mark) = article.from_english.as_deref() else {
                    continue;
                };
                let body = english
                    .articles
                    .get(slug)
                    .and_then(|known| known.body.as_deref());
                if body.is_some_and(|body| fingerprint(body) != mark) {
                    found.push((code.clone(), slug.clone()));
                }
            }
        }
        found
    }

    fn view<'a>(&'a self, language: Option<&str>, named: Named<'a>) -> Option<View<'a>> {
        let english = self.books.get(FALLBACK)?;
        let book = language
            .and_then(|code| self.books.get(code))
            .unwrap_or(english);
        Some(View {
            english,
            book,
            named,
        })
    }

    /// The topics, popular articles and policies.
    #[must_use]
    pub fn home(&self, language: Option<&str>, named: Named<'_>) -> Option<HelpHome> {
        let view = self.view(language, named)?;
        Some(HelpHome {
            topics: view
                .english
                .topics
                .iter()
                .map(|topic| view.topic(topic))
                .collect(),
            popular: view.links(&view.english.popular),
            policies: view
                .english
                .policies
                .keys()
                .map(|id| HelpPolicyLink {
                    id: id.clone(),
                    title: view.policy_title(id),
                })
                .collect(),
        })
    }

    /// One article, or nothing when no such article exists.
    #[must_use]
    pub fn article(
        &self,
        slug: &str,
        language: Option<&str>,
        named: Named<'_>,
    ) -> Option<HelpArticle> {
        let view = self.view(language, named)?;
        let article = view.english.articles.get(slug)?;
        let body = view.article_text(slug, |text| text.body.as_deref());
        Some(HelpArticle {
            slug: slug.to_owned(),
            title: view.title(slug),
            sections: parse(&body),
            related: view.links(&article.related),
        })
    }

    /// One policy, or nothing when no such policy exists.
    #[must_use]
    pub fn policy(&self, id: &str, language: Option<&str>, named: Named<'_>) -> Option<HelpPolicy> {
        let view = self.view(language, named)?;
        let english = view.english.policies.get(id)?;
        let local = view.book.policies.get(id);
        let mut all_local = local.is_some();
        let mut sections = Vec::with_capacity(english.sections.len());
        for section in &english.sections {
            let held = local
                .and_then(|policy| policy.sections.iter().find(|held| held.id == section.id))
                .filter(|held| held.whole(section));
            all_local &= held.is_some();
            sections.push(view.policy_section(section, held));
        }
        // The date belongs to the words. A translation that is short of a section is read in
        // English for that section, so it is the English date that applies.
        let updated = if all_local {
            local
                .and_then(|policy| policy.updated.as_deref())
                .or(english.updated.as_deref())
        } else {
            english.updated.as_deref()
        };
        Some(HelpPolicy {
            id: id.to_owned(),
            title: view.policy_title(id),
            updated: updated.map(|text| named.fill(text)),
            sections,
        })
    }

    /// Articles holding every word of a search, the best matches first.
    #[must_use]
    pub fn search(
        &self,
        query: &str,
        language: Option<&str>,
        named: Named<'_>,
    ) -> Option<HelpResults> {
        let view = self.view(language, named)?;
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        let mut found: Vec<(u32, &String)> = view
            .english
            .articles
            .keys()
            .filter_map(|slug| view.score(slug, &words).map(|score| (score, slug)))
            .collect();
        found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        let slugs: Vec<String> = found
            .into_iter()
            .take(RESULTS)
            .map(|(_, slug)| slug.clone())
            .collect();
        Some(HelpResults {
            articles: view.links(&slugs),
        })
    }
}

impl View<'_> {
    /// Whether this book's own words for an article may be served. They may not when the book
    /// says which English body it was written from and that body has since changed.
    fn current(&self, slug: &str) -> bool {
        let Some(local) = self.book.articles.get(slug) else {
            return true;
        };
        let Some(mark) = local.from_english.as_deref() else {
            return true;
        };
        self.english
            .articles
            .get(slug)
            .and_then(|article| article.body.as_deref())
            .is_none_or(|body| fingerprint(body) == mark)
    }

    fn article_text(&self, slug: &str, pick: impl Fn(&Article) -> Option<&str>) -> String {
        let local = self
            .current(slug)
            .then(|| self.book.articles.get(slug))
            .flatten();
        let text = local
            .and_then(&pick)
            .or_else(|| self.english.articles.get(slug).and_then(&pick))
            .unwrap_or_default();
        self.named.fill(text)
    }

    fn title(&self, slug: &str) -> String {
        self.article_text(slug, |text| text.title.as_deref())
    }

    fn link(&self, slug: &str) -> Option<HelpLink> {
        self.english.articles.contains_key(slug).then(|| {
            let summary = self.article_text(slug, |text| text.summary.as_deref());
            HelpLink {
                slug: slug.to_owned(),
                title: self.title(slug),
                summary: (!summary.is_empty()).then_some(summary),
            }
        })
    }

    fn links(&self, slugs: &[String]) -> Vec<HelpLink> {
        slugs.iter().filter_map(|slug| self.link(slug)).collect()
    }

    fn topic(&self, topic: &Topic) -> HelpTopic {
        let local = self.book.topics.iter().find(|held| held.id == topic.id);
        let title = local
            .and_then(|held| held.title.as_deref())
            .or(topic.title.as_deref());
        HelpTopic {
            id: topic.id.clone(),
            title: self.named.fill(title.unwrap_or_default()),
            groups: topic
                .groups
                .iter()
                .map(|group| {
                    let held =
                        local.and_then(|held| held.groups.iter().find(|one| one.id == group.id));
                    let title = held
                        .and_then(|one| one.title.as_deref())
                        .or(group.title.as_deref());
                    HelpGroup {
                        id: group.id.clone(),
                        title: self.named.fill(title.unwrap_or_default()),
                        articles: self.links(&group.articles),
                    }
                })
                .collect(),
        }
    }

    fn policy_title(&self, id: &str) -> String {
        let title = self
            .book
            .policies
            .get(id)
            .and_then(|policy| policy.title.as_deref())
            .or_else(|| {
                self.english
                    .policies
                    .get(id)
                    .and_then(|policy| policy.title.as_deref())
            });
        self.named.fill(title.unwrap_or_default())
    }

    /// One section, taken whole from one book. `local` is already proved whole by its caller, so
    /// nothing here mixes two books: a heading never carries another language's words.
    fn policy_section(
        &self,
        english: &PolicySection,
        local: Option<&PolicySection>,
    ) -> HelpSection {
        let chosen = local.unwrap_or(english);
        let body = chosen.body.as_deref().unwrap_or_default();
        let mut blocks: Vec<HelpBlock> = parse(&self.named.fill(body))
            .into_iter()
            .flat_map(|section| section.blocks)
            .collect();
        let bullets = &chosen.bullets;
        if !bullets.is_empty() {
            blocks.push(HelpBlock {
                kind: HelpBlockKind::List,
                // A bullet is marked up the way a body line is. Before this it was one plain
                // run, so every <b> in every book was shown to the reader as letters.
                lines: bullets
                    .iter()
                    .map(|text| HelpLine {
                        runs: runs(&self.named.fill(text)),
                    })
                    .collect(),
                tool: None,
            });
        }
        HelpSection {
            title: chosen.title.as_deref().map(|text| self.named.fill(text)),
            folded: false,
            blocks,
        }
    }

    /// How well an article fits: a word in the title counts most. Nothing when a word is missing.
    fn score(&self, slug: &str, words: &[String]) -> Option<u32> {
        if words.is_empty() {
            return None;
        }
        let title = self.title(slug).to_lowercase();
        let keywords = self
            .article_text(slug, |text| text.keywords.as_deref())
            .to_lowercase();
        let summary = self
            .article_text(slug, |text| text.summary.as_deref())
            .to_lowercase();
        let body = plain(&self.article_text(slug, |text| text.body.as_deref())).to_lowercase();
        words.iter().try_fold(0, |total, word| {
            let points = if title.contains(word.as_str()) {
                8
            } else if keywords.contains(word.as_str()) {
                4
            } else if summary.contains(word.as_str()) {
                2
            } else if body.contains(word.as_str()) {
                1
            } else {
                return None;
            };
            Some(total + points)
        })
    }
}
