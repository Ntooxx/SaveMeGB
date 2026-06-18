use crate::model::{DeleteStrategy, PurgeError, PurgeReport};
use anyhow::{Context, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn safe_purge(paths: &[PathBuf], strategy: &DeleteStrategy) -> Result<PurgeReport> {
    let job_id = uuid::Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();
    let mut moved = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();
    let mut bytes_freed: u64 = 0;

    for p in paths {
        if !p.exists() {
            skipped.push(p.clone());
            continue;
        }
        let size = path_size(p);
        match strategy {
            DeleteStrategy::RecycleBin => match trash::delete(p) {
                Ok(()) => {
                    bytes_freed += size;
                    moved.push(p.clone());
                }
                Err(e) => {
                    if let Some(detail) = lock_hint(p) {
                        errors.push(PurgeError {
                            path: p.clone(),
                            message: format!("recycle failed: {e}. {detail}"),
                        });
                    } else {
                        errors.push(PurgeError {
                            path: p.clone(),
                            message: format!("recycle failed: {e}"),
                        });
                    }
                }
            },
            DeleteStrategy::BackupFolder { path: backup_root } => {
                let dest = unique_dest(backup_root, p);
                match move_into(p, &dest) {
                    Ok(_) => {
                        bytes_freed += size;
                        moved.push(dest);
                    }
                    Err(e) => errors.push(PurgeError {
                        path: p.clone(),
                        message: format!("backup move failed: {e}"),
                    }),
                }
            }
            DeleteStrategy::DirectDelete => match std::fs::remove_dir_all(p) {
                Ok(()) => {
                    bytes_freed += size;
                    moved.push(p.clone());
                }
                Err(e) => {
                    if let Some(detail) = lock_hint(p) {
                        errors.push(PurgeError {
                            path: p.clone(),
                            message: format!("delete failed: {e}. {detail}"),
                        });
                    } else {
                        errors.push(PurgeError {
                            path: p.clone(),
                            message: format!("delete failed: {e}"),
                        });
                    }
                }
            },
        }
    }

    Ok(PurgeReport {
        job_id,
        moved,
        skipped,
        errors,
        bytes_freed,
        started_at,
        finished_at: Utc::now().to_rfc3339(),
    })
}

fn lock_hint(p: &Path) -> Option<String> {
    let lower = p.to_string_lossy().to_lowercase();
    if lower.contains("dxcache") || lower.contains("d3dscache") || lower.contains("glcache")
        || lower.contains("pipeline") || lower.contains("shader") || lower.contains("compute")
        || lower.contains("nvcache") || lower.contains("nvidia") || lower.contains("amd\\dxcache")
    {
        return Some("This file may be in use by the GPU driver. Try Force Delete or close any running game/engine.".into());
    }
    if lower.contains("unreal") || lower.contains("unity") {
        return Some("Engine cache may be in use by a running editor or game.".into());
    }
    None
}

fn unique_dest(root: &Path, src: &Path) -> PathBuf {
    let name = src
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| src.as_os_str().to_os_string());
    let mut dest = root.join(&name);
    let mut n = 1u32;
    while dest.exists() {
        let s = name.to_string_lossy().to_string();
        dest = root.join(format!("{s}.{n}"));
        n += 1;
    }
    dest
}

fn path_size(p: &Path) -> u64 {
    let mut total = 0u64;
    for entry in WalkDir::new(p).into_iter().flatten() {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn move_into(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).context("create dest parent")?;
    }
    if std::fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    copy_dir_recursive(src, dest).context("copy recursive")?;
    std::fs::remove_dir_all(src).context("remove source")?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest).context("mkdir")?;
    for entry in std::fs::read_dir(src)?.flatten() {
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).context("copy file")?;
        }
    }
    Ok(())
}
