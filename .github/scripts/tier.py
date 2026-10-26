#!/usr/bin/env python3
"""Work out a pull request's risk tier from its changed files.

Run by .github/workflows/tier.yml from the base branch's checkout, so a PR
can't change the rules that judge it. Reads the PR's files as JSON lines
(one GitHub API pull request file object per line: filename, status,
previous_filename, patch), prints the tier (0, 1 or 2) on stdout, and the
reason for each path on stderr.

Usage: tier.py --config .github/tiers.yml --files files.jsonl
               --author LOGIN [--expected-count N]
"""

import argparse
import json
import re
import sys

KEYS = ("tier0", "tier0_new", "tier1", "tier2", "uses_only_authors")
# The API lists at most this many files; a bigger PR can't be judged.
API_FILE_LIMIT = 3000
# One `uses:` line, as Dependabot writes it: an action ref (a tag or SHA),
# maybe a version comment. Group 1 is everything up to the `@`, which a bump
# must leave unchanged.
USES_LINE = re.compile(
    r"^(\s*(?:-\s+)?uses:\s*[A-Za-z0-9._/-]+)@[A-Za-z0-9._/-]+(?:\s+#[ A-Za-z0-9._/-]*)?$"
)


def load_config(text):
    """Parse tiers.yml: the five keys, each a list of quoted strings."""
    config = {}
    current = None
    for number, raw in enumerate(text.splitlines(), 1):
        line = raw.split(" #", 1)[0].rstrip()
        if not line or line.lstrip().startswith("#"):
            continue
        key = re.fullmatch(r"([a-z0-9_]+):", line)
        item = re.fullmatch(r'  - "([^"\\]+)"', line)
        if key and key.group(1) in KEYS and key.group(1) not in config:
            current = config[key.group(1)] = []
        elif item and current is not None:
            current.append(item.group(1))
        else:
            raise ValueError(f"tiers.yml line {number}: unexpected {raw!r}")
    missing = [k for k in KEYS if k not in config]
    if missing:
        raise ValueError(f"tiers.yml: missing {', '.join(missing)}")
    return config


def glob_regex(pattern):
    """`**` spans directories, `*` and `?` stay within one."""
    out = []
    i = 0
    while i < len(pattern):
        if pattern.startswith("**/", i):
            out.append("(?:.*/)?")
            i += 3
        elif pattern.startswith("**", i):
            out.append(".*")
            i += 2
        elif pattern[i] == "*":
            out.append("[^/]*")
            i += 1
        elif pattern[i] == "?":
            out.append("[^/]")
            i += 1
        else:
            out.append(re.escape(pattern[i]))
            i += 1
    return re.compile("".join(out) + r"\Z")


def matches(path, patterns):
    return any(glob_regex(p).match(path) for p in patterns)


def uses_only(entry):
    """True if a workflow file's patch only swaps `uses:` refs in place."""
    if entry.get("status") != "modified" or not entry.get("patch"):
        return False
    removed, added, changed = [], [], False
    for line in entry["patch"].splitlines() + ["@@"]:
        if line.startswith("@@") or line.startswith(" ") or line == "":
            # A run of changes ends: each removed `uses:` line must be
            # replaced by one for the same action, at the same indentation
            # and in the same order.
            if removed or added:
                if len(removed) != len(added):
                    return False
                for old, new in zip(removed, added):
                    a, b = USES_LINE.match(old), USES_LINE.match(new)
                    if not (a and b and a.group(1) == b.group(1)):
                        return False
                changed = True
                removed, added = [], []
        elif line.startswith("-"):
            removed.append(line[1:])
        elif line.startswith("+"):
            added.append(line[1:])
        elif line.startswith("\\"):
            continue  # "\ No newline at end of file"
        else:
            return False
    return changed


def path_tier(path, config, added=False):
    if path.startswith(".github/") or matches(path, config["tier2"]):
        return 2
    if matches(path, config["tier1"]):
        return 1
    if matches(path, config["tier0"]) or (added and matches(path, config["tier0_new"])):
        return 0
    return 1


def pr_tier(entries, config, author, expected_count=None):
    """Return (tier, reasons). The highest tier of any path wins."""
    reasons = []
    if expected_count is not None and (
        expected_count != len(entries) or expected_count >= API_FILE_LIMIT
    ):
        return 2, [f"file list incomplete: got {len(entries)} of {expected_count}"]
    if not entries:
        return 1, ["no changed files"]
    bump = author in config["uses_only_authors"]
    tier = 0
    for entry in entries:
        paths = [entry["filename"]]
        if entry.get("previous_filename"):
            paths.append(entry["previous_filename"])
        for path in paths:
            t = path_tier(path, config, entry.get("status") == "added")
            why = f"tier {t}"
            if (
                t == 2
                and bump
                and len(paths) == 1
                and path.startswith(".github/workflows/")
                and uses_only(entry)
            ):
                t, why = 1, "tier 1 (uses:-only bump)"
            reasons.append(f"{path}: {why}")
            tier = max(tier, t)
    return tier, reasons


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--config", required=True)
    parser.add_argument("--files", required=True)
    parser.add_argument("--author", required=True)
    parser.add_argument("--expected-count", type=int)
    args = parser.parse_args(argv)

    with open(args.config, encoding="utf-8") as f:
        config = load_config(f.read())
    with open(args.files, encoding="utf-8") as f:
        entries = [json.loads(line) for line in f if line.strip()]
    tier, reasons = pr_tier(entries, config, args.author, args.expected_count)
    for reason in reasons:
        print(reason, file=sys.stderr)
    print(tier)
    return 0


if __name__ == "__main__":
    sys.exit(main())
