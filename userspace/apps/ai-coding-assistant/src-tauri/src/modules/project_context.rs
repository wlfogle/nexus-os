use std::collections::HashMap;
use std::path::{Path, PathBuf};
use git2::{Repository, Error as GitError};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::error;

pub struct ProjectContext {
    repo_path: PathBuf,
    repo: Arc<RwLock<Repository>>,
    commits: Arc<RwLock<HashMap<String, String>>>, // map of commit hash to message
    _watcher: Option<RecommendedWatcher>,
    reload_tx: Option<mpsc::UnboundedSender<()>>,
}

impl ProjectContext {
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo_path = repo_path.as_ref().to_path_buf();
        let repo = Arc::new(RwLock::new(Repository::open(&repo_path)?));
        let commits = Arc::new(RwLock::new(Self::load_commits(&repo)?));
        let cache = Arc::new(RwLock::new(ProjectCache::new())));
        
        Ok(Self {
            repo_path,
            repo,
            commits,
            _watcher: None,
            reload_tx: None,
        })
    }

    fn load_commits(repo: &Arc<RwLock<Repository>>) -> Result<HashMap<String, String>, GitError> {
        let mut map = HashMap::new();
        let repo = repo.read().unwrap();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            map.insert(commit.id().to_string(), commit.summary().unwrap_or_default().to_string());
        }

        Ok(map)
    }

    pub fn list_commits(&self) -> HashMap<String, String> {
        self.commits.read().unwrap().clone()
    }
    
    pub async fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }

    pub fn file_change_tracker(repo_path: impl AsRef<Path>) -> Result<mpsc::UnboundedReceiver<()>, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let path_clone = repo_path.as_ref().to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(_) => {
                if let Err(e) = tx.send(()) {
                    error!("Failed to send reload signal: {}", e);
                }
            }
            Err(e) => error!("File watcher error: {:?}", e),
        }).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        watcher.watch(&path_clone, RecursiveMode::Recursive)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(rx)
    }
}

