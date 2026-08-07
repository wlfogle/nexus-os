#!/usr/bin/env bash
# audit-local-only.sh -- what would be LOST if this repo were deleted and
# re-cloned from its remote?
#
# Reports, per repo:
#   no-remote        repo has no origin at all -> everything is local-only
#   unpushed         commits on any local branch not contained in any remote ref
#   local-branch     branch that exists nowhere on the remote
#   stash            stash entries (never pushed)
#   dirty            tracked files modified/staged but not committed
#   untracked        untracked, non-ignored files
#
# Exit 0 = safe to discard everything. Exit 1 = at least one repo has
# local-only content.

set -Eeuo pipefail

PLAN_TSV="${1:-$HOME/.local/state/librarian/repo-migration-plan.tsv}"
[[ -r "$PLAN_TSV" ]] || { echo "no plan file at $PLAN_TSV" >&2; exit 2; }

risky=0
safe_list=()
risk_list=()

while IFS=$'\t' read -r repo _dest _kb _dirty _unpushed _status; do
    [[ -z "$repo" || ! -d "$repo" ]] && continue
    git -C "$repo" rev-parse --git-dir >/dev/null 2>&1 || continue

    findings=()

    # --- remote present? -----------------------------------------------------
    if ! git -C "$repo" remote get-url origin >/dev/null 2>&1; then
        findings+=("no-remote: NOTHING is backed up")
    else
        # commits on any local branch not reachable from any remote ref
        while read -r count branch; do
            [[ -z "$branch" ]] && continue
            (( count > 0 )) && findings+=("unpushed: $count commit(s) on '$branch'")
        done < <(
            git -C "$repo" for-each-ref --format='%(refname:short)' refs/heads |
            while read -r br; do
                n=$(git -C "$repo" rev-list --count "$br" --not --remotes 2>/dev/null || echo 0)
                printf '%s %s\n' "$n" "$br"
            done
        )
        # branches with no counterpart anywhere on the remote
        while read -r br; do
            [[ -z "$br" ]] && continue
            if ! git -C "$repo" for-each-ref --format='%(refname:short)' refs/remotes |
                 sed 's|^[^/]*/||' | grep -qxF "$br"; then
                findings+=("local-branch: '$br' does not exist on remote")
            fi
        done < <(git -C "$repo" for-each-ref --format='%(refname:short)' refs/heads)
    fi

    # --- no commits at all? --------------------------------------------------
    if ! git -C "$repo" rev-parse --verify -q HEAD >/dev/null 2>&1; then
        findings+=("no-commits: working tree has never been committed")
    fi

    # --- stashes -------------------------------------------------------------
    n=$(git -C "$repo" stash list 2>/dev/null | wc -l || true)
    (( n > 0 )) && findings+=("stash: $n entr(ies)")

    # --- working tree --------------------------------------------------------
    n=$(git -C "$repo" diff --name-only HEAD 2>/dev/null | wc -l || true)
    (( n > 0 )) && findings+=("dirty: $n tracked file(s) modified")
    n=$(git -C "$repo" ls-files --others --exclude-standard 2>/dev/null | wc -l || true)
    (( n > 0 )) && findings+=("untracked: $n file(s)")

    if (( ${#findings[@]} == 0 )); then
        safe_list+=("$repo")
    else
        risky=1
        risk_list+=("$repo")
        printf '\n\033[1m%s\033[0m\n' "${repo/#$HOME\//~/}"
        printf '   - %s\n' "${findings[@]}"
    fi
done <"$PLAN_TSV"

printf '\n============================================================\n'
printf 'SAFE to delete & re-clone (%d):\n' "${#safe_list[@]}"
for r in "${safe_list[@]:-}"; do [[ -n "$r" ]] && printf '   %s\n' "${r/#$HOME\//~/}"; done
printf '\nHAS LOCAL-ONLY CONTENT (%d) -- must be preserved or pushed first\n' "${#risk_list[@]}"
exit $risky
