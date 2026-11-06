//! The genre pages a menu offers. A page id is this project's own word, stable and never reused:
//! `film.<name>`, `series.<name>`, `kids-film.<name>`, `kids-series.<name>`, `kids.<name>` for a page
//! that mixes both, and `event.<id>` for a calendar event.

/// What kinds of title a page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Surface {
    Movie,
    Tv,
    Both,
}

impl Surface {
    /// The word page builders compare with.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Tv => "tv",
            Self::Both => "both",
        }
    }
}

/// One entry of a menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenrePage {
    pub id: &'static str,
    pub name: &'static str,
    pub surface: Surface,
    pub kids: bool,
}

/// Every genre page, in menu order.
pub const PAGES: &[GenrePage] = &[
    GenrePage {
        id: "film.action",
        name: "Action",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.anime",
        name: "Anime",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.black-stories",
        name: "Black Stories",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.british",
        name: "British",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.classics",
        name: "Classics",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.comedies",
        name: "Comedies",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.crime",
        name: "Crime",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.cult",
        name: "Cult",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.documentaries",
        name: "Documentaries",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.dramas",
        name: "Dramas",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.european",
        name: "European",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.fantasy",
        name: "Fantasy",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.halloween",
        name: "Halloween",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.hollywood",
        name: "Hollywood",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.horror",
        name: "Horror",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.independent",
        name: "Independent",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.international",
        name: "International",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.kids-family",
        name: "Kids & Family",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.lgbtq",
        name: "LGBTQ",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.music-musicals",
        name: "Music & Musicals",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.romance",
        name: "Romance",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.sci-fi",
        name: "Sci-Fi",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.shorts",
        name: "Shorts",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.sports",
        name: "Sports",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.stand-up-comedy",
        name: "Stand-up Comedy",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "film.thriller",
        name: "Thriller",
        surface: Surface::Movie,
        kids: false,
    },
    GenrePage {
        id: "series.action",
        name: "Action",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.anime",
        name: "Anime",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.black-stories",
        name: "Black Stories",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.british",
        name: "British",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.comedies",
        name: "Comedies",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.crime",
        name: "Crime",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.documentary",
        name: "Documentary",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.dramas",
        name: "Dramas",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.emmys",
        name: "Emmys",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.european",
        name: "European",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.halloween",
        name: "Halloween",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.horror",
        name: "Horror",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.international",
        name: "International",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.kids",
        name: "Kids",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.mystery",
        name: "Mystery",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.reality",
        name: "Reality",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.romance",
        name: "Romance",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.sci-fi-fantasy",
        name: "Sci-Fi & Fantasy",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.science-nature",
        name: "Science & Nature",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.sport",
        name: "Sport",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.stand-up-chat-shows",
        name: "Stand-up & Chat Shows",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.teen",
        name: "Teen",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.thriller",
        name: "Thriller",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "series.us",
        name: "US",
        surface: Surface::Tv,
        kids: false,
    },
    GenrePage {
        id: "kids-series.action",
        name: "Action",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.animal-time",
        name: "Animal Time",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.cars-trucks-trains",
        name: "Cars, Trucks & Trains",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.dinosaurs",
        name: "Dinosaurs",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.fantasy",
        name: "Fantasy",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.feel-good",
        name: "Feel-good",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids.for-little-ones",
        name: "For Little Ones",
        surface: Surface::Both,
        kids: true,
    },
    GenrePage {
        id: "kids-series.friends",
        name: "Friends",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.funny",
        name: "Funny",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.girls-take-the-lead",
        name: "Girls Take the Lead",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.reality",
        name: "Reality",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.sci-fi",
        name: "Sci-Fi",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids.science-arts",
        name: "Science & Arts",
        surface: Surface::Both,
        kids: true,
    },
    GenrePage {
        id: "kids.science-nature",
        name: "Science & Nature",
        surface: Surface::Both,
        kids: true,
    },
    GenrePage {
        id: "kids-series.singing-dancing",
        name: "Singing & Dancing",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.spooky-stuff",
        name: "Spooky Stuff",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-series.watch-with-the-family",
        name: "Watch with the Family",
        surface: Surface::Tv,
        kids: true,
    },
    GenrePage {
        id: "kids-film.action",
        name: "Action",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.adventure",
        name: "Adventure",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.animals",
        name: "Animals",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.animated",
        name: "Animated",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.cool-special-effects",
        name: "Cool Special Effects",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.documentaries",
        name: "Documentaries",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.fantasy",
        name: "Fantasy",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.feel-good",
        name: "Feel-good",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.friends",
        name: "Friends",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.funny",
        name: "Funny",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.girls-take-the-lead",
        name: "Girls Take the Lead",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.for-little-kids",
        name: "For Little Kids",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.princess-tales",
        name: "Princess Tales",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.sci-fi",
        name: "Sci-Fi",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.singing-dancing",
        name: "Singing & Dancing",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.spooky-stuff",
        name: "Spooky Stuff",
        surface: Surface::Movie,
        kids: true,
    },
    GenrePage {
        id: "kids-film.watch-with-the-family",
        name: "Watch with the Family",
        surface: Surface::Movie,
        kids: true,
    },
];

/// One page by id.
#[must_use]
pub fn page(id: &str) -> Option<&'static GenrePage> {
    PAGES.iter().find(|page| page.id == id)
}

/// The names of the adult pages of a surface.
pub fn names(surface: &str) -> impl Iterator<Item = &'static str> {
    PAGES
        .iter()
        .filter(move |page| !page.kids && page.surface.word() == surface)
        .map(|page| page.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genre_spec::{GenreSpec, resolve};
    use server_catalog::events::EventStore;

    #[test]
    fn c26_every_page_id_is_unique_and_every_name_resolves_to_a_rule() {
        let events = EventStore::default();
        let mut ids: Vec<&str> = PAGES.iter().map(|page| page.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PAGES.len());
        for page in PAGES {
            let rule = resolve(page.name, page.surface.word(), &events);
            assert!(
                !matches!(rule, GenreSpec::Unsupported | GenreSpec::Name(_)),
                "{} has no rule",
                page.id
            );
        }
    }

    #[test]
    fn c26_page_ids_are_plain_words_and_not_numbers() {
        assert!(
            PAGES
                .iter()
                .all(|page| page.id.contains('.') && !page.id.chars().all(|c| c.is_ascii_digit()))
        );
    }
}
