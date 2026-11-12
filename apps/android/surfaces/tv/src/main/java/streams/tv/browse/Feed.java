package streams.tv.browse;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * What a page of rows draws, flattened out of the contract into plain Java.
 *
 * <h2>Why this type exists, which is the one thing to read before changing it</h2>
 * Java source cannot read the contract. Verified on 2026-11-12 with {@code javap} against
 * {@code apps/android/ports/build/classes/kotlin/main}: {@code contract.Tile}'s title getter is
 * {@code getTitle-syqQ2Kk()}, {@code contract.Section}'s kind is {@code getKind-TNS5Z4k()},
 * {@code contract.NavItem}'s screen is {@code getScreen-mbV6UQU()} and {@code contract.Image}'s role
 * is {@code getRole-pIcKOd0()}. A hyphen is not a legal character in a Java identifier, so those
 * four calls cannot be written in this language at all. Note 210 section 4 measured the whole of it
 * -- 69 classes, 184 mangled getters -- and made it wave 0's package S1, which has not landed in the
 * Android build: {@code platform/contract/generated/types/Contract.java} is generated and on disk,
 * but {@code ports/build.gradle.kts:14} adds that folder as a <em>Kotlin</em> source directory and
 * the only class built from it is {@code ContractKt}.
 *
 * So one Kotlin file reads the contract and fills this ({@code browse/Reading.kt}), and every other
 * part of this page -- the geometry, the key map, the focus memory, what a screen reader is handed,
 * the drawing -- is Java against these types. It is wave 1's own precedent: {@code RowMemory} takes
 * {@code String[] titles} rather than {@code List<Tile>} for exactly this reason, and says so.
 *
 * <h2>What it is not</h2>
 * Not a cache and not a model: it is one arrival, built fresh each time the core publishes a
 * {@code Seen}, and the screen keeps no reference to the one before. What survives an arrival is
 * where the viewer was, and that lives in {@link Place} and in {@code RowMemory}, keyed by the row's
 * id and the title that was on it, so a reordered answer does not move the focus.
 *
 * <h2>And it carries no words of its own</h2>
 * Every string here is either a title's name from the catalogue, which is already in the viewer's
 * language, or a word the Kotlin seam read out of {@code seen.words}. Nothing in this file is
 * written in English, which is the rule {@code platform/tools/check_strings.py} enforces and
 * {@code Words.java} explains.
 */
public final class Feed {
    /** Where the top bar's focus goes to the left of the first tab. {@code tv/browse/TopBar.kt:31}. */
    public static final int SEARCH_TAB = -2;

    /** And to the left of search. {@code tv/browse/TopBar.kt:34}. */
    public static final int WHO_TAB = -3;

    /** One tab of the bar. */
    public static final class Tab {
        /** The name the server sent with it, so it is never written in the code. */
        public final String label;

        /** The screen it opens, for {@code shell.go}. Read through the seam. */
        public final String screen;

        public Tab(String label, String screen) {
            this.label = label == null ? "" : label;
            this.screen = screen == null ? "" : screen;
        }
    }

    /** Who is watching, for the face at the left of the bar. */
    public static final class Who {
        public final String name;

        /** The picture, already made absolute against the server, or empty for the letter. */
        public final String picture;

        /** True for a child's profile, which changes the plate behind the letter. */
        public final boolean kids;

        public Who(String name, String picture, boolean kids) {
            this.name = name == null ? "" : name;
            this.picture = picture == null ? "" : picture;
            this.kids = kids;
        }
    }

    /** One title in a row. */
    public static final class Card {
        /**
         * The title's key as a plain string, which is what {@code RowMemory} keeps a row's place
         * under and what the seam turns back into an address when the viewer chooses.
         */
        public final String key;

        /** The title's own name, from the catalogue. */
        public final String name;

        /** The picture for a tall place: {@code tall(tile.images)}, {@code shared/Art.kt:22}. */
        public final String tall;

        /** And for a wide one: {@code wide(tile.images)}, {@code shared/Art.kt:19}. */
        public final String wide;

        /**
         * What the badges on this tile say, already translated and counted by the seam.
         * {@code parts/Badge.kt:23-30} is where the two kinds become words.
         */
        public final String[] badges;

        public Card(String key, String name, String tall, String wide, String[] badges) {
            this.key = key == null ? "" : key;
            this.name = name == null ? "" : name;
            this.tall = tall == null ? "" : tall;
            this.wide = wide == null ? "" : wide;
            this.badges = badges == null ? new String[0] : badges;
        }

        /**
         * True for a series, which is what decides whether the hero's Play opens the title's page
         * or starts playing ({@code tv/browse/Browse.kt:227}).
         *
         * The test is the one at {@code shared/Addresses.kt:17}, {@code title.value.startsWith("tv:")},
         * and it is a wire prefix rather than a word, so it is the same four characters in every
         * language. It is answered here and not by the seam so that the whole of what a key does --
         * including what OK chooses -- is one pure Java state machine with a unit test, which is
         * note 210's own mitigation for the largest risk in this programme. {@code NumbersTest} holds
         * this prefix against that line.
         */
        public boolean series() {
            return key.startsWith("tv:");
        }
    }

    /** One row of titles. */
    public static final class Row {
        /** The section's own id, which is the key a row's place is kept under. */
        public final String id;

        /** The row's name, from the server. */
        public final String title;

        public final List<Card> cards;

        public Row(String id, String title, List<Card> cards) {
            this.id = id == null ? "" : id;
            this.title = title == null ? "" : title;
            this.cards = cards == null ? Collections.<Card>emptyList() : cards;
        }

        /** The title keys of this row, in the order it draws them, for {@code RowMemory}. */
        public String[] keys() {
            String[] keys = new String[cards.size()];
            for (int at = 0; at < keys.length; at++) {
                keys[at] = cards.get(at).key;
            }
            return keys;
        }

        /** The card at a place, or null past the end. */
        public Card at(int index) {
            return index >= 0 && index < cards.size() ? cards.get(index) : null;
        }
    }

    /**
     * What is known about the title the remote is on: the line of facts and the story.
     *
     * Built by the seam because both halves need the words and the plural rules
     * ({@code shared/Theme.kt:131}), and note 210 section 3's reason for not re-writing the CLDR
     * table in Java holds for the facts line too: one statement of one rule.
     */
    public static final class Story {
        /** Genre, year, length, age, in that order on every screen. {@code parts/Facts.kt:52}. */
        public final String[] facts;

        /** The title's own description. */
        public final String overview;

        /** The title's name drawn as a picture, for the hero, or empty. */
        public final String logo;

        public Story(String[] facts, String overview, String logo) {
            this.facts = facts == null ? new String[0] : facts;
            this.overview = overview == null ? "" : overview;
            this.logo = logo == null ? "" : logo;
        }
    }

    /** Nothing known yet: what a page draws before its first answer. */
    public static final Feed NOTHING =
            new Feed(new ArrayList<Tab>(), -1, null, "", new ArrayList<Row>(), null, "");

    public final List<Tab> tabs;

    /** Which tab this page is, or -1 when the page is not one of them. */
    public final int active;

    public final Who who;

    /** Where the pictures come from, for the face and the mark. */
    public final String server;

    public final List<Row> rows;

    /** The title the top of the page opens with, or null for a page with no hero of its own. */
    public final Card hero;

    /**
     * What a page with nothing at all says, already read from the words: {@code games.soon} or
     * {@code mine.empty} ({@code tv/browse/Browse.kt:291}). Empty when the page has rows.
     */
    public final String empty;

    public Feed(List<Tab> tabs, int active, Who who, String server, List<Row> rows, Card hero,
            String empty) {
        this.tabs = tabs == null ? Collections.<Tab>emptyList() : tabs;
        this.active = active;
        this.who = who;
        this.server = server == null ? "" : server;
        this.rows = rows == null ? Collections.<Row>emptyList() : rows;
        this.hero = hero;
        this.empty = empty == null ? "" : empty;
    }

    /** The row at a place, or null past the end. {@code rows.getOrNull(index)}. */
    public Row row(int index) {
        return index >= 0 && index < rows.size() ? rows.get(index) : null;
    }

    /** The last row's place, or -1 for a page with no rows. {@code rows.lastIndex}. */
    public int lastRow() {
        return rows.size() - 1;
    }

    /** The tab at a place, or null, which is what the bar's three negative places need. */
    public Tab tab(int index) {
        return index >= 0 && index < tabs.size() ? tabs.get(index) : null;
    }
}
