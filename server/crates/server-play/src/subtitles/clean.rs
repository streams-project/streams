//! Subtitle files: decode, read the cues and check they are real. Most are SRT in UTF-8, some in
//! a language's older code page or in UTF-16, and some are not subtitles at all. A file is judged
//! on the server, so no device holds code page tables. The length check needs the player's
//! duration, so it stays with the core.

use encoding_rs::{Encoding, UTF_16BE, UTF_16LE};
use regex::Regex;
use std::sync::OnceLock;

/// One cue, in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub s: f64,
    pub e: f64,
    /// Lines joined by a new line. `<i>`, `<b>` and `<u>` kept, every other tag removed.
    pub t: String,
}

/// Fewer cues than this is not a subtitle file for a film or an episode.
pub const MIN_CUES: usize = 10;
/// The size a file may be. The fetch stops at this byte, so a larger file is never read past it;
/// the check below is the same rule for a file that reached here another way.
pub const MAX_BYTES: usize = 3_000_000;

/// The older code page a file in this language is usually written in.
fn legacy_page(lang: &str) -> &'static Encoding {
    let label = match lang.to_ascii_uppercase().as_str() {
        "AR" | "FA" | "UR" => "windows-1256",
        "RU" | "UK" | "BG" | "SR" | "MK" | "BE" => "windows-1251",
        "EL" => "windows-1253",
        "TR" => "windows-1254",
        "HE" => "windows-1255",
        "VI" => "windows-1258",
        "TH" => "windows-874",
        "PL" | "CS" | "SK" | "HU" | "RO" | "HR" | "SL" | "BS" | "SQ" => "windows-1250",
        "ET" | "LV" | "LT" => "windows-1257",
        "ZH" => "gb18030",
        "JA" => "shift_jis",
        "KO" => "euc-kr",
        _ => "windows-1252",
    };
    Encoding::for_label(label.as_bytes()).unwrap_or(encoding_rs::WINDOWS_1252)
}

/// Text of a file: a byte order mark, then UTF-16 with no mark, then strict UTF-8, then the
/// language's code page.
#[must_use]
pub fn decode(bytes: &[u8], lang: &str) -> String {
    if bytes.starts_with(&[0xff, 0xfe]) {
        return UTF_16LE.decode(bytes).0.into_owned();
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return UTF_16BE.decode(bytes).0.into_owned();
    }
    if bytes.len() > 64 {
        let zeros = |offset: usize| {
            bytes[..64]
                .iter()
                .skip(offset)
                .step_by(2)
                .filter(|byte| **byte == 0)
                .count()
        };
        if zeros(1) > 24 {
            return UTF_16LE.decode(bytes).0.into_owned();
        }
        if zeros(0) > 24 {
            return UTF_16BE.decode(bytes).0.into_owned();
        }
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => legacy_page(lang).decode(bytes).0.into_owned(),
    }
}

fn pattern(cell: &'static OnceLock<Regex>, text: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(text).expect("a fixed pattern"))
}

/// Seconds of `00:01:02,345` or `01:02.345`.
fn seconds(stamp: &str) -> Option<f64> {
    static CELL: OnceLock<Regex> = OnceLock::new();
    let found = pattern(&CELL, r"(?:(\d+):)?(\d{1,2}):(\d{2})[,.](\d{1,3})").captures(stamp)?;
    let number = |at: usize| {
        found
            .get(at)
            .map_or(Some(0.0), |part| part.as_str().parse::<f64>().ok())
    };
    let fraction = format!("{:0<3}", found.get(4)?.as_str())
        .parse::<f64>()
        .ok()?;
    Some(number(1)? * 3600.0 + number(2)? * 60.0 + number(3)? + fraction / 1000.0)
}

/// Style overrides out, only italic, bold and underline kept, a hard space read as a space.
fn clean(text: &str) -> String {
    static STYLE: OnceLock<Regex> = OnceLock::new();
    static TAG: OnceLock<Regex> = OnceLock::new();
    let text = pattern(&STYLE, r"\{\\[^}]*\}").replace_all(text, "");
    let text = pattern(&TAG, r"<[^>]*>").replace_all(&text, |found: &regex::Captures<'_>| {
        let tag = found[0].to_ascii_lowercase();
        if matches!(
            tag.as_str(),
            "<i>" | "</i>" | "<b>" | "</b>" | "<u>" | "</u>"
        ) {
            found[0].to_owned()
        } else {
            String::new()
        }
    });
    text.replace("&nbsp;", " ").trim().to_owned()
}

/// Read cues: the first line with an arrow is the timing, and only cues with real times and text
/// survive, sorted by start.
#[must_use]
pub fn parse_cues(text: &str) -> Vec<Cue> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.trim_start_matches('\u{feff}');
    static BLANK: OnceLock<Regex> = OnceLock::new();
    let mut cues = Vec::new();
    for block in pattern(&BLANK, r"\n{2,}").split(text) {
        let lines: Vec<&str> = block.split('\n').collect();
        let Some(at) = lines.iter().position(|line| line.contains("-->")) else {
            continue;
        };
        let Some((from, to)) = lines[at].split_once("-->") else {
            continue;
        };
        let (Some(s), Some(e)) = (seconds(from), seconds(to)) else {
            continue;
        };
        let t = clean(&lines[at + 1..].join("\n"));
        if e > s && !t.is_empty() {
            cues.push(Cue { s, e, t });
        }
    }
    cues.sort_by(|a, b| a.s.total_cmp(&b.s));
    cues
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Script {
    Latin,
    Arabic,
    Cyrillic,
    Greek,
    Hebrew,
    Han,
    Kana,
    Hangul,
    Thai,
    Devanagari,
    Other,
}

fn script_of(ch: char) -> Option<Script> {
    let code = ch as u32;
    let within = |low: u32, high: u32| (low..=high).contains(&code);
    Some(
        if within(0x41, 0x5a) || within(0x61, 0x7a) || within(0xc0, 0x24f) || within(0x1e00, 0x1eff)
        {
            Script::Latin
        } else if within(0x600, 0x6ff)
            || within(0x750, 0x77f)
            || within(0xfb50, 0xfdff)
            || within(0xfe70, 0xfeff)
        {
            Script::Arabic
        } else if within(0x400, 0x52f) {
            Script::Cyrillic
        } else if within(0x370, 0x3ff) {
            Script::Greek
        } else if within(0x590, 0x5ff) {
            Script::Hebrew
        } else if within(0x4e00, 0x9fff) || within(0x3400, 0x4dbf) {
            Script::Han
        } else if within(0x3040, 0x30ff) {
            Script::Kana
        } else if within(0xac00, 0xd7af) || within(0x1100, 0x11ff) {
            Script::Hangul
        } else if within(0xe00, 0xe7f) {
            Script::Thai
        } else if within(0x900, 0x97f) {
            Script::Devanagari
        } else if code > 0x2ff && ch.is_alphabetic() {
            Script::Other
        } else {
            return None;
        },
    )
}

fn expected_scripts(lang: &str) -> &'static [Script] {
    use Script::{
        Arabic, Cyrillic, Devanagari, Greek, Han, Hangul, Hebrew, Kana, Latin, Other, Thai,
    };
    match lang {
        "ar" | "fa" | "ur" | "ps" => &[Arabic],
        "ku" => &[Arabic, Latin],
        "ru" | "uk" | "bg" | "mk" | "be" | "kk" | "mn" => &[Cyrillic],
        "sr" => &[Cyrillic, Latin],
        "el" => &[Greek],
        "he" | "yi" => &[Hebrew],
        "zh" => &[Han],
        "ja" => &[Han, Kana],
        "ko" => &[Hangul, Han],
        "th" => &[Thai],
        "hi" | "mr" | "ne" => &[Devanagari],
        "bn" | "ta" | "te" | "ml" | "kn" | "gu" | "pa" | "si" | "ka" | "hy" | "km" | "my"
        | "lo" | "am" => &[Other],
        _ => &[Latin],
    }
}

/// Common words of the languages written in Latin letters.
const WORDS: [(&str, &[&str]); 12] = [
    (
        "en",
        &[
            "the", "you", "and", "to", "is", "that", "it", "what", "this", "of", "are", "have",
            "don't", "i'm", "we", "with", "for", "not",
        ],
    ),
    (
        "es",
        &[
            "que", "de", "no", "la", "el", "es", "en", "lo", "los", "por", "qué", "una", "con",
            "para", "está", "pero", "esto",
        ],
    ),
    (
        "fr",
        &[
            "je", "de", "est", "pas", "le", "la", "vous", "que", "tu", "les", "et", "un", "il",
            "ce", "une", "qui", "nous", "c'est",
        ],
    ),
    (
        "de",
        &[
            "ich", "die", "der", "und", "nicht", "du", "das", "ist", "sie", "es", "wir", "zu",
            "ein", "was", "mit", "mir", "auf",
        ],
    ),
    (
        "it",
        &[
            "non", "che", "di", "il", "è", "la", "un", "per", "mi", "ti", "sono", "cosa", "questo",
            "lo", "ho", "ci", "come",
        ],
    ),
    (
        "pt",
        &[
            "que", "não", "de", "o", "é", "você", "a", "eu", "um", "isso", "para", "se", "com",
            "uma", "está", "do", "mas",
        ],
    ),
    (
        "nl",
        &[
            "ik", "je", "het", "de", "een", "niet", "dat", "is", "van", "en", "wat", "we", "zijn",
            "hij", "maar", "heb",
        ],
    ),
    (
        "pl",
        &[
            "nie", "to", "się", "jest", "że", "co", "na", "jak", "tak", "mi", "ja", "ale", "mnie",
            "tu", "już", "czy",
        ],
    ),
    (
        "tr",
        &[
            "bir", "bu", "ve", "ne", "ben", "sen", "mi", "değil", "için", "çok", "var", "da", "de",
            "bana", "o", "şey",
        ],
    ),
    (
        "sv",
        &[
            "jag", "det", "du", "är", "inte", "att", "en", "och", "har", "vi", "på", "som", "med",
            "han", "vad",
        ],
    ),
    (
        "id",
        &[
            "yang", "tidak", "aku", "kau", "ini", "itu", "dan", "apa", "di", "kita", "saya", "ada",
            "dia", "akan",
        ],
    ),
    (
        "ro",
        &[
            "nu", "să", "este", "de", "și", "ce", "în", "am", "pe", "mai", "sunt", "eu", "asta",
            "cu",
        ],
    ),
];

fn share_of(sample: &str, letters: usize, set: &str) -> f64 {
    let found = sample.chars().filter(|ch| set.contains(*ch)).count();
    found as f64 / letters as f64
}

/// True when the text fits the language it was listed as.
#[must_use]
pub fn matches_language(cues: &[Cue], tag: &str) -> bool {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let lang = tag.to_lowercase();
    let lang = lang.split('-').next().unwrap_or("");
    if lang.is_empty() {
        return true;
    }
    let lines: Vec<String> = cues
        .iter()
        .take(400)
        .map(|cue| {
            pattern(&TAG, r"<[^>]+>")
                .replace_all(&cue.t, "")
                .into_owned()
        })
        .collect();
    let sample = lines.join(" ");
    let found: Vec<Script> = sample.chars().filter_map(script_of).collect();
    let letters = found.len();
    if letters < 50 {
        return true;
    }
    let expected = expected_scripts(lang);
    let wanted = found
        .iter()
        .filter(|script| expected.contains(script))
        .count();
    if (wanted as f64) / (letters as f64) < 0.6 {
        return false;
    }
    let persian = share_of(&sample, letters, "پچژگ");
    let ukrainian = share_of(&sample, letters, "іїєґІЇЄҐ");
    let russian = share_of(&sample, letters, "ыэъЫЭЪ");
    let refused = (lang == "ar" && persian > 0.01)
        || (lang == "fa" && persian < 0.002)
        || (lang == "uk" && (ukrainian < 0.005 || russian > 0.005))
        || (lang == "ru" && ukrainian > 0.005);
    if refused {
        return false;
    }
    if expected[0] == Script::Latin && WORDS.iter().any(|(name, _)| *name == lang) {
        let lower = sample.to_lowercase();
        let words: Vec<&str> = lower
            .split(|c: char| !c.is_alphabetic() && c != '\'')
            .collect();
        let score = |list: &[&str]| words.iter().filter(|word| list.contains(word)).count();
        let own = WORDS
            .iter()
            .find(|(name, _)| *name == lang)
            .map_or(0, |(_, list)| score(list));
        let best = WORDS
            .iter()
            .filter(|(name, _)| *name != lang)
            .map(|(_, list)| score(list))
            .max()
            .unwrap_or(0);
        if best > 40 && own * 2 < best {
            return false;
        }
    }
    true
}

/// True for a line that advertises a site or a team and says nothing in the film.
#[must_use]
pub fn is_advert(text: &str) -> bool {
    static CELL: OnceLock<Regex> = OnceLock::new();
    pattern(
        &CELL,
        r"(?i)(subscene|yts\.|yify|addic7ed|podnapisi|subtitlecat|advertise your product|subtitles? (by|ripped|synced|downloaded)|sync(ed)? (and|&) correct(ed|ions)|www\.|https?://|\.(com|net|org|tv|me|io)\b)",
    )
    .is_match(text)
}

/// Why a file was refused. The caller moves to the next file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    TooLarge,
    TooFewCues,
    WrongLanguage,
}

/// Read one file and judge it: size, cue count after adverts, language. The core checks its length
/// against the driver's duration, because this server does not know the stream.
///
/// # Errors
/// The reason this file is not usable.
pub fn read_file(bytes: &[u8], lang: &str) -> Result<Vec<Cue>, Refusal> {
    if bytes.len() > MAX_BYTES {
        return Err(Refusal::TooLarge);
    }
    let base = lang.split('-').next().unwrap_or("");
    let mut cues = parse_cues(&decode(bytes, base));
    cues.retain(|cue| !is_advert(&cue.t));
    if cues.len() < MIN_CUES {
        return Err(Refusal::TooFewCues);
    }
    if !matches_language(&cues, lang) {
        return Err(Refusal::WrongLanguage);
    }
    Ok(cues)
}

/// WebVTT shifted by a delay, for players that load a side file.
#[must_use]
pub fn to_vtt(cues: &[Cue], delay_ms: i64) -> String {
    let stamp = |seconds: f64| {
        let total = ((seconds * 1000.0).round() as i64 + delay_ms).max(0);
        let (hours, rest) = (total / 3_600_000, total % 3_600_000);
        format!(
            "{hours:02}:{:02}:{:02}.{:03}",
            rest / 60_000,
            rest / 1000 % 60,
            rest % 1000
        )
    };
    let mut out = String::from("WEBVTT\n\n");
    for cue in cues {
        out.push_str(&format!(
            "{} --> {}\n{}\n\n",
            stamp(cue.s),
            stamp(cue.e),
            cue.t
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srt(lines: &[&str]) -> String {
        let mut out = String::new();
        for (n, line) in lines.iter().enumerate() {
            let n = n as u32;
            out.push_str(&format!(
                "{}\n00:00:{:02},000 --> 00:00:{:02},500\n{line}\n\n",
                n + 1,
                n % 60,
                n % 60
            ));
        }
        out
    }

    #[test]
    fn c44_marks_utf16_and_code_pages_all_decode() {
        assert_eq!(decode(&[0xff, 0xfe, b'h', 0, b'i', 0], ""), "hi");
        assert_eq!(decode(&[0xfe, 0xff, 0, b'h', 0, b'i'], ""), "hi");
        assert_eq!(decode("héllo".as_bytes(), ""), "héllo");
        assert_eq!(
            decode(&[0xcf, 0xf0, 0xe8, 0xe2, 0xe5, 0xf2], "RU"),
            "Привет"
        );
        assert_eq!(decode(&[0xe9], ""), "é", "the default page");
        let plain: Vec<u8> = "subtitle line "
            .repeat(10)
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert!(
            decode(&plain, "").starts_with("subtitle line"),
            "UTF-16 with no mark"
        );
    }

    #[test]
    fn c45_cues_read_with_either_stamp_and_lose_foreign_tags() {
        let cues = parse_cues(
            "\u{feff}1\r\n00:01:02,345 --> 00:01:04.5\r\n{\\an8}<font color=\"red\">Hi</font> <i>there</i>&nbsp;now\r\n\r\n2\r\n01:02.000 --> 01:03.000\r\nShort\r\n\r\n3\r\n00:00:09,000 --> 00:00:08,000\r\nBackwards\r\n\r\n4\r\nno timing here",
        );
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[1].t, "Hi <i>there</i> now");
        assert!((cues[1].s - 62.345).abs() < 1e-9);
        assert!((cues[1].e - 64.5).abs() < 1e-9);
        assert!((cues[0].s - 62.0).abs() < 1e-9);
    }

    #[test]
    fn c46_a_file_in_the_wrong_script_is_refused() {
        let cues = |text: &str| {
            vec![Cue {
                s: 0.0,
                e: 1.0,
                t: text.repeat(30),
            }]
        };
        assert!(matches_language(&cues("hello there "), "en"));
        assert!(!matches_language(&cues("مرحبا كيف حالك "), "en"));
        assert!(matches_language(&cues("مرحبا كيف حالك "), "ar"));
        assert!(matches_language(&[], "en"), "under 50 letters passes");
        assert!(matches_language(&cues("x"), ""), "no language passes");
    }

    #[test]
    fn c46_two_languages_of_one_script_are_told_apart() {
        let cues = |text: &str| {
            vec![Cue {
                s: 0.0,
                e: 1.0,
                t: text.repeat(40),
            }]
        };
        assert!(!matches_language(&cues("Привіт, як справи їжак є "), "ru"));
        assert!(matches_language(&cues("Привіт, як справи їжак є "), "uk"));
        let spanish = "que de no la el es en lo los por una con para pero ";
        assert!(
            !matches_language(&cues(spanish), "en"),
            "a Latin file in another language"
        );
        assert!(matches_language(&cues(spanish), "es"));
    }

    #[test]
    fn c47_advert_lines_are_found() {
        for line in [
            "Subtitles by Someone",
            "www.host.example",
            "Visit site.com today",
            "Synced & corrected",
        ] {
            assert!(is_advert(line), "{line}");
        }
        assert!(!is_advert("Stay with me, come on."));
    }

    #[test]
    fn c43_a_file_needs_ten_real_cues_in_its_language_and_a_size_limit() {
        let good: Vec<&str> = vec!["Stay with me now"; 12];
        assert_eq!(read_file(srt(&good).as_bytes(), "en").unwrap().len(), 12);
        let thin: Vec<&str> = vec!["Stay with me now"; 9];
        assert_eq!(
            read_file(srt(&thin).as_bytes(), "en"),
            Err(Refusal::TooFewCues)
        );
        let adverts: Vec<&str> = vec!["www.host.example"; 12];
        assert_eq!(
            read_file(srt(&adverts).as_bytes(), "en"),
            Err(Refusal::TooFewCues)
        );
        let note = "Find this film at a download site";
        assert_eq!(read_file(note.as_bytes(), "en"), Err(Refusal::TooFewCues));
        assert_eq!(
            read_file(&vec![b'x'; MAX_BYTES + 1], "en"),
            Err(Refusal::TooLarge)
        );
        let arabic: Vec<&str> = vec!["مرحبا كيف حالك يا صديقي العزيز"; 12];
        assert_eq!(
            read_file(srt(&arabic).as_bytes(), "en"),
            Err(Refusal::WrongLanguage)
        );
    }

    #[test]
    fn s115_webvtt_carries_the_delay_and_never_goes_below_zero() {
        let cues = vec![Cue {
            s: 1.5,
            e: 2.0,
            t: String::from("Hi"),
        }];
        assert_eq!(
            to_vtt(&cues, 250),
            "WEBVTT\n\n00:00:01.750 --> 00:00:02.250\nHi\n\n"
        );
        assert!(to_vtt(&cues, -5000).contains("00:00:00.000 --> 00:00:00.000"));
    }
}
