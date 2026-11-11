//! The article format. A body is plain text, so a translator touches almost no markup:
//!
//! ```text
//! blank line      ends a paragraph or list
//! ## Heading      a heading
//! 1. text         a step, and an unmarked line after it continues the step
//! - text          a bullet
//! !! text         a note
//! >> Title        a folded section that runs to the next >> or the end
//! @@ name         an action the app offers
//! <b>text</b>     bold
//! <a=/path>text</a>  a link
//! ```

use contract::help::{HelpBlock, HelpBlockKind, HelpLine, HelpRun, HelpSection};

enum Open {
    Nothing,
    Paragraph(String),
    Lines(HelpBlockKind, Vec<String>),
}

fn step_text(line: &str) -> Option<&str> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    line[digits..].strip_prefix(". ")
}

fn run(text: &str) -> HelpRun {
    HelpRun {
        text: text.to_owned(),
        bold: false,
        link: None,
    }
}

/// The marked runs of one line. A tag with no end stays as plain words.
pub(crate) fn runs(line: &str) -> Vec<HelpRun> {
    let mut out = Vec::new();
    let mut rest = line;
    loop {
        let bold = rest.find("<b>");
        let link = rest.find("<a=");
        let at = match (bold, link) {
            (Some(b), Some(a)) => b.min(a),
            (Some(found), None) | (None, Some(found)) => found,
            (None, None) => break,
        };
        if at > 0 {
            out.push(run(&rest[..at]));
        }
        let after = &rest[at..];
        if let Some(inside) = after.strip_prefix("<b>") {
            let Some(end) = inside.find("</b>") else {
                out.push(run(after));
                return out;
            };
            out.push(HelpRun {
                bold: true,
                ..run(&inside[..end])
            });
            rest = &inside[end + 4..];
        } else {
            let found = after[3..].find('>').and_then(|close| {
                let body = &after[3 + close + 1..];
                body.find("</a>").map(|end| (close, end))
            });
            let Some((close, end)) = found else {
                out.push(run(after));
                return out;
            };
            let body = &after[3 + close + 1..];
            out.push(HelpRun {
                link: Some(after[3..3 + close].to_owned()),
                ..run(&body[..end])
            });
            rest = &body[end + 4..];
        }
    }
    if !rest.is_empty() {
        out.push(run(rest));
    }
    out
}

fn line(text: &str) -> HelpLine {
    HelpLine { runs: runs(text) }
}

fn block(kind: HelpBlockKind, lines: &[String]) -> HelpBlock {
    HelpBlock {
        kind,
        lines: lines.iter().map(|text| line(text)).collect(),
        tool: None,
    }
}

fn close(open: &mut Open, blocks: &mut Vec<HelpBlock>) {
    match std::mem::replace(open, Open::Nothing) {
        Open::Nothing => {}
        Open::Paragraph(text) => blocks.push(block(HelpBlockKind::Paragraph, &[text])),
        Open::Lines(kind, lines) => blocks.push(block(kind, &lines)),
    }
}

fn add_line(open: &mut Open, blocks: &mut Vec<HelpBlock>, kind: HelpBlockKind, text: &str) {
    if let Open::Lines(held, lines) = open
        && *held == kind
    {
        lines.push(text.to_owned());
        return;
    }
    close(open, blocks);
    *open = Open::Lines(kind, vec![text.to_owned()]);
}

fn continue_text(open: &mut Open, text: &str) {
    match open {
        Open::Paragraph(held) => {
            held.push(' ');
            held.push_str(text);
        }
        Open::Lines(_, lines) => {
            if let Some(last) = lines.last_mut() {
                last.push(' ');
                last.push_str(text);
            }
        }
        Open::Nothing => *open = Open::Paragraph(text.to_owned()),
    }
}

/// A body as sections: what comes before the first `>>`, then one folded section for each.
#[must_use]
pub fn parse(body: &str) -> Vec<HelpSection> {
    let mut sections = vec![HelpSection {
        title: None,
        folded: false,
        blocks: Vec::new(),
    }];
    let mut open = Open::Nothing;
    for raw in body.lines() {
        let text = raw.trim();
        let Some(section) = sections.last_mut() else {
            continue;
        };
        if text.is_empty() {
            close(&mut open, &mut section.blocks);
        } else if let Some(title) = text.strip_prefix(">>") {
            close(&mut open, &mut section.blocks);
            sections.push(HelpSection {
                title: Some(title.trim().to_owned()),
                folded: true,
                blocks: Vec::new(),
            });
        } else if let Some(tool) = text.strip_prefix("@@ ") {
            close(&mut open, &mut section.blocks);
            section.blocks.push(HelpBlock {
                kind: HelpBlockKind::Tool,
                lines: Vec::new(),
                tool: Some(tool.trim().to_owned()),
            });
        } else if let Some(title) = text.strip_prefix("## ") {
            close(&mut open, &mut section.blocks);
            section
                .blocks
                .push(block(HelpBlockKind::Heading, &[title.to_owned()]));
        } else if let Some(note) = text.strip_prefix("!! ") {
            close(&mut open, &mut section.blocks);
            section
                .blocks
                .push(block(HelpBlockKind::Note, &[note.to_owned()]));
        } else if let Some(step) = step_text(text) {
            add_line(&mut open, &mut section.blocks, HelpBlockKind::Steps, step);
        } else if let Some(item) = text.strip_prefix("- ") {
            add_line(&mut open, &mut section.blocks, HelpBlockKind::List, item);
        } else {
            continue_text(&mut open, text);
        }
    }
    if let Some(section) = sections.last_mut() {
        close(&mut open, &mut section.blocks);
    }
    sections.retain(|section| section.folded || !section.blocks.is_empty());
    sections
}

/// The words of a body with every tag and marker gone, for searching.
#[must_use]
pub fn plain(body: &str) -> String {
    let mut words = Vec::new();
    for raw in body.lines() {
        let text = raw.trim();
        if text.starts_with("@@ ") {
            continue;
        }
        let text = ["## ", "!! ", "- ", ">>"]
            .iter()
            .find_map(|mark| text.strip_prefix(mark))
            .or_else(|| step_text(text))
            .unwrap_or(text);
        words.extend(runs(text).into_iter().map(|run| run.text));
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(section: &HelpSection) -> Vec<HelpBlockKind> {
        section.blocks.iter().map(|block| block.kind).collect()
    }

    #[test]
    fn a_body_becomes_blocks_in_order() {
        let body = "First line\nsame paragraph\n\n## Head\n1. One\ncontinues\n2. Two\n- Dot\n!! Mind: this\n@@ diagnostics";
        let sections = parse(body);
        assert_eq!(sections.len(), 1);
        assert_eq!(
            kinds(&sections[0]),
            [
                HelpBlockKind::Paragraph,
                HelpBlockKind::Heading,
                HelpBlockKind::Steps,
                HelpBlockKind::List,
                HelpBlockKind::Note,
                HelpBlockKind::Tool,
            ]
        );
        assert_eq!(
            sections[0].blocks[0].lines[0].runs[0].text,
            "First line same paragraph"
        );
        assert_eq!(sections[0].blocks[2].lines.len(), 2);
        assert_eq!(sections[0].blocks[2].lines[0].runs[0].text, "One continues");
        assert_eq!(sections[0].blocks[5].tool.as_deref(), Some("diagnostics"));
    }

    #[test]
    fn a_fold_runs_to_the_next_fold() {
        let sections = parse("Intro\n>> One\nIn one\n>> Two\nIn two");
        assert_eq!(sections.len(), 3);
        assert!(!sections[0].folded);
        assert_eq!(sections[1].title.as_deref(), Some("One"));
        assert!(sections[2].folded);
    }

    #[test]
    fn bold_and_links_are_marked_and_a_broken_tag_stays_as_words() {
        let line = line("See <b>this</b> and <a=/help>that</a> now");
        let texts: Vec<_> = line.runs.iter().map(|run| run.text.as_str()).collect();
        assert_eq!(texts, ["See ", "this", " and ", "that", " now"]);
        assert!(line.runs[1].bold);
        assert_eq!(line.runs[3].link.as_deref(), Some("/help"));
        assert_eq!(runs("open <b>never closed")[1].text, "<b>never closed");
    }

    #[test]
    fn plain_leaves_only_the_words() {
        assert_eq!(
            plain("## Head\n1. <b>One</b>\n@@ tool\n- <a=/x>Two</a>"),
            "Head One Two"
        );
    }
}
