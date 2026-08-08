#!/usr/bin/env bash
# preserve-local-only.sh -- make every local repo disposable.
#
#   push      push branches to remotes we actually own (write access probed)
#   archive   bundle everything that could NOT be pushed
#   all       push, then archive
#
# After this runs, no repo working tree holds unique data: anything that lives
# only on this machine is either on the remote or inside the archive dir.
#
# Archive layout, per repo:
#   <archive>/<owner>__<name>/local-refs.bundle   unpushed commits + local branches
#   <archive>/<owner>__<name>/stashes.bundle      stash commits (as refs)
#   <archive>/<owner>__<name>/dirty.patch         modified tracked files
#   <archive>/<owner>__<name>/untracked.tar.zst   untracked, non-ignored files
#   <archive>/<owner>__<name>/MANIFEST.txt        provenance + restore recipe

set -Eeuo pipefail

VAULT="${LIBRARIAN_VAULT:-/media/loufogle/Data/Repos}"
ARCHIVE="${LIBRARIAN_ARCHIVE:-/media/loufogle/Data/Repos/_local-only-archive}"
STATE_DIR="${LIBRARIAN_STATE:-$HOME/.local/state/librarian}"
PLAN_TSV="$STATE_DIR/repo-migration-plan.tsv"
LOG="$STATE_DIR/preserve.log"

mkdir -p "$ARCHIVE" "$STATE_DIR"
log() { printf '%s %s\n' "$(date +%H:%M:%S)" "$*" | tee -a "$LOG"; }

repo_slug() {
    local repo="$1" url owner
    url="$(git -C "$repo" remote get-url origin 2>/dev/null || true)"
    if [[ -z "$url" ]]; then owner=local; else
        owner="$(sed -E 's|\.git$||; s|^[^:]+://[^/]+/||; s|^[^@]+@[^:]+:||' <<<"$url")"
        owner="${owner%/*}"; owner="${owner##*/}"; [[ -z "$owner" ]] && owner=local
    fi
    printf '%s__%s\n' "$owner" "$(basename "$repo")"
}

# Can we push to origin? Probe without transferring anything.
can_push() {
    local repo="$1"
    git -C "$repo" remote get-url origin >/dev/null 2>&1 || return 1
    GIT_TERMINAL_PROMPT=0 timeout 45 git -C "$repo" push --dry-run origin \
        --porcelain HEAD >/dev/null 2>&1
}

repos_from_plan() {
    cut -f1 "$PLAN_TSV" | while read -r r; do
        [[ -n "$r" && -d "$r" ]] && git -C "$r" rev-parse --git-dir >/dev/null 2>&1 && echo "$r"
    done
}

# ------------------------------------------------------------------- push ----
do_push() {
    local repo br pushed
    while read -r repo; do
        git -C "$repo" remote get-url origin >/dev/null 2>&1 || {
            log "SKIP $(basename "$repo") (no remote)"; continue; }

        # only bother if something is actually unpushed
        local n
        n="$(git -C "$repo" rev-list --count --all --not --remotes 2>/dev/null || echo 0)"
        (( n == 0 )) && { log "ok   $(basename "$repo") (nothing unpushed)"; continue; }

        if ! can_push "$repo"; then
            log "NOPUSH $(basename "$repo") ($n local commit(s)) - no write access, will archive"
            continue
        fi

        pushed=0
        while read -r br; do
            [[ -z "$br" ]] && continue
            local ahead
            ahead="$(git -C "$repo" rev-list --count "$br" --not --remotes 2>/dev/null || echo 0)"
            (( ahead == 0 )) && continue

            # Only push branches that are already tracked upstream, or are the
            # repo's primary branch. Abandoned local branches (experiment /
            # orchestrator / worktree leftovers) are preserved in the bundle
            # instead of being published as remote clutter.
            if ! git -C "$repo" rev-parse --abbrev-ref --symbolic-full-name \
                    "$br@{upstream}" >/dev/null 2>&1; then
                case "$br" in
                    main|master) ;;
                    *) log "     $(basename "$repo") $br (+$ahead) -> bundle only"; continue ;;
                esac
            fi

            if GIT_TERMINAL_PROMPT=0 git -C "$repo" push -u origin "$br":"$br" >>"$LOG" 2>&1; then
                log "PUSH $(basename "$repo") $br (+$ahead)"
                pushed=$(( pushed + 1 ))
            else
                log "FAIL $(basename "$repo") $br - archiving instead"
            fi
        done < <(git -C "$repo" for-each-ref --format='%(refname:short)' refs/heads)
        (( pushed == 0 )) && log "     $(basename "$repo") nothing pushed"
    done < <(repos_from_plan)
}

# ---------------------------------------------------------------- archive ----
do_archive() {
    local repo slug out n
    while read -r repo; do
        slug="$(repo_slug "$repo")"
        out="$ARCHIVE/$slug"

        local -a made=()
        mkdir -p "$out"

        # 1. commits that exist on no remote (after the push pass)
        n="$(git -C "$repo" rev-list --count --all --not --remotes 2>/dev/null || echo 0)"
        if (( n > 0 )); then
            if git -C "$repo" bundle create "$out/local-refs.bundle" \
                    --all --not --remotes >>"$LOG" 2>&1; then
                made+=("local-refs.bundle ($n commit(s) on no remote)")
            fi
        elif ! git -C "$repo" remote get-url origin >/dev/null 2>&1; then
            # no remote at all -> bundle everything we can
            if git -C "$repo" rev-parse --verify -q HEAD >/dev/null 2>&1; then
                git -C "$repo" bundle create "$out/local-refs.bundle" --all >>"$LOG" 2>&1 \
                    && made+=("local-refs.bundle (entire history, no remote)")
            fi
        fi

        # 2. stashes
        n="$(git -C "$repo" stash list 2>/dev/null | wc -l || true)"
        if (( n > 0 )); then
            local -a stash_refs=()
            local i
            for (( i = 0; i < n; i++ )); do stash_refs+=("stash@{$i}"); done
            git -C "$repo" bundle create "$out/stashes.bundle" \
                "${stash_refs[@]}" >>"$LOG" 2>&1 && made+=("stashes.bundle ($n)")
            git -C "$repo" stash list >"$out/stashes.txt" 2>/dev/null || true
        fi

        # 3. modified tracked files
        if git -C "$repo" rev-parse --verify -q HEAD >/dev/null 2>&1; then
            if ! git -C "$repo" diff --quiet HEAD 2>/dev/null; then
                git -C "$repo" diff --binary HEAD >"$out/dirty.patch" 2>/dev/null || true
                [[ -s "$out/dirty.patch" ]] && made+=("dirty.patch ($(git -C "$repo" diff --name-only HEAD | wc -l) file(s))")
            fi
        fi

        # 4. untracked, non-ignored files
        #
        # `-C` must precede `-T` so it applies while the file list is read. With
        # it placed afterwards tar prints "has no effect" and writes an EMPTY
        # archive -- which is precisely how popos-setup's only file was lost on
        # 2026-08-07. The warning was in the log and went unnoticed, so the
        # archive is now verified to hold entries before it is trusted, and the
        # empty file is deleted rather than left looking like a valid backup.
        n="$(git -C "$repo" ls-files --others --exclude-standard 2>/dev/null | wc -l || true)"
        if (( n > 0 )); then
            if git -C "$repo" ls-files --others --exclude-standard -z 2>/dev/null |
               tar -czf "$out/untracked.tar.gz" -C "$repo" --null -T - 2>>"$LOG"; then
                local got
                got="$(tar tzf "$out/untracked.tar.gz" 2>/dev/null | grep -c . || true)"
                if (( got > 0 )); then
                    made+=("untracked.tar.gz ($got of $n file(s))")
                    (( got < n )) &&
                        log "     WARNING $slug: archived $got of $n untracked files"
                else
                    log "     ERROR $slug: untracked archive is EMPTY ($n expected) -- NOT safe to delete this repo"
                    rm -f "$out/untracked.tar.gz"
                fi
            fi
        fi

        if (( ${#made[@]} == 0 )); then
            rmdir "$out" 2>/dev/null || true
            continue
        fi

        {
            printf 'repo        : %s\n' "$repo"
            printf 'origin      : %s\n' "$(git -C "$repo" remote get-url origin 2>/dev/null || echo '(none)')"
            printf 'head        : %s\n' "$(git -C "$repo" rev-parse HEAD 2>/dev/null || echo '(no commits)')"
            printf 'branch      : %s\n' "$(git -C "$repo" rev-parse --abbrev-ref HEAD 2>/dev/null || echo '-')"
            printf 'archived    : %s\n' "$(date -Is)"
            printf '\ncontents:\n'
            printf '  - %s\n' "${made[@]}"
            printf '\nrestore:\n'
            printf '  git clone <origin> <dir> && cd <dir>\n'
            printf '  git bundle unbundle %s/local-refs.bundle   # then git checkout <branch>\n' "$out"
            printf '  git apply %s/dirty.patch\n' "$out"
            printf '  tar xzf %s/untracked.tar.gz\n' "$out"
        } >"$out/MANIFEST.txt"

        log "ARCHIVE $slug: ${made[*]}"
    done < <(repos_from_plan)

    log "archive size: $(du -sh "$ARCHIVE" | cut -f1) at $ARCHIVE"
}

case "${1:-all}" in
    push)    do_push ;;
    archive) do_archive ;;
    all)     do_push; do_archive ;;
    *) echo "usage: $0 {push|archive|all}" >&2; exit 2 ;;
esac
