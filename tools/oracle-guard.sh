#!/usr/bin/env bash
# Oracle guard (docs/software-factory.md, Phase 1.5): fails when a change
# touches product code (src/, ui/) AND modifies or deletes part of the
# existing oracle:
#   - an existing file under tests/golden/ (modified, or deleted; a rename
#     counts as a deletion);
#   - an existing test file under tests/ (deleted);
#   - an existing assertion: a removed or changed line that asserts, in a
#     test file under tests/ or in Rust code under src/.
# Adding goldens, tests and assertions is always fine.
#
# Usage: tools/oracle-guard.sh [base]   (default origin/main; compares the
#        merge base of <base> and HEAD with HEAD)
#
# Exit 0: no oracle change, or no product change. Exit 1: both; the CI
# override is an approval on the "oracle" GitHub Environment. Exit 2: error.
set -euo pipefail

base="${1:-origin/main}"
merge_base="$(git merge-base "$base" HEAD)" || {
  echo "oracle-guard: no merge base between $base and HEAD" >&2
  exit 2
}

# Lines that assert: Rust assert macros and proptest's, Playwright expect(),
# shell checks that fail the run, and screenshot comparisons.
assertion='assert|expect[.(]|toHaveScreenshot|toMatchSnapshot|exit [1-9]|FAIL'

product=()
oracle=()
while IFS=$'\t' read -r status path; do
  case "$path" in src/* | ui/*) product+=("$path") ;; esac
  case "$status" in
    A) continue ;;
    D) kind="deleted" ;;
    *) kind="modified" ;;
  esac
  case "$path" in
    tests/golden/*) oracle+=("$path: existing golden $kind") ;;
    tests/*)
      if [ "$kind" != "modified" ]; then
        oracle+=("$path: existing test file $kind")
      fi
      ;;
  esac
done < <(git diff --no-renames --name-status "$merge_base" HEAD)

# Removed lines that asserted, in test files and Rust sources.
while IFS= read -r line; do
  oracle+=("$line")
done < <(git diff -U0 "$merge_base" HEAD -- 'tests/*' 'src/*.rs' ':!tests/golden/*' ':!tests/fixtures/*' |
  awk -v re="$assertion" '
    /^\+\+\+ / { next }
    /^--- / { file = substr($0, 5); sub(/^a\//, "", file); next }
    /^-/ { text = substr($0, 2); if (text ~ re) { gsub(/^[ \t]+/, "", text); print file ": removed or changed assertion: " substr(text, 1, 100) } }
  ')

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
