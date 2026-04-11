use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

pub struct DbPath(pub PathBuf);

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub file_name: String,
    pub timestamp: String,
}

#[tauri::command]
pub fn get_latest_backup_cmd(db_path: State<'_, DbPath>) -> Result<Option<BackupInfo>, String> {
    let backup_dir = db_path
        .0
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("backups");

    if !backup_dir.exists() {
        return Ok(None);
    }

    let db_name = db_path.0.file_name().unwrap_or_default().to_string_lossy();
    let prefix = format!("{}_", db_name);

    let mut backups: Vec<_> = std::fs::read_dir(&backup_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(&prefix) && name.ends_with(".bak")
        })
        .collect();

    backups.sort_by_key(|e| e.file_name());
    backups.reverse();

    if let Some(latest) = backups.first() {
        let file_name = latest.file_name().to_string_lossy().to_string();
        // Extract timestamp from filename: "worldthreads.db_20260408_143000.bak"
        let timestamp = file_name
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".bak"))
            .map(|ts| {
                // Convert "20260408_143000" -> "2026-04-08 14:30:00"
                if ts.len() == 15 {
                    format!(
                        "{}-{}-{} {}:{}:{}",
                        &ts[0..4],
                        &ts[4..6],
                        &ts[6..8],
                        &ts[9..11],
                        &ts[11..13],
                        &ts[13..15]
                    )
                } else {
                    ts.to_string()
                }
            })
            .unwrap_or_default();

        Ok(Some(BackupInfo {
            file_name,
            timestamp,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn backup_now_cmd(db_path: State<'_, DbPath>) -> Result<BackupInfo, String> {
    crate::db::Database::backup_database(&db_path.0);
    // Return the latest backup info
    get_latest_backup_cmd(db_path)?.ok_or_else(|| "Backup failed".to_string())
}

#[tauri::command]
pub fn restore_backup_cmd(
    db_path: State<'_, DbPath>,
    backup_file_name: String,
) -> Result<(), String> {
    let backup_dir = db_path
        .0
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("backups");

    let backup_path = backup_dir.join(&backup_file_name);
    if !backup_path.exists() {
        return Err("Backup file not found".to_string());
    }

    // Copy the backup over the current database
    std::fs::copy(&backup_path, &db_path.0).map_err(|e| format!("Failed to restore backup: {e}"))?;

    // Also restore WAL file if a backup exists, otherwise remove the current WAL
    let backup_wal = backup_path.with_extension("bak-wal");
    let db_wal = db_path.0.with_extension("db-wal");
    if backup_wal.exists() {
        let _ = std::fs::copy(&backup_wal, &db_wal);
    } else if db_wal.exists() {
        let _ = std::fs::remove_file(&db_wal);
    }

    // Remove SHM file so SQLite rebuilds it on next open
    let db_shm = db_path.0.with_extension("db-shm");
    if db_shm.exists() {
        let _ = std::fs::remove_file(&db_shm);
    }

    Ok(())
}
