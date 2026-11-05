//! Reads Rust source for what the core and the offline layer may not hold.

/// One thing found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub word: String,
    pub why: &'static str,
    pub line: usize,
}

const BANNED: &[(&str, &str)] = &[
    ("std", "std"),
    ("async", "async"),
    ("await", "async"),
    ("Future", "async"),
    ("future", "async"),
    ("task", "async"),
    ("Poll", "async"),
    ("Waker", "async"),
    ("thread", "thread"),
    ("thread_local", "thread"),
    ("spawn", "thread"),
    ("Mutex", "thread"),
    ("Instant", "clock"),
    ("SystemTime", "clock"),
    ("UNIX_EPOCH", "clock"),
    ("Date", "clock"),
    ("File", "IO"),
    ("OpenOptions", "IO"),
    ("TcpStream", "IO"),
    ("TcpListener", "IO"),
    ("UdpSocket", "IO"),
    ("stdin", "IO"),
    ("stdout", "IO"),
    ("stderr", "IO"),
    ("getrandom", "random"),
    ("unsafe", "unsafe"),
    ("asm", "unsafe"),
    ("global_asm", "unsafe"),
    ("wasm_import_module", "unsafe"),
];

const ADDRESS_MARKS: &[&str] = &["http://", "https://", "ws://", "wss://", "ftp://"];

/// Blank out comments, strings and chars, keeping line numbers, so docs may name these words.
#[must_use]
pub fn code_only(text: &str) -> String {
    let bytes: Vec<char> = text.chars().collect();
    let mut kept = String::with_capacity(text.len());
    let mut at = 0;
    let keep_newlines = |from: usize, to: usize, kept: &mut String| {
        for &c in &bytes[from..to.min(bytes.len())] {
            if c == '\n' {
                kept.push('\n');
            }
        }
    };
    while at < bytes.len() {
        let c = bytes[at];
        let next = bytes.get(at + 1).copied();
        if c == '/' && next == Some('/') {
            let end = (at..bytes.len())
                .find(|&i| bytes[i] == '\n')
                .unwrap_or(bytes.len());
            at = end;
        } else if c == '/' && next == Some('*') {
            let mut depth = 1;
            let mut i = at + 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == '/' && bytes.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == '*' && bytes.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            keep_newlines(at, i, &mut kept);
            at = i;
        } else if c == '"' {
            let mut i = at + 1;
            while i < bytes.len() && bytes[i] != '"' {
                i += if bytes[i] == '\\' { 2 } else { 1 };
            }
            keep_newlines(at, i, &mut kept);
            kept.push_str("\"\"");
            at = i + 1;
        } else if c == 'r' && matches!(next, Some('"' | '#')) && raw_start(&bytes, at).is_some() {
            let (quote, hashes) = raw_start(&bytes, at).unwrap_or((at, 0));
            let mut i = quote + 1;
            while i < bytes.len() {
                if bytes[i] == '"' && bytes[i + 1..].iter().take(hashes).all(|&b| b == '#') {
                    break;
                }
                i += 1;
            }
            keep_newlines(at, i, &mut kept);
            kept.push_str("\"\"");
            at = i + 1 + hashes;
        } else if c == '\'' && char_end(&bytes, at).is_some() {
            at = char_end(&bytes, at).unwrap_or(at) + 1;
            kept.push(' ');
        } else {
            kept.push(c);
            at += 1;
        }
    }
    kept
}

fn raw_start(bytes: &[char], at: usize) -> Option<(usize, usize)> {
    let mut i = at + 1;
    let mut hashes = 0;
    while bytes.get(i) == Some(&'#') {
        hashes += 1;
        i += 1;
    }
    (bytes.get(i) == Some(&'"')).then_some((i, hashes))
}

fn char_end(bytes: &[char], at: usize) -> Option<usize> {
    match (bytes.get(at + 1), bytes.get(at + 2), bytes.get(at + 3)) {
        (Some('\\'), Some(_), _) => (at + 3..at + 12).find(|&i| bytes.get(i) == Some(&'\'')),
        (Some(_), Some('\''), _) => Some(at + 2),
        _ => None,
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Words of `code` with their lines.
fn words(code: &str) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for (index, line) in code.lines().enumerate() {
        for word in line.split(|c| !is_word(c)).filter(|word| !word.is_empty()) {
            found.push((word.to_string(), index + 1));
        }
    }
    found
}

/// Everything banned in one source file.
#[must_use]
pub fn scan(text: &str) -> Vec<Hit> {
    let code = code_only(text);
    let list = words(&code);
    let mut hits = Vec::new();
    for (at, (word, line)) in list.iter().enumerate() {
        let next = list.get(at + 1).map(|(word, _)| word.as_str());
        let third = list.get(at + 2).map(|(word, _)| word.as_str());
        if let Some((_, why)) = BANNED.iter().find(|(banned, _)| banned == word) {
            hits.push(Hit {
                word: word.clone(),
                why,
                line: *line,
            });
        } else if word == "extern" && (next, third) != (Some("crate"), Some("alloc")) {
            hits.push(Hit {
                word: "extern".to_string(),
                why: "foreign code",
                line: *line,
            });
        } else if word == "static" && next == Some("mut") {
            hits.push(Hit {
                word: "static mut".to_string(),
                why: "global state",
                line: *line,
            });
        } else if word == "offline" && next.is_some() {
            hits.push(Hit {
                word: word.clone(),
                why: "the core knows nothing of the layer",
                line: *line,
            });
        }
    }
    hits
}

/// Addresses in a source file, comments included.
#[must_use]
pub fn addresses(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| ADDRESS_MARKS.iter().any(|mark| line.contains(mark)))
        .map(|(index, _)| index + 1)
        .collect()
}
