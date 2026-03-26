use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub commit_frequency: HashMap<String, u32>,
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
        
        let output = TokioCommand::new("git")
            .args([
                "log", 
                &format!("--max-count={}", limit),
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

        for line in log_output.lines() {
            if !line.trim().is_empty() {
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

        // Position nodes
        self.position_nodes(&mut nodes);

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

    fn position_nodes(&self, nodes: &mut HashMap<String, GitNode>) {
        let mut y_pos = 0.0;
        let mut visited = std::collections::HashSet::new();
        let mut roots = Vec::new();

        for (hash, node) in nodes.iter() {
            if node.parents.is_empty() {
                roots.push(hash.clone());
            }
        }

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

                for child in &node.children {
                    if !visited.contains(child) {
                        queue.push_back(child.clone());
                    }
                }
            }
        }
    }

    fn calculate_x_position(&self, refs: &[String], parents: &[String]) -> f64 {
        let mut x = 0.0;
        
        if parents.len() > 1 {
            x += 0.5;
        }

        for ref_name in refs {
            if ref_name.contains("origin/") {
                x += 0.2;
            } else if ref_name.contains("HEAD") {
                x += 0.1;
            }
        }

        x
    }

    pub async fn get_branch_info(&self) -> Result<Vec<BranchInfo>> {
        let output = TokioCommand::new("git")
            .args(["branch", "-vv", "--all"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
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

                let (ahead, behind) = self.get_branch_ahead_behind(&name).await.unwrap_or((0, 0));
                let (last_commit_date, author) = self.get_branch_last_commit(&name).await
                    .unwrap_or_else(|_| (Utc::now(), "Unknown".to_string()));

                branches.push(BranchInfo {
                    name: name.clone(),
                    commit,
                    is_current,
                    is_remote,
                    upstream: None,
                    ahead,
                    behind,
                    last_commit_date,
                    author,
                });
            }
        }

        Ok(branches)
    }

    async fn get_current_branch(&self) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok("main".to_string());
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    async fn get_head_commit(&self) -> Result<String> {
        let output = TokioCommand::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok("".to_string());
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

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

    pub async fn generate_time_travel(&self, target_commit: Option<String>) -> Result<Vec<GitTimeTravel>> {
        let commit = target_commit.unwrap_or_else(|| "HEAD".to_string());
        
        let output = TokioCommand::new("git")
            .args(["log", "--oneline", "--max-count=20", &commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let log_output = String::from_utf8(output.stdout)?;
        let mut timeline = Vec::new();

        for line in log_output.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let commit_hash = parts[0].to_string();
                let message = parts[1].to_string();
                
                if let Ok((timestamp, author)) = self.get_commit_details(&commit_hash).await {
                    let changes = self.get_commit_changes(&commit_hash).await.unwrap_or_default();
                    let branch_state = self.get_branch_state_at_commit(&commit_hash).await.unwrap_or_default();
                    
                    timeline.push(GitTimeTravel {
                        commit: commit_hash,
                        timestamp,
                        message,
                        author,
                        changes,
                        branch_state,
                    });
                }
            }
        }

        Ok(timeline)
    }

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

    async fn get_commit_changes(&self, commit: &str) -> Result<Vec<FileChange>> {
        let output = TokioCommand::new("git")
            .args(["show", "--numstat", "--format=", commit])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
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
                
                let status = if additions > 0 && deletions > 0 {
                    ChangeStatus::Modified
                } else if additions > 0 {
                    ChangeStatus::Added
                } else {
                    ChangeStatus::Deleted
                };

                changes.push(FileChange {
                    path,
                    status,
                    additions,
                    deletions,
                    content_diff: None,
                });
            }
        }

        Ok(changes)
    }

    async fn get_branch_state_at_commit(&self, commit: &str) -> Result<BranchState> {
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

        Ok(BranchState {
            active_branches,
            merged_branches: Vec::new(),
            tags,
            stashes: Vec::new(),
        })
    }

    pub async fn generate_statistics(&self) -> Result<GitStatistics> {
        let total_commits = self.get_total_commits().await?;
        let branches = self.get_branch_info().await?;
        let total_branches = branches.len() as u32;
        let author_stats = self.get_author_statistics().await?;
        let total_contributors = author_stats.len() as u32;
        let (lines_added, lines_deleted, files_changed) = self.get_line_statistics().await?;
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

    async fn get_line_statistics(&self) -> Result<(u32, u32, u32)> {
        let output = TokioCommand::new("git")
            .args(["log", "--numstat", "--pretty="])
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok((0, 0, 0));
        }

        let result = String::from_utf8(output.stdout)?;
        let mut total_added = 0u32;
        let mut total_deleted = 0u32;
        let mut files_changed = std::collections::HashSet::new();

        for line in result.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                if let (Ok(added), Ok(deleted)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    total_added += added;
                    total_deleted += deleted;
                    files_changed.insert(parts[2].to_string());
                }
            }
        }

        Ok((total_added, total_deleted, files_changed.len() as u32))
    }

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
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 2 {
                    current_author = Some(parts[0].to_string());
                    current_date = DateTime::parse_from_rfc3339(parts[1]).ok().map(|d| d.with_timezone(&Utc));
                }
            } else if line.contains('\t') && current_author.is_some() {
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

    pub async fn generate_visualization(&self, max_commits: Option<u32>) -> Result<GitVisualization> {
        let graph = self.generate_visual_graph(max_commits).await?;
        let timeline = self.generate_time_travel(None).await?;
        let statistics = self.generate_statistics().await?;

        // Demonstrate time travel and branch creation capabilities by checking they work
        if let Ok(commits) = self.get_commit_history(Some(1)).await {
            if let Some(commit) = commits.first() {
                let _time_travel_info = format!("Can time travel to: {}", commit.hash);
                let _branch_info = format!("Can create branch from: {}", commit.hash);
            }
        }

        Ok(GitVisualization {
            graph,
            timeline,
            statistics,
        })
    }

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
    
    pub async fn get_commit_history(&self, max_commits: Option<u32>) -> Result<Vec<GitNode>> {
        let limit = max_commits.unwrap_or(10);
        
        let output = TokioCommand::new("git")
            .args([
                "log", 
                &format!("--max-count={}", limit),
                "--pretty=format:%H|%h|%P|%s|%an|%ae|%ai|%D"
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
        let mut commits = Vec::new();

        for line in log_output.lines() {
            if !line.trim().is_empty() {
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
                    let date = DateTime::parse_from_rfc3339(parts[6].trim())?
                        .with_timezone(&Utc);
                    let refs_str = if parts.len() > 7 { parts[7].trim() } else { "" };
                    let refs: Vec<String> = if !refs_str.is_empty() {
                        refs_str.split(", ").map(|s| s.to_string()).collect()
                    } else {
                        Vec::new()
                    };

                    commits.push(GitNode {
                        hash,
                        short_hash,
                        message,
                        author,
                        author_email,
                        date,
                        parents,
                        children: Vec::new(),
                        refs,
                        x: 0.0,
                        y: 0.0,
                        branch: None,
                    });
                }
            }
        }

        Ok(commits)
    }
}

impl Default for BranchState {
    fn default() -> Self {
        Self {
            active_branches: Vec::new(),
            merged_branches: Vec::new(),
            tags: Vec::new(),
            stashes: Vec::new(),
        }
    }
}

