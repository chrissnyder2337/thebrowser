#!/usr/bin/env python3
"""Bookkeeping for blackbox.yml: builds each pass's brief for the judge and
turns its answers into verdicts (docs/software-factory.md, Phase 1.6).

  verdicts.py brief PASS              write shots/brief.md for this pass
  verdicts.py record PASS EXEC_FILE   read the judge's verdicts from the action's
                                      execution file, move its shots aside
  verdicts.py summary                 write summary.md, print pass or fail

Each scenario runs once; one that isn't satisfied runs twice more and passes
only if both re-runs are satisfied (2 of 3). Everything public (the summary
comment, the job summary, the screenshot artifact) names scenarios only by
their opaque id, never by their text.

Paths come from the environment: BB (the private work directory, holding
scenarios/ and judge-prompt.md), FIXTURES_URL, BUILD (pr or a mutant name).
"""

import json
import os
import re
import shutil
import sys
from pathlib import Path

ID = re.compile(r"^s[0-9]{2,3}$")
VERDICTS = ("satisfied", "not_satisfied", "unclear")

work = Path(os.environ["BB"])
shots = Path("shots")


def scenarios():
    found = {}
    for path in sorted((work / "scenarios" / "scenarios").glob("*.md")):
        if not ID.match(path.stem):
            sys.exit(f"verdicts: scenario file names must be opaque ids like s01.md, not {path.name}")
        found[path.stem] = path.read_text()
    if not found:
        sys.exit("verdicts: no scenarios/*.md in the scenarios repo")
    return found


def load(n):
    path = work / f"pass-{n}.json"
    return json.loads(path.read_text()) if path.exists() else None


def to_run(n):
    """Ids pass `n` runs: all of them, then the first pass's failures, then
    those of them the second pass satisfied (only they can still reach 2 of 3)."""
    if n == 1:
        return list(scenarios())
    first = load(1)
    failed = [i for i, v in first.items() if v["verdict"] != "satisfied"]
    if n == 2:
        return failed
    second = load(2) or {}
    return [i for i in failed if second.get(i, {}).get("verdict") == "satisfied"]


def brief(n):
    ids = to_run(n)
    all_scenarios = scenarios()
    blocks = "\n\n".join(f"<scenario id=\"{i}\">\n{all_scenarios[i].strip()}\n</scenario>" for i in ids)
    text = (work / "judge-prompt.md").read_text()
    text = text.replace("{{FIXTURES_URL}}", os.environ["FIXTURES_URL"])
    text = text.replace("{{SCENARIOS}}", blocks)
    shots.mkdir(exist_ok=True)
    (shots / "brief.md").write_text(text)
    print(" ".join(ids))


def final_text(execution_file):
    """The judge's last message, from the action's execution file."""
    try:
        messages = json.loads(Path(execution_file).read_text())
    except (OSError, ValueError):
        return ""
    for message in reversed(messages):
        if message.get("type") == "result":
            return message.get("result") or ""
    return ""


def record(n, execution_file):
    ids = to_run(n)
    answers = {}
    blocks = re.findall(r"```json\s*(\{.*?\})\s*```", final_text(execution_file), re.S)
    if blocks:
        try:
            for item in json.loads(blocks[-1]).get("verdicts", []):
                answers[str(item.get("id"))] = item
        except (ValueError, AttributeError):
            pass
    result = {}
    for i in ids:
        item = answers.get(i, {})
        verdict = item.get("verdict")
        if verdict not in VERDICTS:
            verdict = "unclear"
        cited = [s for s in item.get("shots", []) if isinstance(s, str) and re.fullmatch(r"shots/[0-9]{4}\.png", s)]
        result[i] = {"verdict": verdict, "shots": [f"pass-{n}/{Path(s).name}" for s in cited]}
    (work / f"pass-{n}.json").write_text(json.dumps(result, indent=1))
    # The brief holds scenario text: it never goes into the public artifact.
    (shots / "brief.md").unlink(missing_ok=True)
    dest = work / "shots" / f"pass-{n}"
    dest.parent.mkdir(exist_ok=True)
    shutil.move(str(shots), dest)
    for i in ids:
        print(f"pass {n}: {i} {result[i]['verdict']}")


def decide(i):
    runs = [p[i]["verdict"] for p in (load(1), load(2), load(3)) if p and i in p]
    if runs and runs[0] == "satisfied":
        return "pass", runs
    if runs[1:3] == ["satisfied", "satisfied"]:
        return "pass", runs
    if "not_satisfied" not in runs:
        return "unclear", runs
    return "fail", runs


def summary():
    build = os.environ.get("BUILD", "pr")
    rows, outcomes = [], []
    for i in scenarios():
        outcome, runs = decide(i)
        outcomes.append(outcome)
        cited = [s for p in (load(1), load(2), load(3)) if p and i in p for s in p[i]["shots"]]
        rows.append(f"| `{i}` | **{outcome}** | {' → '.join(runs) or 'not run'} | {', '.join(cited[:4])} |")
    passed = all(o == "pass" for o in outcomes)
    if build == "pr":
        ok = passed
        head = "All scenarios passed." if passed else "Some scenarios didn't pass; the maintainer reviews the screenshots."
    else:
        # A broken build should fail something; passing it means the judge or
        # the scenarios need fixing before the tester is trusted again.
        ok = not passed
        head = (f"Mutant `{build}` was caught." if ok
                else f"**Mutant `{build}` passed every scenario: the judge or the scenarios need fixing.**")
    lines = [
        "### Black-box test",
        "",
        head,
        "",
        "| Scenario | Result | Runs | Screenshots |",
        "|---|---|---|---|",
        *rows,
        "",
        "Each scenario runs once; one that isn't satisfied runs twice more and passes only if both re-runs are. "
        "Screenshots are in the run's `blackbox-shots` artifact.",
    ]
    (work / "summary.md").write_text("\n".join(lines) + "\n")
    print("pass" if ok else "fail")


def main(argv):
    if argv[:1] == ["brief"] and len(argv) == 2:
        brief(int(argv[1]))
    elif argv[:1] == ["record"] and len(argv) == 3:
        record(int(argv[1]), argv[2])
    elif argv == ["summary"]:
        summary()
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
