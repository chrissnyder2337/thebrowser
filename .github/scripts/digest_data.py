#!/usr/bin/env python3
"""Collect the factory's weekly numbers as JSON, with no model.

Run weekly by .github/workflows/digest-data.yml (docs/software-factory.md,
Phase 3 → "Weekly digest" and "Metrics"); the digest routine in
docs/routines.md writes them up. For the last 7 and 28 days: PRs merged into
main by tier label, PRs closed without merging, reverts, and the first-push
green rate (a merged PR's first CI run on a pull_request event succeeded).

Reads the GitHub REST API through `gh api`, so it needs GH_TOKEN with
pull-requests, actions and contents read.

Usage: digest_data.py --repo OWNER/NAME [--now 2026-10-26T00:00:00Z] > digest.json
"""

import argparse
import json
import subprocess
from urllib.parse import quote
from datetime import datetime, timedelta, timezone

WINDOWS = (7, 28)
TIERS = ("tier:0", "tier:1", "tier:2")


def gh(path, key=None, stop=None):
    """Every item of a list endpoint (path has per_page=100), page by page.

    Explicit page numbers rather than --paginate, whose Link URLs the
    cloud sessions' GitHub proxy refuses, so the script also runs there. key picks the list out of an object response; stop(item)
    ends the walk early on a sorted list.
    """
    items = []
    for page in range(1, 100):
        out = subprocess.run(
            ["gh", "api", f"{path}&page={page}"],
            check=True, capture_output=True, text=True,
        ).stdout
        batch = json.loads(out)
        batch = batch[key] if key else batch
        for item in batch:
            if stop and stop(item):
                return items
            items.append(item)
        if len(batch) < 100:
            break
    return items


def ci_runs(repo, branch):
    """CI runs started by pull_request events on one head branch."""
    return gh(
        f"repos/{repo}/actions/workflows/ci.yml/runs?event=pull_request"
        f"&branch={quote(branch, safe='')}&per_page=100",
        key="workflow_runs",
    )


def parse(stamp):
    return datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)


def tier_of(pr):
    names = {label["name"] for label in pr.get("labels", [])}
    found = [t for t in TIERS if t in names]
    return found[-1] if found else "none"


def is_revert(title):
    return title.startswith("Revert ")


def first_push_green(pr, runs):
    """True/False for the PR's first CI run, None if it had none.

    Only runs created after the PR was opened count (head branches get
    reused), and any conclusion but success (failure, cancelled because a
    second push superseded it) is not green.
    """
    opened = parse(pr["created_at"])
    mine = [
        r for r in runs
        if parse(r["created_at"]) >= opened
        and (not r.get("pull_requests") or any(p["number"] == pr["number"] for p in r["pull_requests"]))
    ]
    if not mine:
        return None
    first = min(mine, key=lambda r: (r["created_at"], r["run_number"]))
    if first.get("status") != "completed":
        return None
    return first.get("conclusion") == "success"


def summarize(prs, direct_commits, runs_by_pr, now, days):
    """The numbers for the `days` before `now`.

    direct_commits are commits on main that belong to no PR (pushed
    directly); revert PRs are found by title, so a revert inside a PR's
    branch or a revert PR's merge commit isn't counted twice.
    """
    since = now - timedelta(days=days)

    def within(stamp):
        return since <= parse(stamp) <= now

    merged = [p for p in prs if p.get("merged_at") and within(p["merged_at"])]
    dropped = [
        p for p in prs
        if not p.get("merged_at") and p.get("closed_at") and within(p["closed_at"])
    ]
    by_tier = {t: [] for t in (*TIERS, "none")}
    for p in merged:
        by_tier[tier_of(p)].append(p["number"])
    reverts = [{"pr": p["number"], "title": p["title"]} for p in merged if is_revert(p["title"])]
    for c in direct_commits:
        subject = c["commit"]["message"].splitlines()[0]
        if is_revert(subject) and within(c["commit"]["committer"]["date"]):
            reverts.append({"commit": c["sha"][:7], "title": subject})
    verdicts = {p["number"]: first_push_green(p, runs_by_pr.get(p["number"], [])) for p in merged}
    judged = [n for n, v in verdicts.items() if v is not None]
    green = [n for n in judged if verdicts[n]]
    return {
        "days": days,
        "since": since.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "merged": {
            "count": len(merged),
            "by_tier": {t: {"count": len(ns), "prs": sorted(ns)} for t, ns in by_tier.items()},
            "prs": [{"number": p["number"], "title": p["title"], "tier": tier_of(p),
                     "author": p["user"]["login"]} for p in sorted(merged, key=lambda p: p["number"])],
        },
        "closed_unmerged": [{"number": p["number"], "title": p["title"]}
                            for p in sorted(dropped, key=lambda p: p["number"])],
        "reverts": reverts,
        "first_push_green": {
            "judged": len(judged),
            "green": len(green),
            "rate": round(len(green) / len(judged), 3) if judged else None,
            "not_green": sorted(n for n in judged if not verdicts[n]),
        },
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--now", help="end of the window, UTC (default: now)")
    args = parser.parse_args()
    now = parse(args.now) if args.now else datetime.now(timezone.utc).replace(microsecond=0)
    oldest = now - timedelta(days=max(WINDOWS))
    stamp = oldest.strftime("%Y-%m-%dT%H:%M:%SZ")

    # Newest-updated first; a PR closed in the window was updated in it too.
    prs = [
        p for p in gh(
            f"repos/{args.repo}/pulls?state=closed&base=main&sort=updated&direction=desc&per_page=100",
            stop=lambda p: parse(p["updated_at"]) < oldest,
        )
        if parse(p["closed_at"]) >= oldest
    ]
    # Revert commits that no PR contains: pushed straight to main.
    direct = [
        c for c in gh(f"repos/{args.repo}/commits?sha=main&since={stamp}&per_page=100")
        if is_revert(c["commit"]["message"]) and not gh(f"repos/{args.repo}/commits/{c['sha']}/pulls?per_page=100")
    ]
    runs_by_pr = {
        p["number"]: ci_runs(args.repo, p["head"]["ref"]) for p in prs if p.get("merged_at")
    }
    print(json.dumps({
        "repo": args.repo,
        "generated_at": now.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "windows": [summarize(prs, direct, runs_by_pr, now, d) for d in WINDOWS],
    }, indent=2))


if __name__ == "__main__":
    main()
