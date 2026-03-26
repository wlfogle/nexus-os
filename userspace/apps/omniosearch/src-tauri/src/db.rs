use std::path::Path;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use rusqlite::{Connection, params, Row};
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn, error};
use tantivy::{
    Index, IndexWriter, Document, Term,
    schema::{Schema, TEXT, STORED, INDEXED, STRING},
    query::QueryParser,
    collector::TopDocs,
    directory::MmapDirectory,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub file_type: String,
    pub mime_type: String,
    pub is_directory: bool,
    pub permissions: String,
    pub checksum: Option<String>,
    pub indexed_at: DateTime<Utc>,
    pub content_extracted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub total_files: u64,
    pub indexed_files: u64,
    pub pending_files: u64,
    pub failed_files: u64,
    pub last_update: DateTime<Utc>,
    pub indexing_speed: f64, // files per second
    pub index_size_mb: f64,
}

pub struct Database {
    sqlite_conn: Connection,
    tantivy_index: Index,
    tantivy_writer: IndexWriter,
    schema: Schema,
}

impl Database {
    pub async fn new(db_path: &str) -> Result<Self> {
        info!("💾 Initializing database at: {}", db_path);

        // Ensure database directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize SQLite connection
        let sqlite_conn = Connection::open(db_path)
            .context("Failed to open SQLite database")?;
        
        // Enable SQLite FTS5 extension
        sqlite_conn.execute_batch(r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=10000;
            PRAGMA temp_store=memory;
            PRAGMA mmap_size=268435456;
        "#)?;

        // Create tables
        Self::create_tables(&sqlite_conn)?;

        // Initialize Tantivy index
        let (tantivy_index, tantivy_writer, schema) = Self::initialize_tantivy_index(db_path)?;

        let db = Self {
            sqlite_conn,
            tantivy_index,
            tantivy_writer,
            schema,
        };

        info!("✅ Database initialized successfully");
        Ok(db)
    }

    fn create_tables(conn: &Connection) -> Result<()> {
        debug!("📋 Creating database tables...");

        // Main files table
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                size INTEGER NOT NULL,
                modified INTEGER NOT NULL,
                created INTEGER NOT NULL,
                file_type TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                is_directory BOOLEAN NOT NULL,
                permissions TEXT NOT NULL,
                checksum TEXT,
                indexed_at INTEGER NOT NULL,
                content_extracted BOOLEAN NOT NULL DEFAULT FALSE
            )
        "#, [])?;

        // FTS5 virtual table for fast text search
        conn.execute(r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
                path,
                name,
                content=files,
                content_rowid=rowid
            );
        "#, [])?;

        // Triggers to keep FTS table in sync
        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS files_fts_insert AFTER INSERT ON files
            BEGIN
                INSERT INTO files_fts(rowid, path, name) VALUES (new.rowid, new.path, new.name);
            END;
        "#, [])?;

        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS files_fts_update AFTER UPDATE ON files
            BEGIN
                UPDATE files_fts SET path = new.path, name = new.name WHERE rowid = new.rowid;
            END;
        "#, [])?;

        conn.execute(r#"
            CREATE TRIGGER IF NOT EXISTS files_fts_delete AFTER DELETE ON files
            BEGIN
                DELETE FROM files_fts WHERE rowid = old.rowid;
            END;
        "#, [])?;

        // File content table for extracted text
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS file_content (
                file_id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                extracted_at INTEGER NOT NULL,
                FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE
            )
        "#, [])?;

        // Indexing queue for background processing
        conn.execute(r#"
            CREATE TABLE IF NOT EXISTS indexing_queue (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                last_error TEXT
            )
        "#, [])?;

        // Indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_size ON files(size)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_type ON files(file_type)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_queue_priority ON indexing_queue(priority DESC, created_at)", [])?;

        debug!("✅ Database tables created");
        Ok(())
    }

    fn initialize_tantivy_index(db_path: &str) -> Result<(Index, IndexWriter, Schema)> {
        debug!("🔍 Initializing Tantivy full-text search index...");

        let index_dir = Path::new(db_path).parent().unwrap().join("tantivy_index");
        std::fs::create_dir_all(&index_dir)?;

        let mut schema_builder = Schema::builder();
        
        // Add fields to the schema
        let id = schema_builder.add_text_field("id", STRING | STORED);
        let path = schema_builder.add_text_field("path", TEXT | STORED);
        let name = schema_builder.add_text_field("name", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT);
        let file_type = schema_builder.add_text_field("file_type", STRING | INDEXED);
        let size = schema_builder.add_u64_field("size", INDEXED | STORED);
        let modified = schema_builder.add_date_field("modified", INDEXED | STORED);
        
        let schema = schema_builder.build();

        // Open or create index
        let index = if index_dir.exists() && index_dir.read_dir()?.next().is_some() {
            Index::open_in_dir(&index_dir)?
        } else {
            let mmap_directory = MmapDirectory::open(&index_dir)?;
            Index::create_in_dir(&index_dir, schema.clone())?
        };

        // Create index writer
        let index_writer = index.writer(50_000_000)?; // 50MB heap

        debug!("✅ Tantivy index initialized");
        Ok((index, index_writer, schema))
    }

    pub async fn insert_file(&self, file_entry: &FileEntry) -> Result<()> {
        debug!("💾 Inserting file: {}", file_entry.path);

        // Insert into SQLite
        self.sqlite_conn.execute(r#"
            INSERT OR REPLACE INTO files (
                id, path, name, size, modified, created, file_type, mime_type,
                is_directory, permissions, checksum, indexed_at, content_extracted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#, params![
            file_entry.id,
            file_entry.path,
            file_entry.name,
            file_entry.size as i64,
            file_entry.modified.timestamp(),
            file_entry.created.timestamp(),
            file_entry.file_type,
            file_entry.mime_type,
            file_entry.is_directory,
            file_entry.permissions,
            file_entry.checksum,
            file_entry.indexed_at.timestamp(),
            file_entry.content_extracted,
        ])?;

        // Add to Tantivy index
        let mut doc = Document::new();
        doc.add_text(self.schema.get_field("id").unwrap(), &file_entry.id);
        doc.add_text(self.schema.get_field("path").unwrap(), &file_entry.path);
        doc.add_text(self.schema.get_field("name").unwrap(), &file_entry.name);
        doc.add_text(self.schema.get_field("file_type").unwrap(), &file_entry.file_type);
        doc.add_u64(self.schema.get_field("size").unwrap(), file_entry.size);
        doc.add_date(
            self.schema.get_field("modified").unwrap(), 
            tantivy::DateTime::from_timestamp_secs(file_entry.modified.timestamp())
        );

        self.tantivy_writer.add_document(doc)?;

        Ok(())
    }

    pub async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<FileEntry>> {
        debug!("🔍 FTS search: {} (limit: {})", query, limit);

        let mut stmt = self.sqlite_conn.prepare(r#"
            SELECT f.id, f.path, f.name, f.size, f.modified, f.created, f.file_type,
                   f.mime_type, f.is_directory, f.permissions, f.checksum, f.indexed_at,
                   f.content_extracted
            FROM files f
            JOIN files_fts fts ON f.rowid = fts.rowid
            WHERE files_fts MATCH ?1
            ORDER BY bm25(files_fts) 
            LIMIT ?2
        "#)?;

        let file_iter = stmt.query_map(params![query, limit], |row| {
            Ok(Self::row_to_file_entry(row)?)
        })?;

        let mut results = Vec::new();
        for file_result in file_iter {
            results.push(file_result?);
        }

        debug!("🔍 FTS found {} results", results.len());
        Ok(results)
    }

    pub async fn tantivy_search(&self, query: &str, limit: usize) -> Result<Vec<FileEntry>> {
        debug!("🔍 Tantivy search: {} (limit: {})", query, limit);

        let reader = self.tantivy_index.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &self.tantivy_index, 
            vec![
                self.schema.get_field("name").unwrap(),
                self.schema.get_field("path").unwrap(),
                self.schema.get_field("content").unwrap(),
            ]
        );

        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            if let Some(id_value) = retrieved_doc.get_first(self.schema.get_field("id").unwrap()) {
                if let Some(id) = id_value.as_text() {
                    if let Ok(file_entry) = self.get_file_by_id(id).await {
                        results.push(file_entry);
                    }
                }
            }
        }

        debug!("🔍 Tantivy found {} results", results.len());
        Ok(results)
    }

    pub async fn get_file_by_id(&self, id: &str) -> Result<FileEntry> {
        let mut stmt = self.sqlite_conn.prepare(r#"
            SELECT id, path, name, size, modified, created, file_type, mime_type,
                   is_directory, permissions, checksum, indexed_at, content_extracted
            FROM files WHERE id = ?1
        "#)?;

        let file_entry = stmt.query_row(params![id], |row| {
            Ok(Self::row_to_file_entry(row)?)
        })?;

        Ok(file_entry)
    }

    pub async fn get_file_by_path(&self, path: &str) -> Result<Option<FileEntry>> {
        let mut stmt = self.sqlite_conn.prepare(r#"
            SELECT id, path, name, size, modified, created, file_type, mime_type,
                   is_directory, permissions, checksum, indexed_at, content_extracted
            FROM files WHERE path = ?1
        "#)?;

        match stmt.query_row(params![path], |row| {
            Ok(Self::row_to_file_entry(row)?)
        }) {
            Ok(file_entry) => Ok(Some(file_entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn delete_file(&self, path: &str) -> Result<()> {
        debug!("🗑️ Deleting file from index: {}", path);

        // Get file ID before deletion
        if let Some(file_entry) = self.get_file_by_path(path).await? {
            // Delete from Tantivy
            let id_term = Term::from_field_text(self.schema.get_field("id").unwrap(), &file_entry.id);
            self.tantivy_writer.delete_term(id_term);
        }

        // Delete from SQLite (triggers will handle FTS cleanup)
        self.sqlite_conn.execute("DELETE FROM files WHERE path = ?1", params![path])?;

        Ok(())
    }

    pub async fn add_file_content(&self, file_id: &str, content: &str) -> Result<()> {
        debug!("📄 Adding content for file: {}", file_id);

        // Store in SQLite
        self.sqlite_conn.execute(r#"
            INSERT OR REPLACE INTO file_content (file_id, content, extracted_at)
            VALUES (?1, ?2, ?3)
        "#, params![file_id, content, Utc::now().timestamp()])?;

        // Update Tantivy document with content
        if let Ok(file_entry) = self.get_file_by_id(file_id).await {
            let mut doc = Document::new();
            doc.add_text(self.schema.get_field("id").unwrap(), &file_entry.id);
            doc.add_text(self.schema.get_field("path").unwrap(), &file_entry.path);
            doc.add_text(self.schema.get_field("name").unwrap(), &file_entry.name);
            doc.add_text(self.schema.get_field("content").unwrap(), content);
            doc.add_text(self.schema.get_field("file_type").unwrap(), &file_entry.file_type);
            doc.add_u64(self.schema.get_field("size").unwrap(), file_entry.size);
            doc.add_date(
                self.schema.get_field("modified").unwrap(),
                tantivy::DateTime::from_timestamp_secs(file_entry.modified.timestamp())
            );

            // Delete old document and add new one
            let id_term = Term::from_field_text(self.schema.get_field("id").unwrap(), &file_entry.id);
            self.tantivy_writer.delete_term(id_term);
            self.tantivy_writer.add_document(doc)?;
        }

        // Mark content as extracted
        self.sqlite_conn.execute(r#"
            UPDATE files SET content_extracted = TRUE WHERE id = ?1
        "#, params![file_id])?;

        Ok(())
    }

    pub async fn get_indexing_status(&self) -> Result<IndexStatus> {
        let mut stmt = self.sqlite_conn.prepare(r#"
            SELECT 
                COUNT(*) as total_files,
                SUM(CASE WHEN content_extracted THEN 1 ELSE 0 END) as indexed_files,
                (SELECT COUNT(*) FROM indexing_queue) as pending_files,
                SUM(CASE WHEN checksum IS NULL THEN 1 ELSE 0 END) as failed_files,
                MAX(indexed_at) as last_update
            FROM files
        "#)?;

        let row = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)? as u64,
                row.get::<_, i64>(1)? as u64,
                row.get::<_, i64>(2)? as u64,
                row.get::<_, i64>(3)? as u64,
                row.get::<_, i64>(4)?,
            ))
        })?;

        let (total_files, indexed_files, pending_files, failed_files, last_update_ts) = row;

        // Calculate index size
        let index_size_mb = self.calculate_index_size().await?;

        Ok(IndexStatus {
            total_files,
            indexed_files,
            pending_files,
            failed_files,
            last_update: DateTime::from_timestamp(last_update_ts, 0).unwrap_or_else(|| Utc::now()),
            indexing_speed: 0.0, // Would be calculated from actual indexing metrics
            index_size_mb,
        })
    }

    async fn calculate_index_size(&self) -> Result<f64> {
        let db_path = ""; // Would get from config
        let mut total_size = 0u64;

        if let Ok(metadata) = std::fs::metadata(db_path) {
            total_size += metadata.len();
        }

        // Add Tantivy index size
        let index_dir = Path::new(db_path).parent().unwrap().join("tantivy_index");
        if index_dir.exists() {
            for entry in std::fs::read_dir(&index_dir)? {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size as f64 / (1024.0 * 1024.0))
    }

    pub async fn commit(&self) -> Result<()> {
        self.tantivy_writer.commit()?;
        Ok(())
    }

    fn row_to_file_entry(row: &Row) -> Result<FileEntry, rusqlite::Error> {
        Ok(FileEntry {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            size: row.get::<_, i64>(3)? as u64,
            modified: DateTime::from_timestamp(row.get::<_, i64>(4)?, 0).unwrap_or_else(|| Utc::now()),
            created: DateTime::from_timestamp(row.get::<_, i64>(5)?, 0).unwrap_or_else(|| Utc::now()),
            file_type: row.get(6)?,
            mime_type: row.get(7)?,
            is_directory: row.get(8)?,
            permissions: row.get(9)?,
            checksum: row.get(10)?,
            indexed_at: DateTime::from_timestamp(row.get::<_, i64>(11)?, 0).unwrap_or_else(|| Utc::now()),
            content_extracted: row.get(12)?,
        })
    }
}

// Implement Clone for Arc<RwLock<Database>> usage
unsafe impl Send for Database {}
unsafe impl Sync for Database {}
