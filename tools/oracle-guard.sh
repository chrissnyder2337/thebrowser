#!/usr/bin/env bash
# Oracle guard (docs/software-factory.md, Phase 1.5): fails when a change
# touches product code (src/, ui/) AND modifies or deletes part of the
# existing oracle:
#   - an existing file under tests/golden/ (modified or deleted; a rename
#     counts as a deletion);
#   - an existing test file under tests/ (deleted), or any removed or changed
#     line in one (assertions, expected values, case tables, lists of looks);
#   - a removed or changed line in a Rust test module (from the file's first
#     `#[cfg(test)]` on);
#   - an added line that skips tests (`#[ignore]`, `test.skip`, `.fixme`, ...);
#   - any change to the suite's settings or to this guard: playwright.config.js,
#     package.json, tools/oracle-guard.sh.
# Adding goldens, fixtures, tests and assertions is always fine.
#
# Usage: tools/oracle-guard.sh [base]   (default origin/main; compares the
#        merge base of <base> and HEAD with HEAD)
#
# Exit 0: no oracle change, or no product change. Exit 1: both; the CI
# override is an approval on the "oracle" GitHub Environment. Exit 2: error.
set -euo pipefail
trap 'echo "oracle-guard: error on line $LINENO" >&2; exit 2' ERR

base="${1:-origin/main}"
merge_base="$(git merge-base "$base" HEAD)" || {
  echo "oracle-guard: no merge base between $base and HEAD" >&2
  exit 2
}

# Added lines that switch tests off.
skips='#\[ignore|\.(skip|fixme|fail|only)\(|test\.describe\.(skip|fixme)|forbidOnly|updateSnapshots'
config='^(playwright\.config\.js|package\.json|tools/oracle-guard\.sh)$'

# Diffs are read into variables first, so a failing git command stops the
# script (exit 2) instead of looking like an empty diff.
names="$(git diff --no-renames --name-status "$merge_base" HEAD)"

product=()
oracle=()
rust_tests=()
while IFS=$'\t' read -r status path; do
  [ -n "$path" ] || continue
  case "$path" in src/* | ui/*) product+=("$path") ;; esac
  [ "$status" = "A" ] && continue
  kind="modified"
  [ "$status" = "D" ] && kind="deleted"
  if [[ "$path" =~ $config ]]; then
    oracle+=("$path: test settings or guard $kind")
    continue
  fi
  case "$path" in
    tests/golden/*) oracle+=("$path: existing golden $kind") ;;
    tests/fixtures/*) ;;
    tests/*) [ "$kind" = "deleted" ] && oracle+=("$path: existing test file deleted") ;;
    src/*.rs) [ "$kind" = "modified" ] && rust_tests+=("$path") ;;
  esac
done <<< "$names"

# Removed lines in existing test files (fixtures and goldens are covered above).
diff_tests="$(git diff -U0 --no-renames --diff-filter=M "$merge_base" HEAD -- 'tests/*' ':!tests/golden/*' ':!tests/fixtures/*')"
while IFS= read -r line; do
  [ -n "$line" ] && oracle+=("$line")
done <<< "$(awk '
  /^--- / { file = substr($0, 5); sub(/^a\//, "", file); next }
  /^-/ { text = substr($0, 2); gsub(/^[ \t]+/, "", text); if (text != "") print file ": removed or changed test line: " substr(text, 1, 100) }
' <<< "$diff_tests")"

# Removed lines in Rust test modules: hunks whose old lines start at or after
# the old file's first #[cfg(test)].
for path in "${rust_tests[@]+"${rust_tests[@]}"}"; do
  old="$(git show "$merge_base:$path")"
  start="$(grep -n -m1 '^[[:space:]]*#\[cfg(test)\]' <<< "$old" | cut -d: -f1 || true)"
  [ -n "$start" ] || continue
  diff_rs="$(git diff -U0 --no-renames "$merge_base" HEAD -- "$path")"
  while IFS= read -r line; do
    [ -n "$line" ] && oracle+=("$line")
  done <<< "$(awk -v file="$path" -v start="$start" '
    /^@@ / { split($2, a, ","); n = substr(a[1], 2) + 0; next }
    /^-/ && !/^--- / { text = substr($0, 2); gsub(/^[ \t]+/, "", text); if (n >= start && text != "") print file ": removed or changed test line: " substr(text, 1, 100); n++ }
  ' <<< "$diff_rs")"
done

# Added lines that skip or loosen tests, anywhere in tests/ or src/.
diff_added="$(git diff -U0 --no-renames "$merge_base" HEAD -- 'tests/*' 'src/*.rs' ':!tests/golden/*')"
while IFS= read -r line; do
  [ -n "$line" ] && oracle+=("$line")
done <<< "$(awk -v re="$skips" '
  /^\+\+\+ / { file = substr($0, 5); sub(/^b\//, "", file); next }
  /^\+/ { text = substr($0, 2); if (text ~ re) { gsub(/^[ \t]+/, "", text); print file ": added a test skip: " substr(text, 1, 100) } }
' <<< "$diff_added")"

if [ "${#oracle[@]}" -eq 0 ]; then
  echo "oracle-guard: ok, no existing golden or assertion changed."
  exit 0
fi
if [ "${#product[@]}" -eq 0 ]; then
  echo "oracle-guard: ok, oracle changes without product changes:"
  printf '  %s\n' "${oracle[@]}"
  exit 0
fi
echo "oracle-guard: FAIL, this change touches product code and the existing oracle:"
printf '  %s\n' "${oracle[@]}"
echo "Product files changed: ${#product[@]} (for example ${product[0]})."
echo "Existing goldens and assertions are the maintainer's call: they approve the"
echo "\"oracle\" environment in this PR's checks, or land the change separately."
exit 1
