#!/usr/bin/env bash
# Bump Lantern's version and prepare a release pull request.
#
#   tools/bump-version.sh patch|minor|major|X.Y.Z [--push] [--branch NAME] [--no-branch]
#
# Updates the version in Cargo.toml and Cargo.lock, commits "Release vX.Y.Z" on
# a new branch (release/vX.Y.Z, started from origin/main), and with --push
# pushes it and opens a pull request (when the GitHub CLI `gh` is installed).
#
# Merging that pull request publishes the release: the Release workflow sees
# that the tag vX.Y.Z doesn't exist yet, builds all platforms, creates the tag
# and publishes the downloads. There is no separate tagging step.
#
#   --branch NAME   commit on NAME instead of release/vX.Y.Z
#   --no-branch     commit on the current branch (no fetch, no new branch)
#   --push          push the branch and open a pull request
#   --dry-run       print what would change, touch nothing
set -euo pipefail

usage() {
    sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'
    exit "${1:-1}"
}

die() {
    echo "bump-version: $*" >&2
    exit 1
}

# The whole script is one function so bash parses it before running: it
# switches branches, which may change this very file on disk.
main() {
    bump=""
    branch=""
    new_branch=1
    push=0
    dry_run=0
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h | --help) usage 0 ;;
            --push) push=1 ;;
            --dry-run) dry_run=1 ;;
            --no-branch) new_branch=0 ;;
            --branch)
                [[ $# -ge 2 ]] || die "--branch needs a name"
                branch="$2"
                shift
                ;;
            -*) die "unknown option $1" ;;
            *)
                [[ -z "$bump" ]] || die "only one version argument, please"
                bump="$1"
                ;;
        esac
        shift
    done
    [[ -n "$bump" ]] || usage

    cd "$(git rev-parse --show-toplevel)"
    [[ -f Cargo.toml ]] || die "run this inside the Lantern repository"

    # The [package] version is the first `version = "..."` line in Cargo.toml.
    current=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n1)
    [[ "$current" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]] ||
        die "can't read a plain X.Y.Z version from Cargo.toml (found '$current')"
    major=${BASH_REMATCH[1]} minor=${BASH_REMATCH[2]} patch=${BASH_REMATCH[3]}

    case "$bump" in
        patch) next="$major.$minor.$((patch + 1))" ;;
        minor) next="$major.$((minor + 1)).0" ;;
        major) next="$((major + 1)).0.0" ;;
        v[0-9]* | [0-9]*) next="${bump#v}" ;;
        *) die "expected patch, minor, major or a version like 1.2.3 (got '$bump')" ;;
    esac
    [[ "$next" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "'$next' is not a version like 1.2.3"

    # The new version must be greater than the current one.
    newest=$(printf '%s\n%s\n' "$current" "$next" | sort -t. -k1,1n -k2,2n -k3,3n | tail -n1)
    [[ "$next" != "$current" && "$newest" == "$next" ]] ||
        die "new version $next must be greater than the current $current"

    tag="v$next"
    if git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null 2>&1; then
        die "tag $tag already exists on GitHub. Pick another version, or delete that tag (and its release) first."
    fi

    [[ -n "$branch" ]] || branch="release/$tag"
    echo "Version: $current -> $next (tag $tag)"
    if [[ $dry_run == 1 ]]; then
        if [[ $new_branch == 1 ]]; then
            echo "Would commit on branch $branch (from origin/main)"
        else
            echo "Would commit on $(git branch --show-current)"
        fi
        exit 0
    fi

    [[ -z "$(git status --porcelain)" ]] || die "commit or stash your changes first"

    if [[ $new_branch == 1 ]]; then
        git fetch --quiet origin main
        git switch --quiet -C "$branch" origin/main
    fi

    # Edit only the [package] version line (the first one), portably (GNU and BSD).
    awk -v ver="$next" '
        !done && /^version = "/ { print "version = \"" ver "\""; done = 1; next }
        { print }
    ' Cargo.toml >Cargo.toml.tmp
    mv Cargo.toml.tmp Cargo.toml

    # Refresh Cargo.lock's entry for the lantern package only.
    if command -v cargo >/dev/null; then
        cargo update --workspace --offline --quiet 2>/dev/null || cargo update --workspace --quiet
    fi

    git add Cargo.toml Cargo.lock
    git commit --quiet -m "Release $tag"
    echo "Committed \"Release $tag\" on $(git branch --show-current)."

    if [[ $push == 0 ]]; then
        echo
        echo "Next: push this branch and open a pull request into main:"
        echo "  git push -u origin $(git branch --show-current)"
        echo "Merging it publishes $tag automatically."
        exit 0
    fi

    git push --quiet -u origin "$(git branch --show-current)"
    if command -v gh >/dev/null; then
        gh pr create --base main --title "Release $tag" --body "Bumps the version to $next.

    Merging this publishes **$tag**: the Release workflow builds Linux, macOS and
    Windows, creates the tag and attaches the downloads."
    else
        echo "Pushed. Open a pull request into main to publish $tag:"
        git remote get-url origin |
            sed -E 's#(git@github.com:|https://github.com/)([^/]+/[^.]+)(\.git)?#https://github.com/\2/compare/main...'"$(git branch --show-current)"'#'
    fi
}

main "$@"
