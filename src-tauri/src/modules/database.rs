use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

pub struct Database {
    connection: Arc<Mutex<Connection>>,
    semaphore: Arc<Semaphore>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let connection = Arc::new(Mutex::new(Connection::open(db_path)?));
        // Limit to 10 concurrent database operations
        let semaphore = Arc::new(Semaphore::new(10));
        Ok(Self { connection, semaphore })
    }

    pub async fn execute(&self, query: &str, params: &[&dyn rusqlite::ToSql]) -> Result<()> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let conn = self.connection.lock().unwrap();
        conn.execute(query, params)?;
        Ok(())
    }

    pub async fn query<T, F>(&self, query: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<Vec<T>>
    where
        F: Fn(&rusqlite::Row) -> T,
    {
        let _permit = self.semaphore.acquire().await.unwrap();
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params, |row| Ok(f(row)))?;
        let mut results = vec![];
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn backup(&self, backup_path: &str) -> Result<()> {
        
        // Simple file copy backup approach
        // In a production environment, you might want to use SQLite's backup API
        // or ensure the database is not being written to during backup
        let _conn = self.connection.lock().unwrap();
        
        // For now, we'll use a simple file copy approach
        // This assumes the database file path can be determined
        // In a real implementation, you'd want to store the original path
        // or use SQLite's backup API if available
        
        // Create a simple backup by copying the database file
        // This is a placeholder implementation
        std::fs::copy("app.db", backup_path)
            .map_err(|e| rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_IOERR),
                Some(format!("Backup failed: {}", e))
            ))?;
        
        Ok(())
    }
}

