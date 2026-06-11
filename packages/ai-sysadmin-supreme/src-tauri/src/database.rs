use anyhow::Result;
use std::{fs, path::PathBuf};

pub struct Database {
 db_path: PathBuf,
}

impl Database {
 pub async fn new() -> Result<Self> {
 let db_path = PathBuf::from("data/database.db");

 fs::create_dir_all("data")?;

 Ok(Self { db_path })
 }
}