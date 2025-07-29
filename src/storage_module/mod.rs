// Storage layer — reads/writes request collections from local JSON files.
// Collections live in ~/Documents/Makako/<collection>/<request>.json.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ── SavedRequest ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

// ── Collection directory ───────────────────────────────────────────────────────

/// Returns ~/Documents/Makako/default/, creating it if necessary.
pub fn default_collection_dir() -> PathBuf {
    let dir = dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Makako")
        .join("default");

    let _ = std::fs::create_dir_all(&dir);
    dir
}

// ── CRUD ───────────────────────────────────────────────────────────────────────

/// Saves a request to `<dir>/<name>.json`. Returns the path written.
pub fn save_request(dir: &Path, req: &SavedRequest) -> Result<PathBuf, String> {
    let safe_name: String = req
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();

    let path = dir.join(format!("{}.json", safe_name));
    let json = serde_json::to_string_pretty(req).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Loads a request from a JSON file.
pub fn load_request(path: &Path) -> Result<SavedRequest, String> {
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

/// Returns all saved requests in `dir`, sorted by name.
/// Each entry is `(display_name, file_path)`.
pub fn list_requests(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };

    let mut results: Vec<(String, PathBuf)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|e| {
            let path = e.path();
            load_request(&path).ok().map(|r| (r.name, path))
        })
        .collect();

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}
