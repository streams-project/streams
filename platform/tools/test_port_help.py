#!/usr/bin/env python3
"""Tests for platform/tools/port_help.py."""

from __future__ import annotations

import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import port_help  # noqa: E402

SAMPLE_HELP_RS = """
pub const LEGAL: &[LegalDoc] = &[
    LegalDoc { id: "privacy", path: "/help/legal/privacy", sections: 1 },
];
pub const POPULAR: &[&str] = &["what-is", "fake-sites"];
pub const TOPICS: &[Topic] = &[
    Topic {
        id: "account",
        icon: "ph-user",
        tone: "text-accent",
        groups: &[
            Group { id: "profiles", articles: &["what-is", "fake-sites", "doc:privacy"] },
        ],
    },
];
pub const ARTICLES: &[Article] = &[
    Article { slug: "what-is", related: &["profiles-manage", "fake-sites"] },
    Article { slug: "fake-sites", related: &["what-is"] },
];
"""


class PortHelpTest(unittest.TestCase):
    def setUp(self) -> None:
        self.folder = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.folder)
        self.tree = port_help.parse_help_rs(SAMPLE_HELP_RS)
        self.brand = port_help.read_brand_name()
        self.banned = port_help.read_banned_words()

    def test_multiline_indented_value_joins_correctly(self) -> None:
        ftl = (
            "help-art-what-is-body =\n"
            "    First line.\n"
            "    Second line.\n"
            "    Third line.\n"
        )
        entries = port_help.parse_ftl(ftl)
        self.assertEqual(
            entries["help-art-what-is-body"],
            "First line.\nSecond line.\nThird line.",
        )

    def test_blank_lines_inside_body_survive(self) -> None:
        ftl = (
            "help-art-what-is-body =\n"
            "    Paragraph one.\n"
            "\n"
            "    ## Heading\n"
            "    1. Item one.\n"
            "\n"
            "    !! Note: Kept.\n"
            "\n"
            "other-ignored-key = ignored\n"
        )
        entries = port_help.parse_ftl(ftl)
        self.assertEqual(
            entries["help-art-what-is-body"],
            "Paragraph one.\n\n## Heading\n1. Item one.\n\n!! Note: Kept.",
        )
        self.assertNotIn("other-ignored-key", entries)

    def test_brand_variable_is_converted(self) -> None:
        ftl = "help-art-what-is-title = Welcome to { $brand } today\n"
        entries = port_help.parse_ftl(ftl)
        data, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertEqual(errors, [])
        self.assertEqual(
            data["articles"]["what-is"]["title"],
            "Welcome to {brand} today",
        )

    def test_unknown_variable_stops_the_tool(self) -> None:
        ftl = "help-art-what-is-body = Write to { $mystery_var } now.\n"
        entries = port_help.parse_ftl(ftl)
        _, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertTrue(any("unknown variable $mystery_var" in e for e in errors))

    def test_doc_entries_land_in_policies_and_not_articles(self) -> None:
        ftl = (
            "help-topic-account = Account\n"
            "help-group-profiles = Profiles\n"
            "help-art-what-is-title = What is it\n"
            "help-art-what-is-summary = Summary\n"
            "help-art-what-is-keywords = words\n"
            "help-art-what-is-body = Body\n"
            "legal-privacy-title = Privacy Policy\n"
            "legal-privacy-updated = October 2026\n"
            "legal-privacy-1-title = 1. Data\n"
            "legal-privacy-1-body = We keep little.\n"
            "legal-privacy-1-b1 = First bullet.\n"
        )
        entries = port_help.parse_ftl(ftl)
        data, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertEqual(errors, [])
        self.assertIn("what-is", data["articles"])
        self.assertNotIn("privacy", data["articles"])
        self.assertNotIn("doc:privacy", data["articles"])
        self.assertIn("privacy", data["policies"])
        self.assertEqual(
            data["policies"]["privacy"]["sections"][0]["bullets"],
            ["First bullet."],
        )

    def test_english_fix_applies(self) -> None:
        ftl = "help-art-what-is-body = Old wording here.\n"
        entries = port_help.parse_ftl(ftl)
        fixes = {
            "english": [
                {
                    "key": "help-art-what-is-body",
                    "find": "Old wording",
                    "with": "New wording",
                }
            ]
        }
        data, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned, fixes=fixes
        )
        self.assertEqual(errors, [])
        self.assertEqual(data["articles"]["what-is"]["body"], "New wording here.")

    def test_missing_find_stops(self) -> None:
        ftl = "help-art-what-is-body = Something else.\n"
        entries = port_help.parse_ftl(ftl)
        fixes = {
            "english": [
                {
                    "key": "help-art-what-is-body",
                    "find": "Absent text",
                    "with": "Replacement",
                }
            ]
        }
        _, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned, fixes=fixes
        )
        self.assertTrue(any("missing find string" in e for e in errors))

    def test_email_variable_converts(self) -> None:
        ftl = "help-art-what-is-body = Contact { $help_email } for help.\n"
        entries = port_help.parse_ftl(ftl)
        fixes = {"email_variables": ["help_email"]}
        data, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned, fixes=fixes
        )
        self.assertEqual(errors, [])
        self.assertEqual(
            data["articles"]["what-is"]["body"],
            "Contact {support_email} for help.",
        )

    def test_dropped_slug_leaves_related(self) -> None:
        ftl = (
            "help-art-what-is-title = What is it\n"
            "help-art-fake-sites-title = Fake sites\n"
        )
        entries = port_help.parse_ftl(ftl)
        fixes = {"drop_articles": ["fake-sites"]}
        data, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned, fixes=fixes
        )
        self.assertEqual(errors, [])
        self.assertNotIn("fake-sites", data["articles"])
        self.assertNotIn("fake-sites", data["popular"])
        self.assertNotIn("fake-sites", data["topics"][0]["groups"][0]["articles"])
        self.assertEqual(data["articles"]["what-is"]["related"], ["profiles-manage"])

    def test_lowercase_streams_passes(self) -> None:
        ftl = f"help-art-what-is-body = Video {self.brand.lower()} load on demand.\n"
        entries = port_help.parse_ftl(ftl)
        data, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertEqual(errors, [])
        self.assertIn(self.brand.lower(), data["articles"]["what-is"]["body"])

    def test_translation_with_third_party_word_is_omitted_while_english_stops(self) -> None:
        bad_word = self.banned[0]
        ftl = (
            f"help-art-what-is-title = Safe title\n"
            f"help-art-what-is-body = Mentions {bad_word} here.\n"
        )
        entries = port_help.parse_ftl(ftl)
        _, en_errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertTrue(any("third party word" in e for e in en_errors))

        omitted: list[str] = []
        de_data, de_errors = port_help.build_lang(
            self.tree,
            entries,
            "de.ftl",
            self.brand,
            self.banned,
            en_entries={},
            omitted_out=omitted,
        )
        self.assertEqual(de_errors, [])
        self.assertEqual(len(omitted), 1)
        self.assertTrue(omitted[0].startswith("omitted de.ftl help-art-what-is-body"))
        self.assertNotIn("body", de_data["articles"]["what-is"])
        self.assertEqual(de_data["articles"]["what-is"]["title"], "Safe title")


if __name__ == "__main__":
    unittest.main()
