#!/usr/bin/env bash
# restore-repos.sh -- turn bundle-only placeholder folders into real repos.
#
# After the 2026-08-07 consolidation, some entries under the vault contained
# only preserved artefacts (local-refs.bundle, dirty.patch, untracked.tar.gz,
# stashes.bundle) and no working tree. A folder that looks like a repo but is
# not is worse than no folder at all, so this materialises each one:
#
#   1. clone from the origin recorded in MANIFEST.txt
#   2. unbundle any commits that exist on no remote, as refs/restored/*
#   3. re-apply the uncommitted diff
#   4. restore untracked files
#   5. move the artefacts into _bundles/<name>/ so the working tree is clean
#
# Repos with no remote at all are initialised locally from their preserved
# contents instead of cloned.
#
#   restore-repos.sh          report what would happen
#   restore-repos.sh --apply  do it

set -Eeuo pipefail

VAULT="${LIBRARIAN_VAULT:-/media/loufogle/Data/Repos}"
BUNDLES="$VAULT/_bundles"
APPLY=0
[[ "${1:-}" == "--apply" ]] && APPLY=1

say() { printf '%s\n' "$*"; }
run() { if (( APPLY )); then "$@"; else printf '   would: %s\n' "$*"; fi; }

# A folder is a placeholder if it has no .git but does hold preserved artefacts.
is_placeholder() {
    local d="$1"
    [[ -e "$d/.git" ]] && return 1
    compgen -G "$d/*.bundle" >/dev/null && return 0
    compgen -G "$d/*.patch"  >/dev/null && return 0
    compgen -G "$d/*.tar.gz" >/dev/null && return 0
    [[ -f "$d/MANIFEST.txt" ]] && return 0
    return 1
}

# Some placeholders were created before the manifest was written, so their
# origin has to come from somewhere. These were recovered from the pre-migration
# audit; without them the folder would be re-created empty, which is the exact
# hollow-folder problem this script exists to remove.
known_origin() {
    case "$1" in
        cockpit-package-manager)
            printf 'https://github.com/hatlabs/cockpit-package-manager.git' ;;
        nexus-terminal)
            # Archived upstream: readable, not writable.
            printf 'https://github.com/wlfogle/nexus-terminal' ;;
        *) printf '' ;;
    esac
}

origin_of() {
    local d="$1" url=""
    [[ -f "$d/MANIFEST.txt" ]] &&
        url="$(grep -m1 '^origin' "$d/MANIFEST.txt" | sed 's/^origin *: *//' | tr -d '[:space:]')"
    [[ "$url" == "(none)" ]] && url=""
    [[ -z "$url" ]] && url="$(known_origin "$(basename "$d")")"
    printf '%s' "$url"
}

restore_one() {
    local dir="$1"
    local name; name="$(basename "$dir")"
    local url;  url="$(origin_of "$dir")"
    local stash="$BUNDLES/$name"

    say ""
    say "== $name"
    say "   origin: ${url:-<none recorded>}"

    # Park the artefacts outside the working tree first so a clone can land here
    # and so they never show up as untracked noise afterwards.
    run mkdir -p "$stash"
    local f
    for f in "$dir"/*; do
        [[ -e "$f" ]] || continue
        run mv -f "$f" "$stash/"
    done

    if [[ -n "$url" ]]; then
        say "   cloning..."
        if (( APPLY )); then
            rmdir "$dir" 2>/dev/null || true
            if ! GIT_TERMINAL_PROMPT=0 git clone --quiet "$url" "$dir"; then
                say "   !! clone failed (remote gone or private) - initialising empty instead"
                mkdir -p "$dir"
                git -C "$dir" init --quiet
                git -C "$dir" remote add origin "$url"
            fi
        else
            say "   would: git clone $url $dir"
        fi
    else
        say "   no remote: initialising a local repo"
        run mkdir -p "$dir"
        if (( APPLY )); then git -C "$dir" init --quiet; fi
    fi

    # Commits that existed on no remote, kept reachable under refs/restored/.
    if [[ -f "$stash/local-refs.bundle" ]]; then
        say "   restoring unpushed commits from local-refs.bundle"
        if (( APPLY )); then
            if git -C "$dir" fetch --quiet "$stash/local-refs.bundle" \
                   'refs/*:refs/restored/*' 2>/dev/null; then
                say "   -> available under refs/restored/* (git branch -a)"
            else
                say "   !! bundle did not apply to this clone"
            fi
        fi
    fi
    if [[ -f "$stash/stashes.bundle" ]]; then
        say "   restoring stash commits from stashes.bundle"
        if (( APPLY )); then
            git -C "$dir" fetch --quiet "$stash/stashes.bundle" \
                'refs/*:refs/restored-stash/*' 2>/dev/null ||
                say "   !! stash bundle did not apply"
        fi
    fi

    # Untracked files first, then the tracked diff, so the patch applies to a
    # tree that looks like the one it was taken from.
    if [[ -f "$stash/untracked.tar.gz" ]]; then
        if [[ -s "$stash/untracked.tar.gz" ]] &&
           [[ "$(stat -c%s "$stash/untracked.tar.gz")" -gt 100 ]]; then
            say "   restoring untracked files"
            run tar xzf "$stash/untracked.tar.gz" -C "$dir"
        else
            say "   untracked.tar.gz is empty - nothing to restore"
        fi
    fi
    if [[ -f "$stash/dirty.patch" && -s "$stash/dirty.patch" ]]; then
        say "   re-applying uncommitted changes"
        if (( APPLY )); then
            if ! git -C "$dir" apply --3way "$stash/dirty.patch" 2>/dev/null; then
                say "   !! patch did not apply cleanly; kept at $stash/dirty.patch"
            fi
        fi
    fi

    if (( APPLY )); then
        local head
        head="$(git -C "$dir" log -1 --format='%h %s' 2>/dev/null || echo 'no commits')"
        say "   HEAD: $head"
    fi
}

main() {
    (( APPLY )) || say "DRY RUN - pass --apply to make changes"
    mkdir -p "$BUNDLES"

    local found=0 d
    for d in "$VAULT"/*/; do
        [[ "$(basename "$d")" == "_bundles" ]] && continue
        if is_placeholder "${d%/}"; then
            found=$(( found + 1 ))
            restore_one "${d%/}"
        fi
    done

    say ""
    if (( found == 0 )); then
        say "No placeholder folders found - every entry in the vault is a real repo."
    else
        say "$found placeholder(s) processed."
        (( APPLY )) && say "Preserved artefacts now live under $BUNDLES/<name>/."
    fi
}

main
