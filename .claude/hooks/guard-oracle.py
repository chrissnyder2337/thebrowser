#!/usr/bin/env python3
"""PreToolUse guard for the oracle (docs/software-factory.md, Phase 0).

Answers "ask" for:
- an edit or write to an existing file under tests/golden/;
- a Bash command that updates snapshots or writes into tests/golden/;
- a Bash or GitHub tool call that approves (reviews) a deployment, which
  would bypass the `oracle` GitHub Environment.

Anything else, including a brand-new file under tests/golden/, gets no
decision, so the normal permission rules apply. This is a speed bump; the CI
oracle check is the lock.

Reads the hook input JSON on stdin and prints a decision JSON, or nothing.
"""

import json
import os
import re
import sys

GOLDEN = "tests/golden"

EDIT_TOOLS = {"Edit", "MultiEdit", "Write", "NotebookEdit"}

SNAPSHOT_UPDATE = re.compile(
    r"--update-snapshots"
    r"|\b(playwright|jest|vitest)\b.*\s-u(\s|$)"
    r"|\bcargo\s+insta\s+(accept|review)\b"
    r"|\bINSTA_UPDATE=(always|new|unseen)"
    r"|\bUPDATE_(GOLDEN|SNAPSHOTS?)=",
)

# Commands that can create, change or delete files.
WRITES = re.compile(
    r">|\b(tee|cp|mv|rm|rmdir|ln|install|rsync|truncate|dd|touch|chmod|unzip|tar"
    r"|import|convert|magick|python3?|node|perl|ruby)\b"
    r"|\bsed\b[^|;&]*\s-i"
    r"|\bgit\s+(checkout|restore|mv|rm|apply|am|reset|stash|clean|cherry-pick|revert)\b",
)

DEPLOYMENT_APPROVAL = re.compile(
    r"pending_deployments|review_?deployments?|approve_?deployments?"
    r"|deployments?.{0,40}\b(approv|review)|\b(approv|review).{0,40}deployments?",
    re.IGNORECASE,
)


def decide(event):
    tool = event.get("tool_name", "")
    args = event.get("tool_input") or {}
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    cwd = event.get("cwd") or root

    if tool in EDIT_TOOLS:
        path = args.get("file_path") or args.get("notebook_path") or ""
        if not path:
            return None
        full = os.path.normpath(os.path.join(cwd, path))
        rel = os.path.relpath(full, os.path.normpath(root))
        if (rel + "/").startswith(GOLDEN + "/") and os.path.exists(full):
            return f"{tool} would change an existing golden file ({rel})."
        return None

    if tool == "Bash":
        cmd = args.get("command", "")
        if SNAPSHOT_UPDATE.search(cmd):
            return "This command updates snapshots."
        if GOLDEN in cmd and WRITES.search(cmd):
            return f"This command may write into {GOLDEN}/."
        if DEPLOYMENT_APPROVAL.search(cmd):
            return "This command may approve a deployment."
        return None

    if "github" in tool.lower():
        text = tool + " " + json.dumps(args)
        if DEPLOYMENT_APPROVAL.search(text):
            return "This GitHub call may approve a deployment."
    return None


def main():
    reason = decide(json.load(sys.stdin))
    if reason:
        json.dump(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "ask",
                    "permissionDecisionReason": reason
                    + " Existing goldens, assertions and deployment approvals"
                    " are the maintainer's call (see the steward skill).",
                }
            },
            sys.stdout,
        )


if __name__ == "__main__":
    main()
