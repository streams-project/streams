//! Search by person: an index from the names of directors and top billed cast to the titles they are in, built once from the catalog
//! database at start. The catalog's full text index covers title, overview, genres and director only, so a search for an actor would
//! otherwise find nothing.
use crate::det::HashMap;
use rusqlite::{Connection, OpenFlags};

/// Titles below this popularity are left out of the index (kept small; a person is found through the titles people watch).
const MIN_POPULARITY: f64 = 1.5;
const CAST_PER_TITLE: usize = 6;

#[derive(Default)]
pub struct PeopleIndex {
    /// normalised name -> (internal item key, popularity), unsorted
    names: HashMap<String, Vec<(u32, f32)>>,
}

/// Lower case, letters and digits only, single spaces: "Joseph Gordon-Levitt" becomes "joseph gordon levitt".
pub fn normalise(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = true;
    for c in text.chars() {
        // Arabic short vowels and the stretching line carry no letter: drop them, do not split.
        if ('\u{064B}'..='\u{0652}').contains(&c) || c == '\u{0640}' || c == '\u{0670}' {
            continue;
        }
        if c.is_alphanumeric() {
            for lower in c.to_lowercase() {
                fold(lower, &mut out);
            }
            space = false;
        } else if !space {
            out.push(' ');
            space = true;
        }
    }
    out.trim_end().to_string()
}

/// One letter as search compares it: accents off, Arabic letter forms joined, every digit Western.
fn fold(c: char, out: &mut String) {
    let plain = match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ą' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
        'ñ' | 'ń' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ű' | 'ů' => 'u',
        'ý' | 'ÿ' => 'y',
        'ł' => 'l',
        'ś' | 'š' | 'ş' => 's',
        'ź' | 'ż' | 'ž' => 'z',
        'ğ' => 'g',
        'ř' => 'r',
        'đ' | 'ď' => 'd',
        'ť' => 't',
        'ß' => {
            out.push_str("ss");
            return;
        }
        'أ' | 'إ' | 'آ' | 'ٱ' => 'ا',
        'ة' => 'ه',
        'ى' | 'ئ' => 'ي',
        'ؤ' => 'و',
        '\u{0660}'..='\u{0669}' => char::from_u32(c as u32 - 0x0660 + u32::from(b'0')).unwrap_or(c),
        '\u{06F0}'..='\u{06F9}' => char::from_u32(c as u32 - 0x06F0 + u32::from(b'0')).unwrap_or(c),
        other => other,
    };
    out.push(plain);
}

impl PeopleIndex {
    /// Read the index. The file opens read only, so a missing file is never created.
    pub fn load(db_path: &std::path::Path) -> Self {
        let Ok(conn) = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        else {
            return Self::default();
        };
        let Ok(mut stmt) = conn.prepare("SELECT id, popularity, director, cast_members FROM catalog WHERE popularity >= ?1 AND (cast_members IS NOT NULL OR director IS NOT NULL)") else {
            return Self::default();
        };
        let mut names: HashMap<String, Vec<(u32, f32)>> = HashMap::default();
        let rows = stmt.query_map([MIN_POPULARITY], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, f64>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        });
        if let Ok(rows) = rows {
            for (id, pop, director, cast) in rows.filter_map(|r| r.ok()) {
                let people = director
                    .iter()
                    .flat_map(|d| d.split(','))
                    .chain(cast.iter().flat_map(|c| c.split(',').take(CAST_PER_TITLE)));
                let mut seen: Vec<String> = Vec::new();
                for p in people {
                    let n = normalise(p);
                    if n.chars().count() >= 5 && !seen.contains(&n) {
                        names
                            .entry(n.clone())
                            .or_default()
                            .push((id as u32, pop as f32));
                        seen.push(n);
                    }
                }
            }
        }
        Self { names }
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// The best known person whose name has `query` as a whole word (a surname or a first name typed alone: "nolan" gives Christopher Nolan, not every Nolan), with their titles,
    /// most popular first. The weight of a person is the popularity of their five best titles, so a director of several hits beats a name that appears once. None when the
    /// word is short or no person has three titles or more.
    pub fn person_by_word(&self, query: &str, limit: usize) -> Option<(String, f32, Vec<u32>)> {
        let q = normalise(query);
        if q.chars().count() < 4 || q.contains(' ') {
            return None;
        }
        let mut best: Option<(&String, f32)> = None;
        for (name, titles) in &self.names {
            if titles.len() < 3 || !name.split(' ').any(|w| w == q) {
                continue;
            }
            let mut pops: Vec<f32> = titles.iter().map(|t| t.1).collect();
            pops.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            let weight: f32 = pops.iter().take(5).sum();
            if best.is_none_or(|(_, w)| weight > w) {
                best = Some((name, weight));
            }
        }
        let (name, weight) = best?;
        let mut v: Vec<(u32, f32)> = self.names[name].clone();
        v.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        Some((
            name.clone(),
            weight,
            v.into_iter().take(limit).map(|(k, _)| k).collect(),
        ))
    }

    /// Titles of the people the query names, most popular first. A query matches a person when the person's name is the query or starts with it
    /// (a single word must be a whole name, so a first name alone does not pull in everyone). The first element tells whether the query was an exact name.
    pub fn titles_for(&self, query: &str, limit: usize) -> (bool, Vec<u32>) {
        let q = normalise(query);
        if q.chars().count() < 5 {
            return (false, Vec::new());
        }
        let exact = self.names.contains_key(&q);
        let mut best: HashMap<u32, f32> = HashMap::default();
        for (name, titles) in &self.names {
            if name == &q || (q.contains(' ') && name.starts_with(&q)) {
                for &(k, p) in titles {
                    let e = best.entry(k).or_insert(0.0);
                    *e = e.max(p);
                }
            }
        }
        let mut v: Vec<(u32, f32)> = best.into_iter().collect();
        v.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        (exact, v.into_iter().take(limit).map(|(k, _)| k).collect())
    }
}

/// True when `a` and `b` differ by at most `max` single letter edits (insert, delete, replace, or swap of two neighbours).
pub fn within_edits(a: &str, b: &str, max: usize) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.len().abs_diff(b.len()) > max {
        return false;
    }
    let mut prev2: Vec<usize> = vec![0; b.len() + 1];
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        let mut row_min = cur[0];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(prev2[j - 2] + 1);
            }
            cur[j] = v;
            row_min = row_min.min(v);
        }
        if row_min > max {
            return false;
        }
        prev2 = prev;
        prev = cur;
    }
    prev[b.len()] <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_normalised() {
        assert_eq!(normalise("Joseph Gordon-Levitt"), "joseph gordon levitt");
        // Accents fold, so a remote keyboard without them still finds the name.
        assert_eq!(normalise("  Zoë  Saldaña "), "zoe saldana");
    }

    #[test]
    fn arabic_letter_forms_and_digits_fold() {
        assert_eq!(normalise("الأوديسة"), normalise("الاوديسه"));
        assert_eq!(normalise("مُسَلْسَل"), "مسلسل");
        assert_eq!(normalise("عام ٢٠٢٤"), "عام 2024");
    }

    #[test]
    fn small_typos_are_within_reach() {
        assert!(within_edits("stranger thngs", "stranger things", 1));
        assert!(within_edits("inceptoin", "inception", 1));
        assert!(!within_edits("avatar", "avenger", 1));
        assert!(within_edits("the offce", "the office", 1));
        assert!(!within_edits("a", "abcd", 2));
    }
}
