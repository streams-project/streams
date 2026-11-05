//! A language as the sources write it, made one BCP 47 tag: `en`, `pt-BR`, `es-419`, `zh-Hans`.

/// A parsed language label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lang {
    /// BCP 47 tag: a language, and a region or script only where it is a different version (pt-BR, es-419, fr-CA, zh-Hans, zh-Hant)
    pub tag: String,
    /// subtitles for deaf and hard of hearing viewers (SDH, CC, HI)
    pub hi: bool,
    /// forced: only the lines in another language than the audio
    pub forced: bool,
}

/// The tag of a label, or `None` when it names no language ("und", "unknown", "other", an empty string).
pub fn parse(raw: &str) -> Option<Lang> {
    let lower = raw.trim().to_lowercase();
    if lower.is_empty() {
        return None;
    }
    // "HI" alone is a source's code for Hindi, not the hearing impaired flag it means after a language ("English HI")
    if lower == "hi" || lower == "hin" {
        return Some(Lang {
            tag: "hi".into(),
            hi: false,
            forced: false,
        });
    }
    let words = |w: &str| lower.split(|c: char| !c.is_alphanumeric()).any(|x| x == w);
    let hi = words("sdh") || words("cc") || words("hi") || lower.contains("hearing");
    let forced = words("forced");
    // the label without the flags, brackets and punctuation
    let mut base = String::new();
    for word in lower.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_' || c == '/')) {
        if word.is_empty()
            || matches!(
                word,
                "sdh"
                    | "cc"
                    | "hi"
                    | "forced"
                    | "full"
                    | "subtitles"
                    | "subs"
                    | "sub"
                    | "track"
                    | "audio"
                    | "default"
            )
        {
            continue;
        }
        if !base.is_empty() {
            base.push(' ');
        }
        base.push_str(word);
    }
    let base = base.replace('_', "-");
    let tag = special(&base)
        .or_else(|| from_code(&base))
        .or_else(|| from_name(&base))?;
    Some(Lang {
        tag: tag.to_string(),
        hi,
        forced,
    })
}

/// Versions of a language that are worth their own entry, written in the many ways sources write them.
fn special(s: &str) -> Option<&'static str> {
    let has = |w: &str| s.contains(w);
    Some(match s {
        "pt-br" | "br-pt" | "pob" | "ptbr" => "pt-BR",
        "pt-pt" => "pt-PT",
        "es-419" | "es-la" | "es-lat" | "es-mx" => "es-419",
        "fr-ca" => "fr-CA",
        "zh-cn" | "zh-hans" | "zh-sg" | "chs" | "zh-bg" | "zhs" => "zh-Hans",
        "zh-tw" | "zh-hk" | "zh-hant" | "cht" | "zht" | "big5" | "big-5" => "zh-Hant",
        "sr-latn" => "sr-Latn",
        "sr-cyrl" => "sr-Cyrl",
        _ if has("brazil") || has("brasil") => "pt-BR",
        _ if (has("portug") && has("portugal")) || has("european portuguese") => "pt-PT",
        _ if has("latin america") || has("latinoam") || has("latino") || has("mexic") => "es-419",
        _ if has("canad") && has("fren") => "fr-CA",
        _ if has("big 5")
            || has("big5")
            || has("tradition")
            || has("繁")
            || has("taiwan")
            || has("hong kong")
            || has("cantonese") =>
        {
            "zh-Hant"
        }
        _ if has("bg code") || has("simplif") || has("简") || has("mandarin") => "zh-Hans",
        _ => return None,
    })
}

/// Two and three letter codes (ISO 639-1, 639-2/B, 639-2/T), with a region dropped unless `special` kept it.
fn from_code(s: &str) -> Option<&'static str> {
    let code = s.split(['-', ' ', '/']).next().unwrap_or(s);
    if !(2..=3).contains(&code.len()) || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(match code {
        "en" | "eng" => "en",
        "es" | "spa" => "es",
        "fr" | "fre" | "fra" => "fr",
        "de" | "ger" | "deu" => "de",
        "it" | "ita" => "it",
        "pt" | "por" => "pt",
        "ru" | "rus" => "ru",
        "ja" | "jpn" | "jp" => "ja",
        "ko" | "kor" | "kr" => "ko",
        "zh" | "chi" | "zho" | "cn" => "zh",
        "ar" | "ara" => "ar",
        "tr" | "tur" => "tr",
        "nl" | "dut" | "nld" => "nl",
        "pl" | "pol" => "pl",
        "sv" | "swe" => "sv",
        "da" | "dan" => "da",
        "fi" | "fin" => "fi",
        "no" | "nor" | "nb" | "nob" | "nn" | "nno" => "no",
        "hu" | "hun" => "hu",
        "el" | "ell" | "gre" | "gr" => "el",
        "he" | "heb" | "iw" => "he",
        "cs" | "ces" | "cze" | "cz" => "cs",
        "ro" | "ron" | "rum" => "ro",
        "th" | "tha" => "th",
        "vi" | "vie" => "vi",
        "id" | "ind" | "in" => "id",
        "uk" | "ukr" | "ua" => "uk",
        "hr" | "hrv" => "hr",
        "sk" | "slk" | "slo" => "sk",
        "bg" | "bul" => "bg",
        "sr" | "srp" | "scc" => "sr",
        "hi" | "hin" => "hi",
        "bn" | "ben" => "bn",
        "fa" | "fas" | "per" => "fa",
        "ms" | "msa" | "may" => "ms",
        "ca" | "cat" => "ca",
        "lt" | "lit" => "lt",
        "lv" | "lav" => "lv",
        "et" | "est" => "et",
        "sl" | "slv" => "sl",
        "ta" | "tam" => "ta",
        "te" | "tel" => "te",
        "ml" | "mal" => "ml",
        "kn" | "kan" => "kn",
        "mr" | "mar" => "mr",
        "gu" | "guj" => "gu",
        "pa" | "pan" => "pa",
        "ur" | "urd" => "ur",
        "ne" | "nep" => "ne",
        "si" | "sin" => "si",
        "ka" | "kat" | "geo" => "ka",
        "hy" | "hye" | "arm" => "hy",
        "sq" | "sqi" | "alb" => "sq",
        "mk" | "mkd" | "mac" => "mk",
        "bs" | "bos" => "bs",
        "is" | "isl" | "ice" => "is",
        "af" | "afr" => "af",
        "sw" | "swa" => "sw",
        "eu" | "eus" | "baq" => "eu",
        "gl" | "glg" => "gl",
        "az" | "aze" => "az",
        "kk" | "kaz" => "kk",
        "uz" | "uzb" => "uz",
        "be" | "bel" => "be",
        "tl" | "tgl" | "fil" => "fil",
        "km" | "khm" => "km",
        "my" | "mya" | "bur" => "my",
        "lo" | "lao" => "lo",
        "am" | "amh" => "am",
        "so" | "som" => "so",
        "yo" | "yor" => "yo",
        "zu" | "zul" => "zu",
        "xh" | "xho" => "xh",
        "ha" | "hau" => "ha",
        "ig" | "ibo" => "ig",
        "mn" | "mon" => "mn",
        "ps" | "pus" => "ps",
        "ku" | "kur" => "ku",
        "cy" | "cym" | "wel" => "cy",
        "ga" | "gle" => "ga",
        "mt" | "mlt" => "mt",
        "lb" | "ltz" => "lb",
        "la" | "lat" => "la",
        "eo" | "epo" => "eo",
        "jv" | "jav" => "jv",
        "su" | "sun" => "su",
        "tg" | "tgk" => "tg",
        "ky" | "kir" => "ky",
        "tk" | "tuk" => "tk",
        "yi" | "yid" => "yi",
        "mi" | "mri" | "mao" => "mi",
        "sm" | "smo" => "sm",
        "haw" => "haw",
        "yue" => "zh-Hant",
        _ => return None,
    })
}

/// English and native names, with the spellings sources use ("Farsi/Persian", "Ukranian", "Español").
fn from_name(s: &str) -> Option<&'static str> {
    const NAMES: &[(&str, &str)] = &[
        ("english", "en"),
        ("spanish", "es"),
        ("español", "es"),
        ("espanol", "es"),
        ("castellano", "es"),
        ("french", "fr"),
        ("français", "fr"),
        ("francais", "fr"),
        ("german", "de"),
        ("deutsch", "de"),
        ("italian", "it"),
        ("italiano", "it"),
        ("portuguese", "pt"),
        ("português", "pt"),
        ("portugues", "pt"),
        ("russian", "ru"),
        ("русский", "ru"),
        ("japanese", "ja"),
        ("日本語", "ja"),
        ("korean", "ko"),
        ("한국어", "ko"),
        ("chinese", "zh"),
        ("中文", "zh"),
        ("arabic", "ar"),
        ("العربية", "ar"),
        ("عربي", "ar"),
        ("turkish", "tr"),
        ("türkçe", "tr"),
        ("turkce", "tr"),
        ("dutch", "nl"),
        ("nederlands", "nl"),
        ("flemish", "nl"),
        ("polish", "pl"),
        ("polski", "pl"),
        ("swedish", "sv"),
        ("svenska", "sv"),
        ("danish", "da"),
        ("dansk", "da"),
        ("finnish", "fi"),
        ("suomi", "fi"),
        ("norwegian", "no"),
        ("norsk", "no"),
        ("hungarian", "hu"),
        ("magyar", "hu"),
        ("greek", "el"),
        ("ελληνικά", "el"),
        ("hebrew", "he"),
        ("עברית", "he"),
        ("czech", "cs"),
        ("čeština", "cs"),
        ("romanian", "ro"),
        ("română", "ro"),
        ("thai", "th"),
        ("ไทย", "th"),
        ("vietnamese", "vi"),
        ("tiếng việt", "vi"),
        ("indonesian", "id"),
        ("bahasa indonesia", "id"),
        ("ukrainian", "uk"),
        ("ukranian", "uk"),
        ("українська", "uk"),
        ("croatian", "hr"),
        ("hrvatski", "hr"),
        ("slovak", "sk"),
        ("bulgarian", "bg"),
        ("български", "bg"),
        ("serbian", "sr"),
        ("srpski", "sr"),
        ("српски", "sr"),
        ("hindi", "hi"),
        ("हिन्दी", "hi"),
        ("bengali", "bn"),
        ("bangla", "bn"),
        ("farsi", "fa"),
        ("persian", "fa"),
        ("فارسی", "fa"),
        ("malay", "ms"),
        ("catalan", "ca"),
        ("lithuanian", "lt"),
        ("latvian", "lv"),
        ("estonian", "et"),
        ("slovenian", "sl"),
        ("slovene", "sl"),
        ("tamil", "ta"),
        ("telugu", "te"),
        ("malayalam", "ml"),
        ("kannada", "kn"),
        ("marathi", "mr"),
        ("gujarati", "gu"),
        ("punjabi", "pa"),
        ("urdu", "ur"),
        ("nepali", "ne"),
        ("sinhala", "si"),
        ("sinhalese", "si"),
        ("georgian", "ka"),
        ("armenian", "hy"),
        ("albanian", "sq"),
        ("macedonian", "mk"),
        ("bosnian", "bs"),
        ("icelandic", "is"),
        ("afrikaans", "af"),
        ("swahili", "sw"),
        ("basque", "eu"),
        ("galician", "gl"),
        ("azerbaijani", "az"),
        ("kazakh", "kk"),
        ("uzbek", "uz"),
        ("belarusian", "be"),
        ("tagalog", "fil"),
        ("filipino", "fil"),
        ("khmer", "km"),
        ("cambodian", "km"),
        ("burmese", "my"),
        ("lao", "lo"),
        ("amharic", "am"),
        ("somali", "so"),
        ("kurdish", "ku"),
        ("welsh", "cy"),
        ("irish", "ga"),
        ("maltese", "mt"),
        ("mongolian", "mn"),
        ("pashto", "ps"),
        ("yiddish", "yi"),
        ("latin", "la"),
        ("esperanto", "eo"),
        ("javanese", "jv"),
    ];
    // a name anywhere in the label ("farsi/persian", "english (united states)"); longest first so "bahasa indonesia" wins over shorter names
    let mut best: Option<(&str, &str)> = None;
    for (name, tag) in NAMES {
        let hit = s == *name
            || s.split([' ', '/', '-']).any(|w| w == *name)
            || (name.contains(' ') && s.contains(name));
        if hit && best.is_none_or(|(n, _)| name.len() > n.len()) {
            best = Some((name, tag));
        }
    }
    best.map(|(_, t)| t)
}

/// The language part of a tag: `pt` for `pt-BR`.
pub fn base(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// The code a subtitle source searches with for a tag (EN, BR_PT, ZH ...).
pub fn source_code(tag: &str) -> String {
    match tag {
        "pt-BR" => "BR_PT".into(),
        "zh-Hans" | "zh-Hant" => "ZH".into(),
        "es-419" => "ES".into(),
        "fr-CA" => "FR".into(),
        t => base(t).to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(s: &str) -> Option<String> {
        parse(s).map(|l| l.tag)
    }

    #[test]
    fn c41_source_codes_and_names() {
        assert_eq!(tag("EN").as_deref(), Some("en"));
        assert_eq!(tag("BR_PT").as_deref(), Some("pt-BR"));
        assert_eq!(tag("Br_pt").as_deref(), Some("pt-BR"));
        assert_eq!(tag("ZH").as_deref(), Some("zh"));
        assert_eq!(tag("Chinese BG code").as_deref(), Some("zh-Hans"));
        assert_eq!(tag("chinese bg code").as_deref(), Some("zh-Hans"));
        assert_eq!(tag("Big 5 code").as_deref(), Some("zh-Hant"));
        assert_eq!(tag("Farsi/Persian").as_deref(), Some("fa"));
        assert_eq!(tag("farsi_persian").as_deref(), Some("fa"));
        assert_eq!(tag("ukranian").as_deref(), Some("uk"));
        assert_eq!(tag("HI").as_deref(), Some("hi"));
        assert_eq!(tag("Hindi").as_deref(), Some("hi"));
        assert!(!parse("HI").unwrap().hi);
    }

    #[test]
    fn stream_codes_and_labels() {
        assert_eq!(tag("eng").as_deref(), Some("en"));
        assert_eq!(tag("pob").as_deref(), Some("pt-BR"));
        assert_eq!(tag("fre").as_deref(), Some("fr"));
        assert_eq!(tag("ger").as_deref(), Some("de"));
        assert_eq!(tag("en-US").as_deref(), Some("en"));
        assert_eq!(tag("pt-BR").as_deref(), Some("pt-BR"));
        assert_eq!(tag("es-419").as_deref(), Some("es-419"));
        assert_eq!(tag("Spanish (Latin America)").as_deref(), Some("es-419"));
        assert_eq!(tag("Portuguese (Brazil)").as_deref(), Some("pt-BR"));
        assert_eq!(tag("Español").as_deref(), Some("es"));
        assert_eq!(tag("Bahasa Indonesia").as_deref(), Some("id"));
        assert_eq!(tag("Italian").as_deref(), Some("it"));
    }

    #[test]
    fn flags_and_nothing() {
        let l = parse("English SDH").unwrap();
        assert_eq!((l.tag.as_str(), l.hi, l.forced), ("en", true, false));
        let l = parse("English [CC]").unwrap();
        assert!(l.hi);
        let l = parse("Spanish - Forced").unwrap();
        assert_eq!((l.tag.as_str(), l.forced), ("es", true));
        assert_eq!(tag(""), None);
        assert_eq!(tag("und"), None);
        assert_eq!(tag("Unknown"), None);
        assert_eq!(tag("Track 1"), None);
    }

    #[test]
    fn c107_source_search_codes() {
        assert_eq!(source_code("pt-BR"), "BR_PT");
        assert_eq!(source_code("en"), "EN");
        assert_eq!(source_code("zh-Hant"), "ZH");
    }
}
