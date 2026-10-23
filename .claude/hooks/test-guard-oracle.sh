#!/usr/bin/env bash
# Pipes sample PreToolUse inputs into guard-oracle.py and checks each decision.
# Usage: .claude/hooks/test-guard-oracle.sh
set -euo pipefail

hook="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/guard-oracle.py"
project="$(mktemp -d)"
trap 'rm -rf "$project"' EXIT
mkdir -p "$project/tests/golden"
touch "$project/tests/golden/portal-home.png"
ln -s "$project/tests" "$project/link"

failures=0
check() {
  local want="$1" name="$2" input="$3" out got
  out="$(CLAUDE_PROJECT_DIR="$project" python3 "$hook" <<<"$input")"
  got="$(jq -r '.hookSpecificOutput.permissionDecision // empty' <<<"$out")"
  got="${got:-allow}"
  printf '%-5s %-46s %s\n' "$got" "$name" "$(jq -r '.hookSpecificOutput.permissionDecisionReason // ""' <<<"${out:-{\}}")"
  [ "$got" = "$want" ] || { echo "  FAIL: wanted $want" >&2; failures=$((failures + 1)); }
}

edit() { jq -nc --arg t "$1" --arg p "$2" --arg c "$project" \
  '{hook_event_name:"PreToolUse",cwd:$c,tool_name:$t,tool_input:{file_path:$p,content:"x"}}'; }
bash_() { jq -nc --arg cmd "$1" --arg c "$project" \
  '{hook_event_name:"PreToolUse",cwd:$c,tool_name:"Bash",tool_input:{command:$cmd}}'; }

check ask   "Edit existing golden file"        "$(edit Edit "$project/tests/golden/portal-home.png")"
check ask   "Write existing golden (relative)"  "$(edit Write tests/golden/portal-home.png)"
check allow "Write new golden file"             "$(edit Write "$project/tests/golden/geocities-home.png")"
check allow "Edit file outside tests/golden"    "$(edit Edit "$project/README.md")"
check ask   "Bash --update-snapshots"           "$(bash_ 'npx playwright test --update-snapshots')"
check ask   "Bash playwright -u"                "$(bash_ 'npx playwright test -u')"
check ask   "Bash cp into tests/golden"         "$(bash_ 'cp /tmp/shot.png tests/golden/portal-home.png')"
check ask   "Bash approve deployment (gh api)"  "$(bash_ 'gh api -X POST repos/o/r/actions/runs/1/pending_deployments -f state=approved')"
check allow "Bash read of tests/golden"         "$(bash_ 'ls -l tests/golden && git diff --stat tests/golden')"
check allow "Bash read with 2>/dev/null"         "$(bash_ 'ls tests/golden 2>/dev/null; git diff tests/golden 2>&1 | head')"
check ask   "Bash redirect into tests/golden"   "$(bash_ 'echo x > tests/golden/portal-home.png')"
check allow "Bash commit mentioning deployment" "$(bash_ "git commit -m 'CI: review deployment docs'")"
check ask   "Edit golden via symlinked path"    "$(edit Edit "$project/link/golden/portal-home.png")"
check allow "Bash unrelated command"            "$(bash_ 'cargo test --bin lantern')"
check ask   "GitHub tool approving a deployment" \
  "$(jq -nc '{tool_name:"mcp__github__review_pending_deployments",tool_input:{run_id:1,state:"approved"}}')"
check ask   "GitHub tool writing a golden"      \
  "$(jq -nc '{tool_name:"mcp__github__create_or_update_file",tool_input:{path:"tests/golden/portal-home.png",content:"x"}}')"
check ask   "GitHub push_files with a golden"   \
  "$(jq -nc '{tool_name:"mcp__github__push_files",tool_input:{files:[{path:"README.md"},{path:"tests/golden/a.png"}]}}')"
check allow "GitHub review mentioning deploys"  \
  "$(jq -nc '{tool_name:"mcp__github__pull_request_review_write",tool_input:{body:"Reviewed: the deployment step now caches CEF"}}')"
check ask   "Malformed input (fails closed)"    'not json'
check allow "GitHub tool unrelated"             \
  "$(jq -nc '{tool_name:"mcp__github__list_pull_requests",tool_input:{owner:"o",repo:"r"}}')"

if [ "$failures" -ne 0 ]; then
  echo "$failures case(s) failed" >&2
  exit 1
fi
echo "all cases passed"
