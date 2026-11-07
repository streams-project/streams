//! Help. The text lives once on the server, and every app draws the same articles from these.

use alloc::string::String;
use alloc::vec::Vec;

words! {
    /// What a block of an article is.
    pub enum HelpBlockKind {
        Paragraph,
        Heading,
        Steps,
        List,
        Note,
        /// An action the app offers here. An app without it leaves the block out.
        Tool,
    }
}

wire! {
    /// Some words of a line, with how they are marked.
    pub struct HelpRun {
        pub text: String,
        #[serde(default, skip_serializing_if = "core::ops::Not::not")]
        pub bold: bool,
        /// Where the words lead: an address in the app, a web page or a mail address.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub link: Option<String>,
    }

    /// One line of a block. A paragraph has one, a list has one per item.
    pub struct HelpLine {
        pub runs: Vec<HelpRun>,
    }

    pub struct HelpBlock {
        pub kind: HelpBlockKind,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub lines: Vec<HelpLine>,
        /// The action of a tool block.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub tool: Option<String>,
    }

    /// A part of a page. A folded one starts closed where the app can open and close it.
    pub struct HelpSection {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,
        #[serde(default, skip_serializing_if = "core::ops::Not::not")]
        pub folded: bool,
        pub blocks: Vec<HelpBlock>,
    }

    /// An article as a list shows it.
    pub struct HelpLink {
        pub slug: String,
        pub title: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub summary: Option<String>,
    }

    pub struct HelpGroup {
        pub id: String,
        pub title: String,
        pub articles: Vec<HelpLink>,
    }

    pub struct HelpTopic {
        pub id: String,
        pub title: String,
        pub groups: Vec<HelpGroup>,
    }

    pub struct HelpPolicyLink {
        pub id: String,
        pub title: String,
    }

    /// `/v1/help`.
    pub struct HelpHome {
        pub topics: Vec<HelpTopic>,
        pub popular: Vec<HelpLink>,
        pub policies: Vec<HelpPolicyLink>,
    }

    /// `/v1/help/articles/{slug}`.
    pub struct HelpArticle {
        pub slug: String,
        pub title: String,
        pub sections: Vec<HelpSection>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub related: Vec<HelpLink>,
    }

    /// `/v1/help/policies/{id}`.
    pub struct HelpPolicy {
        pub id: String,
        pub title: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub updated: Option<String>,
        pub sections: Vec<HelpSection>,
    }

    /// `/v1/help/search`.
    pub struct HelpResults {
        pub articles: Vec<HelpLink>,
    }
}
