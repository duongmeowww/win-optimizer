use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};

#[derive(serde::Serialize, Clone)]
pub struct DuplicateFile {
    pub path: String,
    pub size: u64,
    pub hash: String,
}

#[derive(serde::Serialize, Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<String>,
}

#[tauri::command]
pub async fn scan_duplicate_files(dir: String, min_size_mb: u64) -> Vec<DuplicateGroup> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = Path::new(&dir);
        if !root.exists() {
            return Vec::new();
        }

        let min_bytes = min_size_mb * 1024 * 1024;
        let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();

        let walker = walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    let len = meta.len();
                    if len >= min_bytes {
                        size_map.entry(len).or_default().push(entry.path().to_path_buf());
                    }
                }
            }
        }

        let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for (_, paths) in size_map {
            if paths.len() < 2 {
                continue;
            }
            for path in paths {
                if let Ok(hash) = compute_file_hash(&path) {
                    hash_map.entry(hash).or_default().push(path);
                }
            }
        }

        let mut groups = Vec::new();
        for (hash, paths) in hash_map {
            if paths.len() >= 2 {
                let first_meta = fs::metadata(&paths[0]).ok();
                let size = first_meta.map(|m| m.len()).unwrap_or(0);
                let file_strings = paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
                groups.push(DuplicateGroup {
                    hash,
                    size,
                    files: file_strings,
                });
            }
        }

        groups
    })
    .await
    .unwrap_or_default()
}

fn compute_file_hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
