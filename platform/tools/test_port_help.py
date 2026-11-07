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
pub const POPULAR: &[&str] = &["what-is"];
pub const TOPICS: &[Topic] = &[
    Topic {
        id: "account",
        icon: "ph-user",
        tone: "text-accent",
        groups: &[
            Group { id: "profiles", articles: &["what-is", "doc:privacy"] },
        ],
    },
];
pub const ARTICLES: &[Article] = &[
    Article { slug: "what-is", related: &["profiles-manage"] },
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
        ftl = "help-art-what-is-body = Write to { $help_email } now.\n"
        entries = port_help.parse_ftl(ftl)
        _, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertTrue(any("unknown variable $help_email" in e for e in errors))

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

    def test_third_party_word_stops_the_tool(self) -> None:
        ftl = "help-art-what-is-body = Trailers play from YouTube.\n"
        entries = port_help.parse_ftl(ftl)
        _, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertTrue(any("third party word" in e for e in errors))

    def test_unsafe_literal_brand_word_stops_the_tool(self) -> None:
        ftl = f"help-art-what-is-body = Some video {self.brand.lower()} fail.\n"
        entries = port_help.parse_ftl(ftl)
        _, errors = port_help.build_lang(
            self.tree, entries, "en.ftl", self.brand, self.banned
        )
        self.assertTrue(any("unsafe literal brand word" in e for e in errors))


if __name__ == "__main__":
    unittest.main()
