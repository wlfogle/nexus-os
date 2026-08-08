#!/usr/bin/env bash
# clone-all-repos.sh -- make the vault hold a real clone of every repo you own.
#
# For each repository on the account, one of three things happens:
#
#   absent   -> clone it
#   hollow   -> a folder exists holding only preserved artefacts (bundles,
#               patches, tarballs) and no working tree. The artefacts are moved
#               to _bundles/<name>/, the repo is cloned, and any commits that
#               exist on no remote are restored as refs/restored/*, followed by
#               the uncommitted diff and untracked files.
#   real     -> left alone; only `git fetch` runs so it is up to date.
#
#   clone-all-repos.sh            show what would happen
#   clone-all-repos.sh --apply    do it
#   clone-all-repos.sh --apply --no-archived   skip archived repos

set -Eeuo pipefail

VAULT="${LIBRARIAN_VAULT:-/media/loufogle/Data/Repos}"
BUNDLES="$VAULT/_bundles"
OWNER="${GITHUB_OWNER:-wlfogle}"
APPLY=0
SKIP_ARCHIVED=0

# Distros that were experiments on other bases and are not part of the current
# NexusOS line. Override with LIBRARIAN_EXCLUDE.
EXCLUDE="${LIBRARIAN_EXCLUDE:-garuda|blendos}"

for a in "$@"; do
    case "$a" in
        --apply) APPLY=1 ;;
        --no-archived) SKIP_ARCHIVED=1 ;;
        --all) EXCLUDE="" ;;
        *) echo "unknown argument: $a" >&2; exit 2 ;;
    esac
done

command -v gh >/dev/null || { echo "gh CLI is required" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "gh is not authenticated" >&2; exit 1; }

mkdir -p "$VAULT" "$BUNDLES"

cloned=0 restored=0 fetched=0 failed=0 skipped=0

# Move preserved artefacts out of a folder so a clone can land there.
park_artefacts() {
    local dir="$1" name="$2"
    local stash="$BUNDLES/$name"
    mkdir -p "$stash"
    local f
    for f in "$dir"/* "$dir"/.[!.]*; do
        [[ -e "$f" ]] || continue
        mv -f "$f" "$stash/" 2>/dev/null || true
    done
}

# Re-apply whatever was preserved for this repo.
reapply() {
    local dir="$1" name="$2"
    local stash="$BUNDLES/$name"
    [[ -d "$stash" ]] || return 0

    if [[ -f "$stash/local-refs.bundle" ]]; then
        if git -C "$dir" fetch --quiet "$stash/local-refs.bundle" \
               'refs/*:refs/restored/*' 2>/dev/null; then
            echo "      unpushed commits -> refs/restored/*"
        fi
    fi
    if [[ -f "$stash/stashes.bundle" ]]; then
        git -C "$dir" fetch --quiet "$stash/stashes.bundle" \
            'refs/*:refs/restored-stash/*' 2>/dev/null &&
            echo "      stashes -> refs/restored-stash/*"
    fi
    # Untracked first, so the tracked diff applies to a matching tree.
    if [[ -f "$stash/untracked.tar.gz" ]] &&
       [[ "$(stat -c%s "$stash/untracked.tar.gz")" -gt 100 ]]; then
        tar xzf "$stash/untracked.tar.gz" -C "$dir" 2>/dev/null &&
            echo "      untracked files restored"
    fi
    if [[ -f "$stash/dirty.patch" && -s "$stash/dirty.patch" ]]; then
        if git -C "$dir" apply --3way "$stash/dirty.patch" 2>/dev/null; then
            echo "      uncommitted changes re-applied"
        else
            echo "      !! dirty.patch did not apply; kept in _bundles"
        fi
    fi
}

echo "owner: $OWNER   vault: $VAULT"
(( APPLY )) || echo "DRY RUN - pass --apply to make changes"
echo

while IFS=$'\t' read -r name url archived; do
    [[ -z "$name" ]] && continue
    if [[ -n "$EXCLUDE" ]] && grep -qiE "$EXCLUDE" <<<"$name"; then
        printf 'EXCLUDE  %-38s (matches "%s")\n' "$name" "$EXCLUDE"
        skipped=$(( skipped + 1 )); continue
    fi
    if (( SKIP_ARCHIVED )) && [[ "$archived" == "true" ]]; then
        skipped=$(( skipped + 1 )); continue
    fi

    dir="$VAULT/$name"
    tag=""; [[ "$archived" == "true" ]] && tag=" [archived]"

    if [[ -e "$dir/.git" ]]; then
        printf 'REAL     %-38s%s\n' "$name" "$tag"
        if (( APPLY )); then
            GIT_TERMINAL_PROMPT=0 git -C "$dir" fetch --quiet --all --prune 2>/dev/null \
                && fetched=$(( fetched + 1 )) || true
        fi
        continue
    fi

    if [[ -d "$dir" ]]; then
        printf 'HOLLOW   %-38s%s -> park artefacts + clone\n' "$name" "$tag"
        if (( APPLY )); then
            park_artefacts "$dir" "$name"
            rmdir "$dir" 2>/dev/null || true
            if GIT_TERMINAL_PROMPT=0 git clone --quiet "$url" "$dir"; then
                reapply "$dir" "$name"
                restored=$(( restored + 1 ))
            else
                echo "      !! clone failed"
                failed=$(( failed + 1 ))
            fi
        fi
        continue
    fi

    printf 'CLONE    %-38s%s\n' "$name" "$tag"
    if (( APPLY )); then
        if GIT_TERMINAL_PROMPT=0 git clone --quiet "$url" "$dir"; then
            reapply "$dir" "$name"
            cloned=$(( cloned + 1 ))
        else
            echo "      !! clone failed"
            failed=$(( failed + 1 ))
        fi
    fi
done < <(gh repo list "$OWNER" --limit 300 \
            --json name,sshUrl,url,isArchived \
            --jq '.[] | [.name, .url, (.isArchived|tostring)] | @tsv')

echo
echo "cloned $cloned   restored $restored   fetched $fetched   failed $failed   skipped $skipped"
(( APPLY )) && echo "Preserved artefacts remain under $BUNDLES/<name>/"
