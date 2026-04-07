use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};

pub fn get_status(path: &str) -> Result<String> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    
    let statuses = repo.statuses(Some(&mut opts))
        .context("Failed to get git status")?;
    
    let mut result = String::new();
    
    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("<unknown>");
        
        let status_str = match status {
            s if s.is_wt_new() => "??",
            s if s.is_wt_modified() => " M",
            s if s.is_wt_deleted() => " D",
            s if s.is_wt_renamed() => " R",
            s if s.is_index_new() => "A ",
            s if s.is_index_modified() => "M ",
            s if s.is_index_deleted() => "D ",
            s if s.is_index_renamed() => "R ",
            _ => "  ",
        };
        
        result.push_str(&format!("{} {}\n", status_str, path));
    }
    
    Ok(result)
}

pub fn get_diff(path: &str) -> Result<String> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_workdir(Some(&head), None)?;
    
    let mut result = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        match line.origin() {
            '+' | '-' | ' ' => result.push(line.origin()),
            _ => {}
        }
        result.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
        true
    })?;
    
    Ok(result)
}

pub fn get_branch_name(path: &str) -> Result<String> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let head = repo.head()?;
    if let Some(name) = head.shorthand() {
        Ok(name.to_string())
    } else {
        Ok("HEAD".to_string())
    }
}

pub fn is_repo(path: &str) -> bool {
    Repository::open(path).is_ok()
}

pub fn get_recent_commits(path: &str, limit: usize) -> Result<Vec<String>> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    
    let mut commits = Vec::new();
    
    for (i, oid) in revwalk.enumerate() {
        if i >= limit {
            break;
        }
        
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        
        let message = commit.message().unwrap_or("<invalid utf8>");
        let author = commit.author();
        let time = commit.time();
        let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        
        let commit_info = format!(
            "{} - {} ({}) [{}]",
            &oid.to_string()[..8],
            message.lines().next().unwrap_or(""),
            author.name().unwrap_or("Unknown"),
            timestamp
        );
        
        commits.push(commit_info);
    }
    
    Ok(commits)
}

pub fn get_remote_url(path: &str) -> Result<Option<String>> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    // Try to get the origin remote first
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            return Ok(Some(url.to_string()));
        }
    }
    
    // If no origin, try to get the first remote
    let remotes = repo.remotes()?;
    if let Some(remote_name) = remotes.get(0) {
        if let Ok(remote) = repo.find_remote(remote_name) {
            if let Some(url) = remote.url() {
                return Ok(Some(url.to_string()));
            }
        }
    }
    
    Ok(None)
}

/// Get ahead/behind commit counts compared to upstream branch
pub fn get_ahead_behind_count(path: &str) -> Result<(usize, usize)> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    // Get current branch
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => {
            // No commits yet or detached HEAD
            return Ok((0, 0));
        }
    };
    
    // Get local branch reference
    let local_oid = head.target().context("Failed to get HEAD target")?;
    
    // Find the upstream branch
    let upstream_oid = match find_upstream_branch(&repo, &head)? {
        Some(oid) => oid,
        None => {
            // No upstream configured
            return Ok((0, 0));
        }
    };
    
    // Calculate ahead/behind using git2's graph functionality
    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((ahead, behind)) => Ok((ahead, behind)),
        Err(_) => {
            // If graph calculation fails, try alternative method
            calculate_ahead_behind_alternative(&repo, local_oid, upstream_oid)
        }
    }
}

/// Find the upstream branch for the current branch
fn find_upstream_branch(repo: &Repository, head: &git2::Reference) -> Result<Option<git2::Oid>> {
    // Method 1: Try to get configured upstream
    if let Some(branch_name) = head.shorthand() {
        let branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
        if let Ok(upstream) = branch.upstream() {
            if let Some(target) = upstream.get().target() {
                return Ok(Some(target));
            }
        }
        
        // Method 2: Try common upstream patterns
        let upstream_patterns = [
            format!("origin/{}", branch_name),
            format!("upstream/{}", branch_name),
            format!("refs/remotes/origin/{}", branch_name),
        ];
        
        for pattern in &upstream_patterns {
            if let Ok(reference) = repo.find_reference(pattern) {
                if let Some(target) = reference.target() {
                    return Ok(Some(target));
                }
            }
        }
    }
    
    // Method 3: Try to find any remote tracking branch
    let remote_branches = repo.branches(Some(git2::BranchType::Remote))?;
    
    for branch_result in remote_branches {
        if let Ok((branch, _)) = branch_result {
            if let Some(name) = branch.name()? {
                // Look for branches that might be tracking the current branch
                if let Some(local_name) = head.shorthand() {
                    if name.ends_with(&format!("/{}", local_name)) {
                        if let Some(target) = branch.get().target() {
                            return Ok(Some(target));
                        }
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Alternative method to calculate ahead/behind when graph method fails
fn calculate_ahead_behind_alternative(
    repo: &Repository, 
    local_oid: git2::Oid, 
    upstream_oid: git2::Oid
) -> Result<(usize, usize)> {
    // Find merge base
    let merge_base = match repo.merge_base(local_oid, upstream_oid) {
        Ok(base) => base,
        Err(_) => {
            // No common history - this could happen with unrelated branches
            return Ok((0, 0));
        }
    };
    
    // Count commits ahead (local commits not in upstream)
    let ahead = count_commits_between(repo, merge_base, local_oid)?;
    
    // Count commits behind (upstream commits not in local)
    let behind = count_commits_between(repo, merge_base, upstream_oid)?;
    
    Ok((ahead, behind))
}

/// Count commits between two OIDs
fn count_commits_between(repo: &Repository, from: git2::Oid, to: git2::Oid) -> Result<usize> {
    if from == to {
        return Ok(0);
    }
    
    let mut revwalk = repo.revwalk()?;
    revwalk.push(to)?;
    revwalk.hide(from)?;
    
    let mut count = 0;
    for _commit in revwalk {
        count += 1;
    }
    
    Ok(count)
}

/// Get detailed branch information
pub fn get_branch_info(path: &str) -> Result<BranchInfo> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let head = repo.head()?;
    let branch_name = head.shorthand().unwrap_or("HEAD").to_string();
    
    let (ahead, behind) = get_ahead_behind_count(path).unwrap_or((0, 0));
    
    // Get last commit info
    let last_commit = if let Ok(commit) = head.peel_to_commit() {
        Some(LastCommit {
            hash: commit.id().to_string()[..8].to_string(),
            message: commit.message().unwrap_or("<invalid utf8>").lines().next().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            timestamp: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        })
    } else {
        None
    };
    
    // Check if working directory is clean
    let is_clean = {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts))?;
        statuses.is_empty()
    };
    
    // Get upstream branch name
    let upstream_branch = if let Some(branch_name) = head.shorthand() {
        let branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
        match branch.upstream() {
            Ok(upstream) => {
                match upstream.name() {
                    Ok(Some(name)) => Some(name.to_string()),
                    _ => None,
                }
            },
            Err(_) => None,
        }
    } else {
        None
    };
    
    Ok(BranchInfo {
        name: branch_name,
        upstream_branch,
        ahead_count: ahead,
        behind_count: behind,
        is_clean,
        last_commit,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub upstream_branch: Option<String>,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub is_clean: bool,
    pub last_commit: Option<LastCommit>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LastCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

/// Get all branches (local and remote)
pub fn get_all_branches(path: &str) -> Result<BranchList> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let mut local_branches = Vec::new();
    let mut remote_branches = Vec::new();
    
    // Get local branches
    let local_iter = repo.branches(Some(git2::BranchType::Local))?;
    for branch_result in local_iter {
        if let Ok((branch, _)) = branch_result {
            if let Some(name) = branch.name()? {
                let is_head = branch.is_head();
                local_branches.push(BranchItem {
                    name: name.to_string(),
                    is_current: is_head,
                    last_commit_hash: branch.get().target()
                        .map(|oid| oid.to_string()[..8].to_string()),
                });
            }
        }
    }
    
    // Get remote branches
    let remote_iter = repo.branches(Some(git2::BranchType::Remote))?;
    for branch_result in remote_iter {
        if let Ok((branch, _)) = branch_result {
            if let Some(name) = branch.name()? {
                remote_branches.push(BranchItem {
                    name: name.to_string(),
                    is_current: false,
                    last_commit_hash: branch.get().target()
                        .map(|oid| oid.to_string()[..8].to_string()),
                });
            }
        }
    }
    
    Ok(BranchList {
        local_branches,
        remote_branches,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BranchList {
    pub local_branches: Vec<BranchItem>,
    pub remote_branches: Vec<BranchItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BranchItem {
    pub name: String,
    pub is_current: bool,
    pub last_commit_hash: Option<String>,
}

/// Get git stash list
pub fn get_stash_list(path: &str) -> Result<Vec<StashEntry>> {
    let mut repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let mut stashes = Vec::new();
    
    repo.stash_foreach(|index, message, oid| {
        stashes.push(StashEntry {
            index,
            message: if message.is_empty() {
                "<empty>".to_string()
            } else {
                message.to_string()
            },
            oid: oid.to_string()[..8].to_string(),
        });
        true
    })?;
    
    Ok(stashes)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub oid: String,
}

/// Get file changes for specific commit
pub fn get_commit_changes(path: &str, commit_hash: &str) -> Result<Vec<FileChange>> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let oid = git2::Oid::from_str(commit_hash)
        .context("Invalid commit hash")?;
    let commit = repo.find_commit(oid)?;
    
    let mut changes = Vec::new();
    
    // Get the tree for this commit
    let tree = commit.tree()?;
    
    // Get parent commit tree (if any)
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };
    
    // Create diff
    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&tree),
        None
    )?;
    
    // Collect file changes
    diff.foreach(
        &mut |delta, _progress| {
            let old_file = delta.old_file();
            let new_file = delta.new_file();
            
            let change_type = match delta.status() {
                git2::Delta::Added => "added",
                git2::Delta::Deleted => "deleted",
                git2::Delta::Modified => "modified",
                git2::Delta::Renamed => "renamed",
                git2::Delta::Copied => "copied",
                _ => "unknown",
            };
            
            let file_path = new_file.path()
                .or_else(|| old_file.path())
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            
            changes.push(FileChange {
                path: file_path,
                change_type: change_type.to_string(),
                additions: 0, // Would need line-by-line diff for accurate count
                deletions: 0, // Would need line-by-line diff for accurate count
            });
            
            true
        },
        None,
        None,
        None,
    )?;
    
    Ok(changes)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: String,
    pub additions: usize,
    pub deletions: usize,
}

/// Get detailed repository statistics
pub fn get_repository_stats(path: &str) -> Result<RepositoryStats> {
    let repo = Repository::open(path)
        .context("Failed to open git repository")?;
    
    let mut total_commits = 0;
    let mut contributors = std::collections::HashSet::new();
    let total_additions = 0;
    let total_deletions = 0;
    
    // Walk through all commits
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    
    for commit_oid in revwalk {
        let commit_oid = commit_oid?;
        let commit = repo.find_commit(commit_oid)?;
        
        total_commits += 1;
        
        // Add contributor
        if let Some(author_name) = commit.author().name() {
            contributors.insert(author_name.to_string());
        };
    }
    
    // Get current status
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opts))?;
    
    let mut modified_files = 0;
    let mut untracked_files = 0;
    let mut staged_files = 0;
    
    for entry in statuses.iter() {
        let status = entry.status();
        
        if status.is_wt_modified() || status.is_wt_deleted() || status.is_wt_renamed() {
            modified_files += 1;
        }
        
        if status.is_wt_new() {
            untracked_files += 1;
        }
        
        if status.is_index_modified() || status.is_index_new() || 
           status.is_index_deleted() || status.is_index_renamed() {
            staged_files += 1;
        }
    }
    
    let (ahead, behind) = get_ahead_behind_count(path).unwrap_or((0, 0));
    
    Ok(RepositoryStats {
        total_commits,
        contributors: contributors.len(),
        total_additions,
        total_deletions,
        modified_files,
        untracked_files,
        staged_files,
        ahead_count: ahead,
        behind_count: behind,
        branch_count: count_branches(&repo)?,
        tag_count: count_tags(&repo)?,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositoryStats {
    pub total_commits: usize,
    pub contributors: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub modified_files: usize,
    pub untracked_files: usize,
    pub staged_files: usize,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub branch_count: usize,
    pub tag_count: usize,
}

fn count_branches(repo: &Repository) -> Result<usize> {
    let mut count = 0;
    let branches = repo.branches(None)?;
    
    for _branch in branches {
        count += 1;
    }
    
    Ok(count)
}

fn count_tags(repo: &Repository) -> Result<usize> {
    let mut count = 0;
    
    repo.tag_foreach(|_oid, name| {
        if std::str::from_utf8(name).is_ok() {
            count += 1;
        }
        true
    })?;
    
    Ok(count)
}
