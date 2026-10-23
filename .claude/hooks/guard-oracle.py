#!/usr/bin/env python3
"""PreToolUse guard for the oracle (docs/software-factory.md, Phase 0).

Answers "ask" for:
- an edit or write to an existing file under tests/golden/;
- a Bash command that updates snapshots or writes into tests/golden/;
- a Bash or GitHub tool call that approves (reviews) a deployment, which
  would bypass the `oracle` GitHub Environment.

Anything else, including a brand-new file under tests/golden/, gets no
decision, so the normal permission rules apply. If the guard itself fails it
answers "ask" too, so it fails closed. This is a speed bump; the CI oracle
check is the lock.

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

# Commands that can create, change or delete files. Redirections to
# /dev/null or another descriptor (2>&1) don't count.
WRITES = re.compile(
    r"(^|[^0-9&>-])>>?(?!\s*(/dev/null|&))"
    r"|\b(tee|cp|mv|rm|rmdir|ln|install|rsync|truncate|dd|touch|chmod|unzip|tar"
    r"|import|convert|magick|python3?|node|perl|ruby)\b"
    r"|\bsed\b[^|;&]*\s-i"
    r"|\bgit\s+(checkout|restore|mv|rm|apply|am|reset|stash|clean|cherry-pick|revert)\b",
)

# The REST endpoint that approves or rejects a waiting environment.
DEPLOYMENT_APPROVAL = re.compile(r"pending_deployments", re.IGNORECASE)
# A tool whose name is about approving or reviewing deployments.
DEPLOYMENT_TOOL = re.compile(
    r"(approv|review).*deployment|deployment.*(approv|review)", re.IGNORECASE
)


def in_golden(path, root):
    """Whether `path` (absolute, or relative to `root`) is under tests/golden/."""
    full = os.path.realpath(os.path.join(root, path))
    rel = os.path.relpath(full, os.path.realpath(root)).replace(os.sep, "/")
    return (rel + "/").startswith(GOLDEN + "/"), full


def github_paths(args):
    """Repository paths a GitHub tool call would write (create, update, delete, push)."""
    paths = [args.get("path")]
    paths += [f.get("path") for f in args.get("files") or [] if isinstance(f, dict)]
    return [p for p in paths if isinstance(p, str) and p]


def decide(event):
    tool = event.get("tool_name", "")
    args = event.get("tool_input") or {}
    root = os.environ.get("CLAUDE_PROJECT_DIR") or event.get("cwd") or os.getcwd()
    cwd = event.get("cwd") or root

    if tool in EDIT_TOOLS:
        path = args.get("file_path") or args.get("notebook_path") or ""
        if not path:
            return None
        golden, full = in_golden(os.path.join(cwd, path), root)
        if golden and os.path.exists(full):
            return f"{tool} would change an existing golden file ({path})."
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
        if DEPLOYMENT_TOOL.search(tool) or DEPLOYMENT_APPROVAL.search(json.dumps(args)):
            return "This GitHub call may approve a deployment."
        # Writes through the API land on the branch directly; the guard can't
        # tell a new file from an existing one there, so it always asks.
        for path in github_paths(args):
            if path.lstrip("/").startswith(GOLDEN + "/"):
                return f"This GitHub call writes {path}."
    return None


def main():
    try:
        reason = decide(json.load(sys.stdin))
    except Exception as err:  # fail closed: a broken guard asks
        reason = f"The oracle guard failed ({err!r})."
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
