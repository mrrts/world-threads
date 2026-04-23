pub mod schema;
pub mod queries;

use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Database {
    /// Wrapped in Arc so background tasks (e.g. relational_stance
    /// refresh) can clone the inner handle and run after the
    /// originating Tauri command has returned, without us having to
    /// reopen the SQLite file.
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        // Backup the database before doing anything
        if path.exists() {
            Self::backup_database(path);
        }

        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }

        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        schema::run_migrations(&conn)?;
        Ok(Database { conn: Arc::new(Mutex::new(conn)) })
    }

    /// Create a rolling backup of the database. Keeps the 5 most recent backups.
    pub fn backup_database(path: &Path) {
        let backup_dir = path.parent().unwrap_or(Path::new(".")).join("backups");
        if std::fs::create_dir_all(&backup_dir).is_err() {
            log::warn!("Failed to create backup directory");
            return;
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let db_name = path.file_name().unwrap_or_default().to_string_lossy();
        let backup_name = format!("{}_{}.bak", db_name, timestamp);
        let backup_path = backup_dir.join(&backup_name);

        match std::fs::copy(path, &backup_path) {
            Ok(bytes) => {
                log::info!("Database backup created: {} ({} bytes)", backup_name, bytes);

                // Also copy WAL file if it exists
                let wal_path = path.with_extension("db-wal");
                if wal_path.exists() {
                    let _ = std::fs::copy(&wal_path, backup_path.with_extension("bak-wal"));
                }
            }
            Err(e) => {
                log::warn!("Failed to backup database: {}", e);
                return;
            }
        }

        // Keep only the 5 most recent backups
        Self::prune_backups(&backup_dir, &db_name, 5);
    }

    fn prune_backups(backup_dir: &Path, db_name: &str, keep: usize) {
        let prefix = format!("{}_", db_name);
        let mut backups: Vec<_> = std::fs::read_dir(backup_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with(&prefix) && name.ends_with(".bak")
            })
            .collect();

        backups.sort_by_key(|e| e.file_name());
        backups.reverse();

        for old in backups.into_iter().skip(keep) {
            let _ = std::fs::remove_file(old.path());
            // Also remove WAL backup if it exists
            let wal = old.path().with_extension("bak-wal");
            if wal.exists() {
                let _ = std::fs::remove_file(wal);
            }
        }
    }
}
