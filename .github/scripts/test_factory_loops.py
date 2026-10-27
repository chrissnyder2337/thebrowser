"""Tests for cef_watch.py and digest_data.py (the Phase 3 loops).

Run: python3 -m unittest discover -s .github/scripts -v
"""

import os
import sys
import unittest
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import cef_watch  # noqa: E402
import digest_data  # noqa: E402


def v(num, yanked=False):
    return {"num": num, "yanked": yanked}


class CefWatch(unittest.TestCase):
    def test_reads_the_real_pin(self):
        with open(os.path.join(HERE, "..", "..", "Cargo.toml"), encoding="utf-8") as f:
            self.assertRegex(cef_watch.pinned(f.read()), r"^\d+\.\d+\.\d+$")

    def test_newer_ignores_build_metadata(self):
        self.assertIsNone(cef_watch.newer("154.5.0", [v("154.5.0+154.0.34"), v("154.4.0+154.0.33")]))
        self.assertEqual(
            cef_watch.newer("154.5.0", [v("156.1.0+156.0.8"), v("156.2.0+156.0.14"), v("154.5.0+154.0.34")]),
            "156.2.0+156.0.14",
        )

    def test_skips_yanked_and_prereleases(self):
        versions = [v("157.0.0", yanked=True), v("157.1.0-beta.1"), v("155.0.0")]
        self.assertEqual(cef_watch.newer("154.5.0", versions), "155.0.0")

    def test_compares_numerically(self):
        self.assertEqual(cef_watch.newer("9.0.0", [v("10.0.0"), v("9.10.0")]), "10.0.0")


def pr(number, title="Area: change", tier=None, created="2026-10-20T10:00:00Z",
       merged="2026-10-21T10:00:00Z", closed=None):
    return {
        "number": number, "title": title, "user": {"login": "someone"},
        "labels": [{"name": tier}] if tier else [],
        "created_at": created, "merged_at": merged, "closed_at": closed or merged,
        "head": {"ref": f"branch-{number}"},
    }


def run(created, conclusion, number=None, run_number=1, status="completed"):
    return {
        "created_at": created, "conclusion": conclusion, "status": status, "run_number": run_number,
        "pull_requests": [{"number": number}] if number else [],
    }


NOW = datetime(2026, 10, 26, tzinfo=timezone.utc)


class Digest(unittest.TestCase):
    def test_counts_by_tier_and_window(self):
        prs = [
            pr(1, tier="tier:0"),
            pr(2, tier="tier:2"),
            pr(3),
            pr(4, tier="tier:1", merged="2026-10-01T00:00:00Z"),
            pr(5, merged=None, closed="2026-10-22T00:00:00Z"),
        ]
        week = digest_data.summarize(prs, [], {}, NOW, 7)
        self.assertEqual(week["merged"]["count"], 3)
        self.assertEqual(week["merged"]["by_tier"]["tier:0"]["prs"], [1])
        self.assertEqual(week["merged"]["by_tier"]["none"]["prs"], [3])
        self.assertEqual(week["closed_unmerged"], [{"number": 5, "title": "Area: change"}])
        self.assertEqual(digest_data.summarize(prs, [], {}, NOW, 28)["merged"]["by_tier"]["tier:1"]["prs"], [4])

    def test_reverts_from_pr_titles_and_direct_commits(self):
        prs = [pr(7, title='Revert "Art: palette"')]
        direct = [
            {"sha": "abcdef1234", "commit": {"message": 'Revert "UI: toolbar"\n\nbody',
                                              "committer": {"date": "2026-10-22T00:00:00Z"}}},
            {"sha": "1234567890", "commit": {"message": 'Revert "UI: old"',
                                              "committer": {"date": "2026-09-01T00:00:00Z"}}},
            {"sha": "fedcba9876", "commit": {"message": "Docs: Revert is a word",
                                              "committer": {"date": "2026-10-22T00:00:00Z"}}},
        ]
        self.assertEqual(digest_data.summarize(prs, direct, {}, NOW, 7)["reverts"], [
            {"pr": 7, "title": 'Revert "Art: palette"'},
            {"commit": "abcdef1", "title": 'Revert "UI: toolbar"'},
        ])

    def test_window_ends_at_now(self):
        prs = [pr(1, merged="2026-10-25T00:00:00Z"), pr(2, merged="2026-10-27T00:00:00Z")]
        self.assertEqual(digest_data.summarize(prs, [], {}, NOW, 7)["merged"]["by_tier"]["none"]["prs"], [1])

    def test_first_push_is_the_first_run_after_the_pr_opened(self):
        p = pr(9)
        runs = [
            run("2026-10-19T00:00:00Z", "success"),  # an older PR on the same branch
            run("2026-10-20T10:05:00Z", "failure", number=9, run_number=2),
            run("2026-10-20T11:00:00Z", "success", number=9, run_number=3),
        ]
        self.assertIs(digest_data.first_push_green(p, runs), False)
        self.assertIs(digest_data.first_push_green(p, runs[:1] + runs[2:]), True)
        self.assertIsNone(digest_data.first_push_green(p, runs[:1]))
        # Another PR's run on the same branch doesn't count.
        self.assertIsNone(digest_data.first_push_green(p, [run("2026-10-20T10:05:00Z", "success", number=8)]))

    def test_rate(self):
        prs = [pr(1), pr(2), pr(3)]
        runs = {1: [run("2026-10-20T11:00:00Z", "success")], 2: [run("2026-10-20T11:00:00Z", "cancelled")]}
        green = digest_data.summarize(prs, [], runs, NOW, 7)["first_push_green"]
        self.assertEqual(green, {"judged": 2, "green": 1, "rate": 0.5, "not_green": [2]})


if __name__ == "__main__":
    unittest.main()
