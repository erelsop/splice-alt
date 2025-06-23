use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;
use crate::metadata::{SampleMetadata, BitwigCategory};

pub fn init_database(db_path: &Path) -> Result<()> {
    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create database directory {:?}: {}", parent, e))?;
    }
    
    let conn = Connection::open(db_path)?;
    
    // Create the samples table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL UNIQUE,
            pack_name TEXT NOT NULL,
            pack_uuid TEXT NOT NULL,
            filename TEXT NOT NULL,
            file_hash TEXT NOT NULL UNIQUE,
            bpm INTEGER,
            audio_key TEXT,
            chord_type TEXT,
            tags TEXT, -- JSON array of tags
            mapped_category TEXT NOT NULL,
            sample_type TEXT NOT NULL,
            duration INTEGER NOT NULL,
            file_size INTEGER NOT NULL,
            provider_name TEXT NOT NULL,
            date_downloaded TEXT NOT NULL,
            date_processed DATETIME DEFAULT CURRENT_TIMESTAMP,
            splice_url TEXT,
            preview_url TEXT,
            asset_uuid TEXT NOT NULL
        )",
        [],
    )?;
    
    // Create indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_file_hash ON samples(file_hash)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pack_name ON samples(pack_name)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_category ON samples(mapped_category)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tags ON samples(tags)",
        [],
    )?;
    
    println!("Database initialized at: {:?}", db_path);
    Ok(())
}

pub struct SampleRecord {
    pub id: Option<i64>,
    pub file_path: String,
    pub pack_name: String,
    pub pack_uuid: String,
    pub filename: String,
    pub file_hash: String,
    pub bpm: Option<u32>,
    pub audio_key: Option<String>,
    pub chord_type: Option<String>,
    pub tags: String, // JSON
    pub mapped_category: String,
    pub sample_type: String,
    pub duration: u32,
    pub file_size: u64,
    pub provider_name: String,
    pub date_downloaded: String,
    pub splice_url: Option<String>,
    pub preview_url: String,
    pub asset_uuid: String,
}

impl From<&SampleMetadata> for SampleRecord {
    fn from(metadata: &SampleMetadata) -> Self {
        Self {
            id: None,
            file_path: String::new(), // Will be set when we know the final path
            pack_name: metadata.sample_meta_data.pack.name.clone(),
            pack_uuid: metadata.sample_meta_data.pack.uuid.clone(),
            filename: metadata.sample_meta_data.filename.clone(),
            file_hash: metadata.sample_meta_data.file_hash.clone(),
            bpm: metadata.sample_meta_data.bpm,
            audio_key: metadata.sample_meta_data.audio_key.clone(),
            chord_type: metadata.sample_meta_data.chord_type.clone(),
            tags: serde_json::to_string(&metadata.sample_meta_data.tags).unwrap_or_default(),
            mapped_category: metadata.get_category().as_str().to_string(),
            sample_type: metadata.sample_meta_data.sample_type.clone(),
            duration: metadata.sample_meta_data.duration,
            file_size: metadata.sample.file_size,
            provider_name: metadata.sample_meta_data.provider_name.clone(),
            date_downloaded: metadata.sample_meta_data.purchased_at.clone(),
            splice_url: Some(metadata.sample.url.clone()),
            preview_url: metadata.sample_meta_data.preview_url.clone(),
            asset_uuid: metadata.sample_meta_data.asset_uuid.clone(),
        }
    }
}

pub fn insert_sample(db_path: &Path, record: SampleRecord) -> Result<i64> {
    let conn = Connection::open(db_path)?;
    
    // Check if sample already exists by hash
    if sample_exists_by_hash(&conn, &record.file_hash)? {
        return Err(anyhow::anyhow!("Sample with hash {} already exists", record.file_hash));
    }
    
    let _row_id = conn.execute(
        "INSERT INTO samples (
            file_path, pack_name, pack_uuid, filename, file_hash,
            bpm, audio_key, chord_type, tags, mapped_category,
            sample_type, duration, file_size, provider_name,
            date_downloaded, splice_url, preview_url, asset_uuid
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            record.file_path,
            record.pack_name,
            record.pack_uuid,
            record.filename,
            record.file_hash,
            record.bpm,
            record.audio_key,
            record.chord_type,
            record.tags,
            record.mapped_category,
            record.sample_type,
            record.duration,
            record.file_size,
            record.provider_name,
            record.date_downloaded,
            record.splice_url,
            record.preview_url,
            record.asset_uuid,
        ],
    )?;
    
    Ok(conn.last_insert_rowid())
}

pub fn sample_exists_by_hash(conn: &Connection, file_hash: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM samples WHERE file_hash = ?1 LIMIT 1")?;
    let exists = stmt.exists(params![file_hash])?;
    Ok(exists)
}

pub fn get_sample_by_hash(db_path: &Path, file_hash: &str) -> Result<Option<SampleRecord>> {
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, file_path, pack_name, pack_uuid, filename, file_hash,
                bpm, audio_key, chord_type, tags, mapped_category,
                sample_type, duration, file_size, provider_name,
                date_downloaded, splice_url, preview_url, asset_uuid
         FROM samples WHERE file_hash = ?1"
    )?;
    
    let sample_iter = stmt.query_map(params![file_hash], |row| {
        Ok(SampleRecord {
            id: Some(row.get(0)?),
            file_path: row.get(1)?,
            pack_name: row.get(2)?,
            pack_uuid: row.get(3)?,
            filename: row.get(4)?,
            file_hash: row.get(5)?,
            bpm: row.get(6)?,
            audio_key: row.get(7)?,
            chord_type: row.get(8)?,
            tags: row.get(9)?,
            mapped_category: row.get(10)?,
            sample_type: row.get(11)?,
            duration: row.get(12)?,
            file_size: row.get(13)?,
            provider_name: row.get(14)?,
            date_downloaded: row.get(15)?,
            splice_url: row.get(16)?,
            preview_url: row.get(17)?,
            asset_uuid: row.get(18)?,
        })
    })?;
    
    for sample in sample_iter {
        return Ok(Some(sample?));
    }
    
    Ok(None)
}

pub fn update_file_path(db_path: &Path, file_hash: &str, new_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        "UPDATE samples SET file_path = ?1 WHERE file_hash = ?2",
        params![new_path, file_hash],
    )?;
    
    Ok(())
}

pub fn get_samples_by_category(db_path: &Path, category: BitwigCategory) -> Result<Vec<SampleRecord>> {
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, file_path, pack_name, pack_uuid, filename, file_hash,
                bpm, audio_key, chord_type, tags, mapped_category,
                sample_type, duration, file_size, provider_name,
                date_downloaded, splice_url, preview_url, asset_uuid
         FROM samples WHERE mapped_category = ?1
         ORDER BY pack_name, filename"
    )?;
    
    let sample_iter = stmt.query_map(params![category.as_str()], |row| {
        Ok(SampleRecord {
            id: Some(row.get(0)?),
            file_path: row.get(1)?,
            pack_name: row.get(2)?,
            pack_uuid: row.get(3)?,
            filename: row.get(4)?,
            file_hash: row.get(5)?,
            bpm: row.get(6)?,
            audio_key: row.get(7)?,
            chord_type: row.get(8)?,
            tags: row.get(9)?,
            mapped_category: row.get(10)?,
            sample_type: row.get(11)?,
            duration: row.get(12)?,
            file_size: row.get(13)?,
            provider_name: row.get(14)?,
            date_downloaded: row.get(15)?,
            splice_url: row.get(16)?,
            preview_url: row.get(17)?,
            asset_uuid: row.get(18)?,
        })
    })?;
    
    let mut samples = Vec::new();
    for sample in sample_iter {
        samples.push(sample?);
    }
    
    Ok(samples)
} 