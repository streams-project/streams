//! Genre pages: turn what a client sends for `genre` into a rule over catalogue items.

use server_catalog::catalog::CatalogItem;
use server_catalog::events::EventStore;

/// What a rule reads besides the title.
#[derive(Clone, Copy, Debug)]
pub struct SpecFacts<'a> {
    pub events: &'a EventStore,
    /// True when judged page memberships were loaded. Those pages then use the judged list alone.
    pub judged: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GenreSpec {
    /// Matches everything (a picker that is not a filter, for example Moods)
    Any,
    /// A collection the data cannot express (about 30 titles for Astrology)
    Unsupported,
    /// Titles that won or were nominated for an award family (`emmy`, `oscar`, `bafta`, `globe`), from the awards file
    Awards(&'static str),
    /// Any of these genre names, movie and series naming
    Names(&'static [&'static str]),
    Thriller,
    Anime,
    BlackStories,
    BookAdaptations,
    British,
    Classics,
    Cult,
    European,
    Halloween,
    Horror,
    Independent,
    International,
    Lgbtq,
    Musical,
    Reality,
    Romance,
    ScienceNature,
    Shorts,
    Sports,
    StandUp,
    ChatShows,
    Teen,
    /// The page of a calendar event , offered in its window; a title belongs by the event's keywords
    Event(String),
    /// Hollywood films (US, widely rated) and US series
    Us,
    /// A page of the Kids menus (genre_pages::KIDS_*): the rule is read from `kids_rule` by the page's lower case name
    Kids(&'static str),
    /// Not in the table: exact genre name, case insensitive
    Name(String),
}

/// The rules of the Kids pages that have no genre of their own: (page name, any of these keywords, any of these genres). A children's
/// profile's age limit keeps every page to children's titles; the rule says what the page is about.
const KIDS_RULES: &[(&str, &[&str], &[&str])] = &[
    (
        "animal time",
        &[
            "animal",
            "animals",
            "talking animal",
            "anthropomorphic animal",
            "dog",
            "cat",
            "horse",
            "pets",
            "wildlife",
            "zoo",
            "farm",
        ],
        &[],
    ),
    (
        "animals",
        &[
            "animal",
            "animals",
            "talking animal",
            "anthropomorphic animal",
            "dog",
            "cat",
            "horse",
            "pets",
            "wildlife",
            "zoo",
            "farm",
        ],
        &[],
    ),
    (
        "cars, trucks & trains",
        &[
            "car",
            "cars",
            "truck",
            "trucks",
            "train",
            "trains",
            "racing",
            "race car",
            "monster truck",
            "vehicle",
            "vehicles",
            "construction vehicle",
        ],
        &[],
    ),
    ("dinosaurs", &["dinosaur", "dinosaurs", "prehistoric"], &[]),
    (
        "feel-good",
        &[
            "feelgood",
            "feel-good",
            "heartwarming",
            "cheerful",
            "lighthearted",
        ],
        &[],
    ),
    (
        "for little ones",
        &[
            "preschool",
            "pre-school",
            "toddler",
            "nursery",
            "educational",
            "learning",
            "alphabet",
            "counting",
        ],
        &[],
    ),
    (
        "friends",
        &[
            "friendship",
            "friends",
            "best friends",
            "buddies",
            "friendship group",
        ],
        &[],
    ),
    ("funny", &["slapstick comedy", "silly"], &["Comedy"]),
    (
        "girls take the lead",
        &[
            "female protagonist",
            "girl",
            "girls",
            "heroine",
            "strong female lead",
            "girl power",
            "princess",
            "sisters",
        ],
        &[],
    ),
    (
        "science & arts",
        &[
            "science",
            "educational",
            "art",
            "arts",
            "crafts",
            "mathematics",
            "stem",
            "experiment",
            "inventions",
            "music education",
        ],
        &[],
    ),
    (
        "singing & dancing",
        &[
            "musical",
            "singing",
            "song",
            "songs",
            "dance",
            "dancing",
            "sing-along",
        ],
        &["Music"],
    ),
    (
        "spooky stuff",
        &[
            "halloween",
            "ghost",
            "ghosts",
            "monster",
            "monsters",
            "witch",
            "spooky",
            "haunted house",
            "haunting",
            "vampire",
        ],
        &[],
    ),
    (
        "cool special effects",
        &[
            "superhero",
            "robot",
            "giant robot",
            "giant monster",
            "visual effects",
            "space",
            "spaceship",
            "alien",
            "superpower",
        ],
        &["Science Fiction", "Sci-Fi & Fantasy"],
    ),
    (
        "princess tales",
        &["princess", "prince", "fairy tale", "fairytale", "royalty"],
        &[],
    ),
];

fn kids_rule(
    name: &str,
) -> Option<&'static (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
)> {
    KIDS_RULES.iter().find(|(n, _, _)| *n == name)
}

const EUROPE: &[&str] = &[
    "FR", "DE", "IT", "ES", "SE", "DK", "NO", "FI", "PL", "NL", "BE", "AT", "CH", "IE", "PT", "GR",
    "CZ", "HU", "RO", "BG", "HR", "RS", "SK", "SI", "LT", "LV", "EE", "IS", "LU", "UA", "AL", "BA",
    "MK", "ME", "CY", "MT",
];

fn has_genre(it: &CatalogItem, names: &[&str]) -> bool {
    it.genres
        .iter()
        .any(|g| names.iter().any(|n| g.eq_ignore_ascii_case(n)))
}
fn kw_is(it: &CatalogItem, words: &[&str]) -> bool {
    it.keywords.iter().any(|k| words.iter().any(|w| k == w))
}
/// Membership judged for this page category (`cat:<slug>`); None when the judged file was not loaded, so the page falls back to its keyword rule.
fn judged(it: &CatalogItem, slug: &str, facts: &SpecFacts<'_>) -> Option<bool> {
    facts.judged.then(|| {
        it.keywords
            .iter()
            .any(|k| k.strip_prefix("cat:") == Some(slug))
    })
}
/// The channels of the after-school cartoons (Cartoon Network, Boomerang, Disney XD, Adult Swim, Toonami): a series of theirs is not a preschool one whatever its British rating.
fn after_school_channel(it: &CatalogItem) -> bool {
    it.networks.iter().any(|n| {
        matches!(
            n.as_str(),
            "Cartoon Network" | "Boomerang" | "Disney XD" | "Adult Swim" | "Toonami"
        )
    })
}
pub use server_catalog::events::same_genre;
/// A keyword holding one of the words at the start of one of its words ("romance" in "teenage romance", not in "bromance"; "sport" in "sports car", not in
/// "passport").
fn kw_has(it: &CatalogItem, parts: &[&str]) -> bool {
    it.keywords
        .iter()
        .any(|k| parts.iter().any(|p| starts_a_word(k, p)))
}
fn starts_a_word(k: &str, p: &str) -> bool {
    k.match_indices(p)
        .any(|(i, _)| i == 0 || !k.as_bytes()[i - 1].is_ascii_alphanumeric())
}
fn from_country(it: &CatalogItem, codes: &[&str]) -> bool {
    it.countries.iter().any(|c| codes.contains(&c.as_str()))
}

impl GenreSpec {
    /// The calendar event this page is the page of (the collection banner of Home leads here), if it is one.
    pub fn event_id(&self) -> Option<&str> {
        match self {
            GenreSpec::Halloween => Some("halloween"),
            GenreSpec::Lgbtq => Some("pride"),
            GenreSpec::Event(e) => Some(e.as_str()),
            _ => None,
        }
    }

    /// Whether the page is offered today: an event page in its window only, every other page always.
    pub fn in_season(&self, events: &EventStore, today: (u8, u8)) -> bool {
        match self {
            GenreSpec::Event(e) => events.in_season(e, today),
            _ => true,
        }
    }
}

impl GenreSpec {
    /// Whether children's titles (Kids or Family genres) belong on a page of this genre. Only pages about animation, anime,
    /// kids, family or teens keep them; a page for any other genre (Action, Thriller, Pride) is for adults and teens.
    pub fn keeps_children(&self) -> bool {
        match self {
            GenreSpec::Any
            | GenreSpec::Anime
            | GenreSpec::Teen
            | GenreSpec::Unsupported
            | GenreSpec::Event(_) => true,
            GenreSpec::Names(names) => names.iter().any(|n| {
                matches!(
                    n.to_ascii_lowercase().as_str(),
                    "kids" | "family" | "animation"
                )
            }),
            GenreSpec::Name(n) => matches!(
                n.to_ascii_lowercase().as_str(),
                "kids" | "family" | "animation" | "kids & family"
            ),
            _ => false,
        }
    }

    /// Whether the page's members come from the judged lists (the judged file is loaded and this page is one of its categories).
    pub fn is_judged(&self, facts: &SpecFacts<'_>) -> bool {
        use GenreSpec::*;
        facts.judged
            && matches!(
                self,
                BlackStories
                    | Cult
                    | Halloween
                    | Independent
                    | Lgbtq
                    | ScienceNature
                    | Sports
                    | StandUp
                    | ChatShows
            )
    }

    pub fn matches(&self, it: &CatalogItem, facts: &SpecFacts<'_>) -> bool {
        use GenreSpec::*;
        // the pages whose membership was judged use the judged list alone; the keyword rules below only propose candidates
        let slug = match self {
            BlackStories => Some("black_stories"),
            Cult => Some("cult"),
            Halloween => Some("halloween"),
            Independent => Some("independent"),
            Lgbtq => Some("lgbtq"),
            ScienceNature => Some("science_nature"),
            Sports => Some("sport"),
            StandUp => Some("stand_up"),
            ChatShows => Some("chat_shows"),
            _ => None,
        };
        if let Some(slug) = slug
            && let Some(j) = judged(it, slug, facts)
        {
            // the Halloween series page  also takes the series that carry the event's keywords (witches, vampires, ghosts, zombies)
            return j
                || (matches!(self, ChatShows) && judged(it, "stand_up", facts) == Some(true))
                || (matches!(self, Halloween)
                    && it.media_type == "tv"
                    && facts.events.title_belongs("halloween", it));
        }
        match self {
            Any => true,
            Unsupported => false,
            Awards(family) => it.keywords.iter().any(|k| {
                k.strip_prefix("award:")
                    .is_some_and(|r| r.split(':').next() == Some(*family))
            }),
            Names(names) => has_genre(it, names),
            // For Little Kids (ages about 5 to 8): U rated animation or family titles that are not preschool ones (those are For Little Ones)
            // For Little Ones: preschool television: TV-Y, or animation rated TV-G or U that runs 12 minutes or less, or the preschool words
            // a TV-Y7 cartoon with a British U (Johnny Test, 11 minutes, Cartoon Network) is an after-school cartoon, not a preschool one: the US rating and the channel say so
            Kids("for little ones") => {
                it.cert_us == 6
                    || ((it.cert_us == 8 || (it.cert == 1 && it.cert_us != 7))
                        && (1..=12).contains(&it.runtime)
                        && has_genre(it, &["Animation"])
                        && !after_school_channel(it))
                    || kids_rule("for little ones").is_some_and(|(_, kws, _)| kw_is(it, kws))
            }
            // For Little Kids: the preschool titles  and short U rated animation or family films and specials
            Kids("little kids") => {
                kids_rule("for little ones").is_some_and(|(_, kws, _)| kw_is(it, kws))
                    || (it.cert == 1
                        && has_genre(it, &["Animation", "Family", "Kids"])
                        && it.runtime > 0
                        && it.runtime <= 75)
            }
            Kids(name) => kids_rule(name).is_some_and(|(_, kws, genres)| {
                kw_is(it, kws) || (!genres.is_empty() && has_genre(it, genres))
            }),
            Thriller => {
                has_genre(it, &["Thriller"])
                    || (it.media_type == "tv" && has_genre(it, &["Mystery"]))
            }
            Anime => it.is_anime,
            BlackStories => kw_is(
                it,
                &[
                    "black cinema",
                    "african american",
                    "blaxploitation",
                    "black history",
                    "civil rights",
                    "civil rights movement",
                    "slavery",
                    "racism",
                    "hip-hop",
                ],
            ),
            BookAdaptations => kw_is(
                it,
                &["based on novel or book", "based on young adult novel"],
            ),
            British => from_country(it, &["GB"]),
            Classics => {
                it.media_type == "movie"
                    && it.release_year.is_some_and(|y| y <= 1979)
                    && it.vote_average >= 7.0
            }
            Cult => kw_is(
                it,
                &[
                    "cult film",
                    "cult classic",
                    "cult",
                    "midnight movie",
                    "cult tv",
                    "cult favorite",
                    "grindhouse",
                    "exploitation",
                    "b movie",
                    "so bad it's good",
                    "trash cinema",
                    "splatter",
                    "campy",
                ],
            ),
            European => from_country(it, EUROPE) && !from_country(it, &["US", "GB"]),
            Halloween => {
                kw_has(it, &["halloween"])
                    || (it.media_type == "tv" && facts.events.title_belongs("halloween", it))
            }
            // series have no Horror or Romance genre, so they are found through keywords
            Horror => {
                has_genre(it, &["Horror"])
                    || (it.media_type == "tv"
                        && kw_has(
                            it,
                            &[
                                "horror", "slasher", "zombie", "haunted", "ghost", "vampire",
                                "demon",
                            ],
                        ))
            }
            Romance => {
                has_genre(it, &["Romance"])
                    || (it.media_type == "tv" && kw_has(it, &["romance", "romantic"]))
            }
            Independent => kw_is(
                it,
                &[
                    "independent film",
                    "indie film",
                    "indie",
                    "independent cinema",
                    "low budget",
                    "micro budget",
                    "no budget",
                    "arthouse",
                    "art house",
                    "experimental film",
                    "experimental cinema",
                ],
            ),
            International => {
                !it.original_language.is_empty()
                    && it.original_language != "en"
                    && it.original_language != "xx"
            }
            Lgbtq => it.keywords.iter().any(|k| {
                k == "gay"
                    || k.starts_with("gay ")
                    || k == "coming out"
                    || [
                        "lgbt",
                        "homosexual",
                        "lesbian",
                        "queer",
                        "transgender",
                        "bisexual",
                        "drag queen",
                        "drag race",
                    ]
                    .iter()
                    .any(|p| k.contains(p))
            }),
            Musical => has_genre(it, &["Music"]) || kw_has(it, &["musical"]),
            Reality => has_genre(it, &["Reality"]) || it.tv_type.as_deref() == Some("Reality"),
            Event(e) => facts.events.title_belongs(e, it),
            ScienceNature => {
                has_genre(it, &["Documentary"])
                    && kw_has(
                        it,
                        &[
                            "nature",
                            "wildlife",
                            "animal",
                            "science",
                            "space",
                            "ocean",
                            "planet",
                            "universe",
                            "dinosaur",
                            "environment",
                            "biology",
                            "physics",
                            "climate",
                        ],
                    )
            }
            Shorts => it.media_type == "movie" && it.runtime > 0 && it.runtime <= 40,
            Sports => kw_has(
                it,
                &[
                    "sport",
                    "football",
                    "basketball",
                    "boxing",
                    "wrestling",
                    "baseball",
                    "olympic",
                    "soccer",
                    "tennis",
                    "golf",
                    "cycling",
                    "formula",
                    "motorsport",
                    "athlet",
                    "marathon",
                    "swimming",
                    "hockey",
                    "rugby",
                    "cricket",
                    "skiing",
                    "skateboard",
                    "surf",
                ],
            ),
            StandUp => kw_has(
                it,
                &[
                    "stand-up comedy",
                    "stand-up comedian",
                    "stand-up special",
                    "stand up comedy",
                ],
            ),
            ChatShows => StandUp.matches(it, facts) || it.tv_type.as_deref() == Some("Talk Show"),
            Teen => kw_has(it, &["high school", "teen", "coming of age"]),
            Us => {
                if it.media_type == "tv" {
                    from_country(it, &["US"])
                } else {
                    from_country(it, &["US"]) && (it.vote_count >= 1000 || it.popularity >= 30.0)
                }
            }
            Name(n) => it.genres.iter().any(|g| g.eq_ignore_ascii_case(n)),
        }
    }

    /// True for collections whose own content the hero feed would otherwise exclude (documentary, reality, talk).
    pub fn includes_nonscripted(&self) -> bool {
        use GenreSpec::*;
        match self {
            Reality | ChatShows | ScienceNature => true,
            Names(n) => n
                .iter()
                .any(|g| ["Documentary", "News", "Talk", "Soap", "Reality"].contains(g)),
            Name(n) => ["documentary", "news", "talk", "soap", "reality"]
                .contains(&n.to_lowercase().as_str()),
            _ => false,
        }
    }
}

fn by_name(key: &str) -> Option<GenreSpec> {
    use GenreSpec::*;
    Some(match key {
        "action" => Names(&["Action", "Action & Adventure"]),
        "adventure" => Names(&["Adventure", "Action & Adventure"]),
        "animation" => Names(&["Animation"]),
        "anime" => Anime,
        "astrology" => Unsupported,
        "emmys" => Awards("emmy"),
        "black stories" => BlackStories,
        "book adaptations" => BookAdaptations,
        "british" => British,
        "classics" => Classics,
        "comedies" | "comedy" => Names(&["Comedy"]),
        "crime" => Names(&["Crime"]),
        "cult" => Cult,
        "documentaries" | "documentary" | "documentary series" => Names(&["Documentary"]),
        "dramas" | "drama" => Names(&["Drama"]),
        "european" => European,
        "fantasy" => Names(&["Fantasy", "Sci-Fi & Fantasy"]),
        "halloween" => Halloween,
        "history" => Names(&["History"]),
        "hollywood" | "us" => Us,
        "horror" => Horror,
        "independent" => Independent,
        "international" => International,
        "kids & family" | "kids" | "family" => Names(&["Family", "Kids"]),
        "lgbtq" | "pride" => Lgbtq,
        "moods" => Any,
        "music & musicals" | "music" | "musicals" => Musical,
        "mystery" | "mysteries" => Names(&["Mystery"]),
        "news" => Names(&["News"]),
        "reality" => Reality,
        "romance" => Romance,
        "sci-fi" | "science fiction" | "sci-fi & fantasy" => {
            Names(&["Science Fiction", "Sci-Fi & Fantasy"])
        }
        "science & nature" => ScienceNature,
        "shorts" => Shorts,
        "soap" => Names(&["Soap"]),
        "sports" | "sport" => Sports,
        "stand-up comedy" => StandUp,
        "stand-up & chat shows" => ChatShows,
        "talk" => Names(&["Talk"]),
        "teen" => Teen,
        "thriller" | "thrillers" => Thriller,
        "tv movie" => Names(&["TV Movie"]),
        "war" | "war & politics" => Names(&["War", "War & Politics"]),
        "western" => Western,
        "animated" => Names(&["Animation"]),
        "watch with the family" => Names(&["Family", "Kids"]),
        // the Kids menus' own pages
        "animal time" => Kids("animal time"),
        "animals" => Kids("animals"),
        "cars, trucks & trains" | "cars trucks & trains" => Kids("cars, trucks & trains"),
        "dinosaurs" => Kids("dinosaurs"),
        "feel-good" | "feel good" => Kids("feel-good"),
        "for little ones" => Kids("for little ones"),
        "for little kids" | "little kids" => Kids("little kids"),
        "friends" => Kids("friends"),
        "funny" => Kids("funny"),
        "girls take the lead" => Kids("girls take the lead"),
        "science & arts" => Kids("science & arts"),
        "singing & dancing" => Kids("singing & dancing"),
        "spooky stuff" => Kids("spooky stuff"),
        "cool special effects" => Kids("cool special effects"),
        "princess tales" => Kids("princess tales"),
        _ => return None,
    })
}
#[allow(non_upper_case_globals)]
const Western: GenreSpec = GenreSpec::Names(&["Western"]);

/// Resolve a page's name for a surface ("movie", "tv" or "both").
#[must_use]
pub fn resolve(raw: &str, _surface: &str, events: &EventStore) -> GenreSpec {
    let t = raw.trim();
    let key = t
        .to_lowercase()
        .replace(" and ", " & ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(spec) = by_name(&key) {
        return spec;
    }
    // an event by its id or its label
    if let Some(e) = events.get(&key).or_else(|| events.by_label(&key)) {
        return GenreSpec::Event(e.id.clone());
    }
    GenreSpec::Name(t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_events() -> &'static EventStore {
        static NONE: std::sync::OnceLock<EventStore> = std::sync::OnceLock::new();
        NONE.get_or_init(EventStore::default)
    }

    fn facts() -> SpecFacts<'static> {
        SpecFacts {
            events: no_events(),
            judged: false,
        }
    }

    fn item(mt: &str) -> CatalogItem {
        CatalogItem {
            id: 1,
            title_id: 1,
            media_type: mt.to_string(),
            title: "t".into(),
            release_year: Some(2000),
            genres: vec![],
            overview: String::new(),
            tagline: None,
            director: None,
            poster_path: None,
            backdrop_path: None,
            logo_path: None,
            vote_average: 7.5,
            popularity: 10.0,
            is_anime: false,
            vote_count: 0,
            adult: false,
            original_language: "en".into(),
            tv_type: None,
            countries: vec![],
            keywords: vec![],
            runtime: 0,
            networks: vec![],
            flags: 0,
            cert: 0,
            cert_gb: 0,
            cert_us: 0,
            other_names: vec![],
        }
    }

    #[test]
    fn names_and_ids_resolve_alike() {
        assert_eq!(
            resolve("Comedies", "movie", no_events()),
            resolve("Comedy", "tv", no_events())
        );
        assert_eq!(
            resolve("Sci-Fi", "both", no_events()),
            resolve("Sci-Fi & Fantasy", "tv", no_events())
        );
        assert_eq!(
            resolve("Stand-up and Chat Shows", "tv", no_events()),
            GenreSpec::ChatShows
        );
        assert_eq!(resolve("Hollywood", "movie", no_events()), GenreSpec::Us);
        assert_eq!(
            resolve("Emmys", "tv", no_events()),
            GenreSpec::Awards("emmy")
        );
        let mut c = item("tv");
        assert!(!resolve("Emmys", "tv", no_events()).matches(&c, &facts()));
        c.keywords = vec!["award:emmy:nominated".into()];
        assert!(resolve("Emmys", "tv", no_events()).matches(&c, &facts()));
        assert_eq!(
            resolve("Noir", "movie", no_events()),
            GenreSpec::Name("Noir".into())
        );
    }

    #[test]
    fn genre_names_cross_media() {
        let mut m = item("movie");
        m.genres = vec!["Science Fiction".into()];
        let mut t = item("tv");
        t.genres = vec!["Sci-Fi & Fantasy".into()];
        let s = resolve("Sci-Fi", "both", no_events());
        assert!(s.matches(&m, &facts()) && s.matches(&t, &facts()));
        let mut mys = item("tv");
        mys.genres = vec!["Mystery".into()];
        assert!(resolve("Thriller", "tv", no_events()).matches(&mys, &facts()));
        let mut mm = item("movie");
        mm.genres = vec!["Mystery".into()];
        assert!(!resolve("Thriller", "movie", no_events()).matches(&mm, &facts()));
    }

    #[test]
    fn collections_use_metadata() {
        let mut a = item("movie");
        a.countries = vec!["GB".into()];
        assert!(resolve("British", "movie", no_events()).matches(&a, &facts()));
        let mut e = item("tv");
        e.countries = vec!["FR".into()];
        assert!(resolve("European", "tv", no_events()).matches(&e, &facts()));
        e.countries = vec!["FR".into(), "US".into()];
        assert!(!resolve("European", "tv", no_events()).matches(&e, &facts()));
        let mut h = item("tv");
        h.keywords = vec!["zombie".into()];
        assert!(resolve("Horror", "tv", no_events()).matches(&h, &facts()));
        let mut hm = item("movie");
        hm.keywords = vec!["zombie".into()];
        assert!(
            !resolve("Horror", "movie", no_events()).matches(&hm, &facts()),
            "movies use the genre"
        );
        let mut r = item("tv");
        r.keywords = vec!["romance".into()];
        assert!(resolve("Romance", "tv", no_events()).matches(&r, &facts()));
        // a keyword word must start where the rule's word starts
        r.keywords = vec!["bromance".into()];
        assert!(!resolve("Romance", "tv", no_events()).matches(&r, &facts()));
        r.keywords = vec!["teenage romance".into()];
        assert!(resolve("Romance", "tv", no_events()).matches(&r, &facts()));
        let mut p = item("movie");
        p.keywords = vec!["passport".into()];
        assert!(!kw_has(&p, &["sport"]));
        p.keywords = vec!["sports car".into()];
        assert!(kw_has(&p, &["sport"]));
        let mut s = item("movie");
        s.runtime = 25;
        assert!(resolve("Shorts", "movie", no_events()).matches(&s, &facts()));
        s.runtime = 120;
        assert!(!resolve("Shorts", "movie", no_events()).matches(&s, &facts()));
        let mut l = item("movie");
        l.keywords = vec!["gay theme".into()];
        assert!(resolve("LGBTQ", "movie", no_events()).matches(&l, &facts()));
        l.keywords = vec!["dragon".into()];
        assert!(!resolve("Pride", "movie", no_events()).matches(&l, &facts()));
        let mut c = item("movie");
        c.keywords = vec!["occult".into()];
        assert!(!resolve("Cult", "movie", no_events()).matches(&c, &facts()));
        c.keywords = vec!["cult".into()];
        assert!(resolve("Cult", "movie", no_events()).matches(&c, &facts()));
        let mut cl = item("movie");
        cl.release_year = Some(1960);
        assert!(resolve("Classics", "movie", no_events()).matches(&cl, &facts()));
        cl.release_year = Some(1990);
        assert!(!resolve("Classics", "movie", no_events()).matches(&cl, &facts()));
        let mut i = item("tv");
        i.original_language = "ko".into();
        assert!(resolve("International", "tv", no_events()).matches(&i, &facts()));
        let mut talk = item("tv");
        talk.tv_type = Some("Talk Show".into());
        assert!(resolve("Stand-up & Chat Shows", "tv", no_events()).matches(&talk, &facts()));
    }
}
