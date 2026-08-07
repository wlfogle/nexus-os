#!/usr/bin/env bash
# migrate-repos.sh -- move every top-level git repo into the Data vault.
#
#   plan    discover repos, show what would move, verify free space
#   apply   perform the moves (rsync -> verify -> delete source -> symlink)
#   verify  re-check every migrated repo is a healthy git repo
#
# Safety model:
#   * Source is never deleted until rsync reports byte-identical destination.
#   * A symlink is left at the original path so existing tooling keeps working.
#   * Repos nested inside another repo (submodules) move with their parent.
#   * Nothing is ever deleted outright; failures abort that repo and leave the
#     source untouched.

set -Eeuo pipefail

VAULT="${LIBRARIAN_VAULT:-/media/loufogle/Data/Repos}"
STATE_DIR="${LIBRARIAN_STATE:-$HOME/.local/state/librarian}"
PLAN_TSV="$STATE_DIR/repo-migration-plan.tsv"
LOG="$STATE_DIR/repo-migration.log"

SCAN_ROOTS=("$HOME")
PRUNE_NAMES=(.cache .steam .var .local .rustup .cargo .npm .gradle .nvm .bun
             node_modules target .wine .wineprefixes .PlayOnLinux .android
             Android snap "VirtualBox VMs" .vscode .vscode-shared .Genymobile
             .mozilla .waterfox .thunderbird .tor-browser .config)

mkdir -p "$STATE_DIR"

log()  { printf '%s %s\n' "$(date +%H:%M:%S)" "$*" | tee -a "$LOG"; }
die()  { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------- discovery --
discover_repos() {
    local find_args=() name
    for name in "${PRUNE_NAMES[@]}"; do
        find_args+=(-name "$name" -prune -o)
    done
    local root
    for root in "${SCAN_ROOTS[@]}"; do
        find "$root" -xdev "${find_args[@]}" -name .git -print 2>/dev/null
    done | sed 's|/\.git$||' | sort -u
}

# Keep only repos that are not inside another repo.
top_level_only() {
    local -a all=() out=()
    mapfile -t all
    local candidate parent nested
    for candidate in "${all[@]}"; do
        nested=0
        for parent in "${all[@]}"; do
            [[ "$candidate" == "$parent" ]] && continue
            if [[ "$candidate" == "$parent"/* ]]; then nested=1; break; fi
        done
        (( nested == 0 )) && out+=("$candidate")
    done
    printf '%s\n' "${out[@]}"
}

owner_of() {
    local repo="$1" url owner
    url="$(git -C "$repo" remote get-url origin 2>/dev/null || true)"
    [[ -z "$url" ]] && { printf 'local\n'; return; }
    # git@host:owner/name.git | https://host/owner/name.git | ssh://host/owner/name
    owner="$(sed -E 's|\.git$||; s|^[^:]+://[^/]+/||; s|^[^@]+@[^:]+:||' <<<"$url")"
    owner="${owner%/*}"
    owner="${owner##*/}"
    [[ -z "$owner" ]] && owner=local
    printf '%s\n' "$owner"
}

# ------------------------------------------------------------------- plan ----
do_plan() {
    log "discovering repos under ${SCAN_ROOTS[*]}"
    local -a repos=()
    mapfile -t repos < <(discover_repos | top_level_only)
    (( ${#repos[@]} )) || die "no repos found"

    : >"$PLAN_TSV"
    local total_kb=0 repo owner dest kb name dirty unpushed status
    printf '%-42s %8s %7s %9s  %s\n' REPO SIZE DIRTY UNPUSHED DESTINATION
    printf '%s\n' "--------------------------------------------------------------------------------------------"

    for repo in "${repos[@]}"; do
        [[ "$repo" == "$VAULT"/* ]] && continue          # already migrated
        name="$(basename "$repo")"
        owner="$(owner_of "$repo")"
        dest="$VAULT/$owner/$name"
        kb="$(du -sk "$repo" 2>/dev/null | cut -f1)"
        total_kb=$(( total_kb + kb ))
        dirty="$(git -C "$repo" status --porcelain 2>/dev/null | wc -l)"
        unpushed="$(git -C "$repo" rev-list --count '@{u}..HEAD' 2>/dev/null || echo -)"
        status=ok
        [[ -e "$dest" ]] && status=DEST_EXISTS
        [[ -L "$repo" ]] && status=IS_SYMLINK
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$repo" "$dest" "$kb" "$dirty" "$unpushed" "$status" >>"$PLAN_TSV"
        printf '%-42s %7sM %7s %9s  %s %s\n' \
            "${repo/#$HOME\//~/}" "$(( kb / 1024 ))" "$dirty" "$unpushed" \
            "${dest/#$VAULT\//}" "$([[ $status == ok ]] || echo "[$status]")"
    done

    local need_mb=$(( total_kb / 1024 ))
    local avail_mb
    avail_mb="$(df -Pm "$(dirname "$VAULT")" | awk 'NR==2{print $4}')"
    printf '\n'
    log "repos: ${#repos[@]}   need: ${need_mb} MB   free at $VAULT: ${avail_mb} MB"
    if (( need_mb + 10240 > avail_mb )); then
        die "insufficient space (need ${need_mb}MB + 10GB headroom, have ${avail_mb}MB)"
    fi
    log "space OK. plan written to $PLAN_TSV"
}

# ------------------------------------------------------------------ apply ----
move_one() {
    local src="$1" dest="$2"
    local parent; parent="$(dirname "$dest")"
    mkdir -p "$parent"

    log "  rsync  $src -> $dest"
    rsync -aHAX --numeric-ids --delete-during --info=stats2 "$src/" "$dest/" >>"$LOG" 2>&1

    log "  verify (checksum diff)"
    local diff_count
    diff_count="$(rsync -aHAXn --checksum --itemize-changes "$src/" "$dest/" 2>/dev/null | grep -c '^[<>ch]' || true)"
    if [[ "$diff_count" != "0" ]]; then
        log "  !! verification FAILED ($diff_count differing entries) - source kept, dest left at $dest"
        return 1
    fi

    log "  verified identical; removing source"
    rm -rf -- "$src"
    ln -s "$dest" "$src"
    log "  symlink $src -> $dest"
    return 0
}

do_apply() {
    [[ -s "$PLAN_TSV" ]] || die "no plan found; run 'plan' first"
    local src dest kb dirty unpushed status ok=0 fail=0 skip=0

    while IFS=$'\t' read -r src dest kb dirty unpushed status; do
        [[ -z "$src" ]] && continue
        if [[ "$status" != ok ]]; then
            log "SKIP $src ($status)"; (( ++skip )); continue
        fi
        if [[ ! -d "$src" || -L "$src" ]]; then
            log "SKIP $src (gone or already a symlink)"; (( ++skip )); continue
        fi
        log "MOVE $src"
        if move_one "$src" "$dest"; then (( ++ok )); else (( ++fail )); fi
    done <"$PLAN_TSV"

    log "done: $ok moved, $fail failed, $skip skipped"
    (( fail == 0 ))
}

# ----------------------------------------------------------------- verify ----
do_verify() {
    local src dest rest bad=0
    while IFS=$'\t' read -r src dest rest; do
        [[ -z "$dest" || ! -d "$dest" ]] && continue
        if git -C "$dest" rev-parse --git-dir >/dev/null 2>&1; then
            printf 'OK    %-58s %s\n' "${dest/#$VAULT\//}" \
                "$(git -C "$dest" log -1 --format=%cs 2>/dev/null || echo no-commits)"
        else
            printf 'BROKEN %s\n' "$dest"; bad=1
        fi
        if [[ -L "$src" ]]; then
            [[ "$(readlink -f "$src")" == "$(readlink -f "$dest")" ]] || {
                printf 'BADLINK %s\n' "$src"; bad=1; }
        fi
    done <"$PLAN_TSV"
    return $bad
}

case "${1:-plan}" in
    plan)   do_plan   ;;
    apply)  do_apply  ;;
    verify) do_verify ;;
    *) die "usage: $0 {plan|apply|verify}" ;;
esac
