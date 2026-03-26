use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use std::process::Command;
use std::process::Stdio;
use chrono::{DateTime, Utc};
use tokio::process::Command as TokioCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitNode {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub date: DateTime<Utc>,
    pub parents: Vec<String>,
    pub children: Vec<String>,
    pub refs: Vec<String>,
    pub x: f64,
    pub y: f64,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitEdge {
    pub from: String,
    pub to: String,
    pub branch: Option<String>,
    pub merge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitGraph {
    pub nodes: HashMap<String, GitNode>,
    pub edges: Vec<GitEdge>,
    pub branches: Vec<BranchInfo>,
    pub current_branch: String,
    pub head_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub commit: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit_date: DateTime<Utc>,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitTimeTravel {
    pub commit: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub author: String,
    pub changes: Vec<FileChange>,
    pub branch_state: BranchState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: ChangeStatus,
    pub additions: u32,
    pub deletions: u32,
    pub content_diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchState {
    pub active_branches: Vec<String>,
    pub merged_branches: Vec<String>,
    pub tags: Vec<String>,
    pub stashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitVisualization {
    pub graph: GitGraph,
    pub timeline: Vec<GitTimeTravel>,
    pub statistics: GitStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatistics {
    pub total_commits: u32,
    pub total_branches: u32,
    pub total_contributors: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_changed: u32,
    pub commit_frequency: HashMap<String, u32>, // date -> count
    pub author_stats: HashMap<String, AuthorStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorStats {
    pub commits: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub files_touched: u32,
    pub first_commit: DateTime<Utc>,
    pub last_commit: DateTime<Utc>,
}

pub struct GitAdvanced {
    pub repo_path: String,
}

impl GitAdvanced {
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
        }
    }

    /// Generate a complete visual git graph with positioned nodes
    pub async fn generate_visual_graph(&self, max_commits: Option<u32>) -> Result<GitGraph> {
        let limit = max_commits.unwrap_or(100);
        
        // Get commit history with formatting
        let output = TokioCommand::new("git")
            .args([
                "log", 
                &format!("--max-count={}", limit),
                "--graph",
                "--pretty=format:%H|%h|%P|%s|%an|%ae|%ai|%D",
                "--all"
            ])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Git log failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let log_output = String::from_utf8(output.stdout)?;
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        // Parse commits and create nodes
        for line in log_output.lines() {
            if line.contains('|') && !line.trim().starts_with('*') && !line.trim().starts_with('\\') {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 7 {
                    let hash = parts[0].trim().to_string();
                    let short_hash = parts[1].trim().to_string();
                    let parents: Vec<String> = if !parts[2].trim().is_empty() {
                        parts[2].trim().split_whitespace().map(|s| s.to_string()).collect()
                    } else {
                        Vec::new()
                    };
                    let message = parts[3].trim().to_string();
                    let author = parts[4].trim().to_string();
                    let author_email = parts[5].trim().to_string();
                    let date = DateTime::parse_from_rfc3339(parts[6].trim())?.with_timezone(&Utc);
                    let refs_str = if parts.len() > 7 { parts[7].trim() } else { "" };
                    let refs: Vec<String> = if !refs_str.is_empty() {
                        refs_str.split(", ").map(|s| s.to_string()).collect()
                    } else {
                        Vec::new()
                    };

                    let node = GitNode {
                        hash: hash.clone(),
                        short_hash,
                        message,
                        author,
                        author_email,
                        date,
                        parents: parents.clone(),
                        children: Vec::new(),
                        refs,
                        x: 0.0,
                        y: 0.0,
                        branch: None,
                    };

                    nodes.insert(hash.clone(), node);

                    // Create edges
                    for parent in parents {
                        edges.push(GitEdge {
                            from: parent,
                            to: hash.clone(),
                            branch: None,
                            merge: false,
                        });
                    }
                }
            }
        }

        // Fill in children
        for edge in &edges {
            if let Some(parent) = nodes.get_mut(&edge.from) {
                parent.children.push(edge.to.clone());
            }
        }

        // Position nodes using a simple layout algorithm
        self.position_nodes(&mut nodes);

        // Get branches
        let branches = self.get_branch_info().await?;
        let current_branch = self.get_current_branch().await?;
        let head_commit = self.get_head_commit().await?;

        Ok(GitGraph {
            nodes,
            edges,
            branches,
            current_branch,
            head_commit,
        })
    }

    /// Position nodes in the graph using a layered approach
    fn position_nodes(&self, nodes: &mut HashMap<String, GitNode>) {
        let mut y_pos = 0.0;
        let mut visited = std::collections::HashSet::new();
        let mut roots = Vec::new();

        // Find root nodes (commits with no parents)
        for (hash, node) in nodes.iter() {
            if node.parents.is_empty() {
                roots.push(hash.clone());
            }
        }

        // Topological sort and positioning
        let mut queue = std::collections::VecDeque::new();
        for root in roots {
            queue.push_back(root);
        }

        while let Some(hash) = queue.pop_front() {
            if !visited.insert(hash.clone()) {
                continue;
            }

            if let Some(node) = nodes.get_mut(&hash) {
                node.y = y_pos;
                node.x = self.calculate_x_position(&node.refs, &node.parents);
                y_pos += 1.0;

                // Add children to queue
                for child in &node.children {
                    if !visited.contains(child) {
                        queue.push_back(child.clone());
                    }
                }
            }
        }
    }

    /// Calculate X position based on branch information
    fn calculate_x_position(&self, refs: &[String], parents: &[String]) -> f64 {
        // Simple branch-based positioning
        let mut x = 0.0;
        
        // Check if this is a merge commit
        if parents.len() > 1 {
            x += 0.5;
        }

        // Position based on branch references
        for ref_name in refs {
            if ref_name.contains("origin/") {
                x += 0.2;
            } else if ref_name.contains("HEAD") {
                x += 0.1;
            }
        }

        x
    }

    /// Get detailed branch information
    pub async fn get_branch_info(&self) -> Result<Vec<BranchInfo>> {
        let output = TokioCommand::new("git")
            .args(["branch", "-vv", "--all"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Git branch failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let branch_output = String::from_utf8(output.stdout)?;
        let mut branches = Vec::new();

        for line in branch_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let is_current = line.starts_with('*');
            let is_remote = line.contains("remotes/");
            
            // Parse branch name and commit
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = if is_current {
                    parts[1].to_string()
                } else {
                    parts[0].to_string()
                };
                
                let commit = if is_current {
                    parts[2].to_string()
                } else {
                    parts[1].to_string()
                };

                // Get additional branch details
                let (ahead, behind) = self.get_branch_ahead_behind(&name).await.unwrap_or((0, 0));
                let (last_commit_date, author) = self.get_branch_last_commit(&name).await
                    .unwrap_or_else(|_| (Utc::now(), "Unknown".to_string()));
                
                let upstream = self.parse_upstream_from_line(line).await;

                branches.push(BranchInfo {
                    name: name.clone(),
                    commit,
                    is_current,
                    is_remote,
                    upstream,
                    ahead,
                    behind,
                    last_commit_date,
                    author,
                });
            }
        }

        Ok(branches)
    }

    /// Parse upstream information from git branch -vv output
    async fn parse_upstream_from_line(&self, line: &str) -> Option<String> {
        // Look for [origin/branch] pattern in the line
        if let Some(start) = line.find('[') {
            if let Some(end) = line[start..].find(']') {
                let upstream_part = &line[start + 1..start + end];
                if upstream_part.contains('/') {
                    return Some(upstream_part.split(':').next().unwrap_or(upstream_part).to_string());
                }
            }
        }
        None
    }

    /// Get current branch name
    async fn get_current_branch(&self) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Git branch --show-current failed"));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Get HEAD commit hash
    async fn get_head_commit(&self) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Git rev-parse HEAD failed"));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Get ahead/behind count for a branch
    async fn get_branch_ahead_behind(&self, branch: &str) -> Result<(u32, u32)> {
        let output = TokioCommand::new("git")
            .args(["rev-list", "--count", "--left-right", &format!("{}...HEAD", branch)])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok((0, 0));
        }

        let result = String::from_utf8(output.stdout)?;
        let parts: Vec<&str> = result.trim().split('\t').collect();
        if parts.len() == 2 {
            let ahead = parts[0].parse().unwrap_or(0);
            let behind = parts[1].parse().unwrap_or(0);
            Ok((ahead, behind))
        } else {
            Ok((0, 0))
        }
    }

    /// Get last commit date and author for a branch
    async fn get_branch_last_commit(&self, branch: &str) -> Result<(DateTime<Utc>, String)> {
        let output = TokioCommand::new("git")
            .args(["log", "-1", "--format=%ai|%an", branch])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get last commit for branch"));
        }

        let result = String::from_utf8(output.stdout)?;
        let parts: Vec<&str> = result.trim().split('|').collect();
        if parts.len() == 2 {
            let date = DateTime::parse_from_rfc3339(parts[0])?.with_timezone(&Utc);
            let author = parts[1].to_string();
            Ok((date, author))
        } else {
            Err(anyhow!("Invalid commit format"))
        }
    }

    /// Generate git time travel visualization
    pub async fn generate_time_travel(&self, target_commit: Option<String>) -> Result<Vec<GitTimeTravel>> {
        let commit = target_commit.unwrap_or_else(|| "HEAD".to_string());
        
        // Get commit history leading to target
        let output = TokioCommand::new("git")
            .args(["log", "--oneline", "--max-count=50", &commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get commit history"));
        }

        let log_output = String::from_utf8(output.stdout)?;
        let mut timeline = Vec::new();

        for line in log_output.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let commit_hash = parts[0].to_string();
                let message = parts[1].to_string();
                
                // Get detailed commit info
                let commit_details = self.get_commit_details(&commit_hash).await?;
                let changes = self.get_commit_changes(&commit_hash).await?;
                let branch_state = self.get_branch_state_at_commit(&commit_hash).await?;
                
                timeline.push(GitTimeTravel {
                    commit: commit_hash,
                    timestamp: commit_details.0,
                    message,
                    author: commit_details.1,
                    changes,
                    branch_state,
                });
            }
        }

        Ok(timeline)
    }

    /// Get detailed commit information
    async fn get_commit_details(&self, commit: &str) -> Result<(DateTime<Utc>, String)> {
        let output = TokioCommand::new("git")
            .args(["show", "-s", "--format=%ai|%an", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get commit details"));
        }

        let result = String::from_utf8(output.stdout)?;
        let parts: Vec<&str> = result.trim().split('|').collect();
        if parts.len() == 2 {
            let date = DateTime::parse_from_rfc3339(parts[0])?.with_timezone(&Utc);
            let author = parts[1].to_string();
            Ok((date, author))
        } else {
            Err(anyhow!("Invalid commit format"))
        }
    }

    /// Get changes for a specific commit
    async fn get_commit_changes(&self, commit: &str) -> Result<Vec<FileChange>> {
        let output = TokioCommand::new("git")
            .args(["show", "--numstat", "--format=", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get commit changes"));
        }

        let result = String::from_utf8(output.stdout)?;
        let mut changes = Vec::new();

        for line in result.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                let additions = parts[0].parse().unwrap_or(0);
                let deletions = parts[1].parse().unwrap_or(0);
                let path = parts[2].to_string();
                
                // Determine status based on stats
                let status = if additions > 0 && deletions > 0 {
                    ChangeStatus::Modified
                } else if additions > 0 {
                    ChangeStatus::Added
                } else {
                    ChangeStatus::Deleted
                };

                // Get diff content for small files
                let content_diff = if additions + deletions < 100 {
                    self.get_file_diff(commit, &path).await.ok()
                } else {
                    None
                };

                changes.push(FileChange {
                    path,
                    status,
                    additions,
                    deletions,
                    content_diff,
                });
            }
        }

        Ok(changes)
    }

    /// Get diff content for a specific file in a commit
    async fn get_file_diff(&self, commit: &str, path: &str) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["show", &format!("{}:{}", commit, path)])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get file diff"));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Get branch state at a specific commit
    async fn get_branch_state_at_commit(&self, commit: &str) -> Result<BranchState> {
        // Get branches that contain this commit
        let branches_output = TokioCommand::new("git")
            .args(["branch", "--contains", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let active_branches = if branches_output.status.success() {
            String::from_utf8(branches_output.stdout)?
                .lines()
                .map(|line| line.trim().trim_start_matches('*').trim().to_string())
                .filter(|name| !name.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        // Get merged branches at this point
        let merged_branches = self.get_merged_branches_at_commit(commit).await.unwrap_or_default();

        // Get tags at this commit
        let tags_output = TokioCommand::new("git")
            .args(["tag", "--points-at", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let tags = if tags_output.status.success() {
            String::from_utf8(tags_output.stdout)?
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|name| !name.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        let stashes = self.get_stashes().await.unwrap_or_default();

        Ok(BranchState {
            active_branches,
            merged_branches,
            tags,
            stashes,
        })
    }

    /// Get merged branches at a specific commit
    async fn get_merged_branches_at_commit(&self, commit: &str) -> Result<Vec<String>> {
        let output = TokioCommand::new("git")
            .args(["branch", "--merged", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let merged_branches = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").to_string())
            .filter(|branch| !branch.is_empty() && branch != "HEAD")
            .collect();

        Ok(merged_branches)
    }

    /// Get current stashes
    async fn get_stashes(&self) -> Result<Vec<String>> {
        let output = TokioCommand::new("git")
            .args(["stash", "list", "--oneline"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stashes = String::from_utf8(output.stdout)?
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|stash| !stash.is_empty())
            .collect();

        Ok(stashes)
    }

    /// Generate comprehensive git statistics
    pub async fn generate_statistics(&self) -> Result<GitStatistics> {
        // Get total commit count
        let total_commits = self.get_total_commits().await?;
        
        // Get total branch count
        let total_branches = self.get_total_branches().await?;
        
        // Get contributor statistics
        let author_stats = self.get_author_statistics().await?;
        let total_contributors = author_stats.len() as u32;
        
        // Calculate aggregate statistics
        let lines_added: u32 = author_stats.values().map(|stats| stats.lines_added).sum();
        let lines_deleted: u32 = author_stats.values().map(|stats| stats.lines_deleted).sum();
        let files_changed: u32 = author_stats.values().map(|stats| stats.files_touched).sum();
        
        // Get commit frequency data
        let commit_frequency = self.get_commit_frequency().await?;

        Ok(GitStatistics {
            total_commits,
            total_branches,
            total_contributors,
            lines_added,
            lines_deleted,
            files_changed,
            commit_frequency,
            author_stats,
        })
    }

    /// Get total number of commits
    async fn get_total_commits(&self) -> Result<u32> {
        let output = TokioCommand::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(0);
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(result.trim().parse().unwrap_or(0))
    }

    /// Get total number of branches
    async fn get_total_branches(&self) -> Result<u32> {
        let output = TokioCommand::new("git")
            .args(["branch", "-a"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(0);
        }

        let result = String::from_utf8(output.stdout)?;
        Ok(result.lines().count() as u32)
    }

    /// Get detailed author statistics
    async fn get_author_statistics(&self) -> Result<HashMap<String, AuthorStats>> {
        let output = TokioCommand::new("git")
            .args(["log", "--pretty=format:%an|%ai", "--numstat"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(HashMap::new());
        }

        let result = String::from_utf8(output.stdout)?;
        let mut author_stats: HashMap<String, AuthorStats> = HashMap::new();
        let mut current_author: Option<String> = None;
        let mut current_date: Option<DateTime<Utc>> = None;

        for line in result.lines() {
            if line.contains('|') && !line.contains('\t') {
                // Author line
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 2 {
                    current_author = Some(parts[0].to_string());
                    current_date = DateTime::parse_from_rfc3339(parts[1]).ok().map(|d| d.with_timezone(&Utc));
                }
            } else if line.contains('\t') && current_author.is_some() {
                // Stat line
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 3 {
                    let author = current_author.as_ref().unwrap();
                    let additions: u32 = parts[0].parse().unwrap_or(0);
                    let deletions: u32 = parts[1].parse().unwrap_or(0);
                    
                    let stats = author_stats.entry(author.clone()).or_insert_with(|| AuthorStats {
                        commits: 0,
                        lines_added: 0,
                        lines_deleted: 0,
                        files_touched: 0,
                        first_commit: Utc::now(),
                        last_commit: Utc::now(),
                    });
                    
                    stats.lines_added += additions;
                    stats.lines_deleted += deletions;
                    stats.files_touched += 1;
                    
                    if let Some(date) = current_date {
                        if date < stats.first_commit {
                            stats.first_commit = date;
                        }
                        if date > stats.last_commit {
                            stats.last_commit = date;
                        }
                    }
                }
            } else if line.trim().is_empty() && current_author.is_some() {
                // End of commit, increment commit count
                let author = current_author.as_ref().unwrap();
                if let Some(stats) = author_stats.get_mut(author) {
                    stats.commits += 1;
                }
                current_author = None;
                current_date = None;
            }
        }

        Ok(author_stats)
    }

    /// Get commit frequency by date
    async fn get_commit_frequency(&self) -> Result<HashMap<String, u32>> {
        let output = TokioCommand::new("git")
            .args(["log", "--pretty=format:%ai"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(HashMap::new());
        }

        let result = String::from_utf8(output.stdout)?;
        let mut frequency = HashMap::new();

        for line in result.lines() {
            if let Ok(date) = DateTime::parse_from_rfc3339(line.trim()) {
                let date_key = date.format("%Y-%m-%d").to_string();
                *frequency.entry(date_key).or_insert(0) += 1;
            }
        }

        Ok(frequency)
    }

    /// Generate complete visualization data
    pub async fn generate_visualization(&self) -> Result<GitVisualization> {
        let graph = self.generate_visual_graph(Some(200)).await?;
        let timeline = self.generate_time_travel(None).await?;
        let statistics = self.generate_statistics().await?;

        Ok(GitVisualization {
            graph,
            timeline,
            statistics,
        })
    }

    /// Time travel to a specific commit (checkout)
    pub async fn time_travel_to_commit(&self, commit: &str) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["checkout", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to checkout commit: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(format!("Time traveled to commit: {}", commit))
    }

    /// Create a new branch from current state
    pub async fn create_branch_from_commit(&self, branch_name: &str, commit: &str) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["checkout", "-b", branch_name, commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to create branch: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(format!("Created branch '{}' from commit {}", branch_name, commit))
    }
}

// Tauri commands for frontend integration
#[tauri::command]
pub async fn get_git_visualization(repo_path: String) -> Result<GitVisualization, String> {
    let git_advanced = GitAdvanced::new(&repo_path);
    git_advanced.generate_visualization().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_git_time_travel(repo_path: String, commit: Option<String>) -> Result<Vec<GitTimeTravel>, String> {
    let git_advanced = GitAdvanced::new(&repo_path);
    git_advanced.generate_time_travel(commit).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn time_travel_to_commit(repo_path: String, commit: String) -> Result<String, String> {
    let git_advanced = GitAdvanced::new(&repo_path);
    git_advanced.time_travel_to_commit(&commit).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_branch_from_commit(repo_path: String, branch_name: String, commit: String) -> Result<String, String> {
    let git_advanced = GitAdvanced::new(&repo_path);
    git_advanced.create_branch_from_commit(&branch_name, &commit).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_git_advanced_basic_functionality() {
        // Create a temporary git repository for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path().to_str().expect("Failed to get repo path");
        
        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");
            
        // Configure git
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user email");
            
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user name");
        
        // Create and commit a file
        fs::write(format!("{}/test.txt", repo_path), "Hello, world!").expect("Failed to write test file");
        std::process::Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file to git");
            
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit changes");

        let git_advanced = GitAdvanced::new(repo_path);
        
        // Test basic functionality
        let graph = git_advanced.generate_visual_graph(Some(10)).await;
        assert!(graph.is_ok());
        
        let stats = git_advanced.generate_statistics().await;
        assert!(stats.is_ok());
        
        let visualization = git_advanced.generate_visualization().await;
        assert!(visualization.is_ok());
    }
}
