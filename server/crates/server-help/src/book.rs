//! The files of a help folder. English holds the tree and the order. Another language gives its
//! words where it has them, so a missing translation shows English instead of a gap.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use contract::hello::{PLACEHOLDER_BRAND, PLACEHOLDER_SUPPORT_EMAIL};
use contract::help::{
    HelpArticle, HelpBlock, HelpBlockKind, HelpGroup, HelpHome, HelpLine, HelpLink, HelpPolicy,
    HelpPolicyLink, HelpResults, HelpRun, HelpSection, HelpTopic,
};
use serde::Deserialize;

use crate::markup::{parse, plain};

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
}

#[derive(Debug, Default, Deserialize)]
struct PolicySection {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    bullets: Vec<String>,
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
            let book = serde_json::from_slice(&bytes)
                .map_err(|error| HelpError::File(code.to_owned(), error))?;
            books.insert(code.to_owned(), book);
        }
        if !books.contains_key(FALLBACK) {
            return Err(HelpError::NoEnglish);
        }
        Ok(Self { books })
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
        let sections = english
            .sections
            .iter()
            .enumerate()
            .map(|(at, section)| {
                view.policy_section(section, local.and_then(|p| p.sections.get(at)))
            })
            .collect();
        Some(HelpPolicy {
            id: id.to_owned(),
            title: view.policy_title(id),
            updated: local
                .and_then(|policy| policy.updated.as_deref())
                .or(english.updated.as_deref())
                .map(|text| named.fill(text)),
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
    fn article_text(&self, slug: &str, pick: impl Fn(&Article) -> Option<&str>) -> String {
        let text = self
            .book
            .articles
            .get(slug)
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

    fn policy_section(
        &self,
        english: &PolicySection,
        local: Option<&PolicySection>,
    ) -> HelpSection {
        let chosen = local.unwrap_or(english);
        let body = chosen
            .body
            .as_deref()
            .or(english.body.as_deref())
            .unwrap_or_default();
        let mut blocks: Vec<HelpBlock> = parse(&self.named.fill(body))
            .into_iter()
            .flat_map(|section| section.blocks)
            .collect();
        let bullets = if chosen.bullets.is_empty() {
            &english.bullets
        } else {
            &chosen.bullets
        };
        if !bullets.is_empty() {
            blocks.push(HelpBlock {
                kind: HelpBlockKind::List,
                lines: bullets
                    .iter()
                    .map(|text| HelpLine {
                        runs: vec![HelpRun {
                            text: self.named.fill(text),
                            bold: false,
                            link: None,
                        }],
                    })
                    .collect(),
                tool: None,
            });
        }
        HelpSection {
            title: chosen
                .title
                .as_deref()
                .or(english.title.as_deref())
                .map(|text| self.named.fill(text)),
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
