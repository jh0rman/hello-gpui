// Storage layer — reads/writes request collections from local JSON files.
// Collections live in ~/Documents/Makako/<collection>/<request>.json.
// env.json in any directory defines variables for {{interpolation}}.

use std::collections::HashMap;
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

// ── CollectionNode ─────────────────────────────────────────────────────────────

/// In-memory representation of the collection directory tree.
#[derive(Debug, Clone)]
pub enum CollectionNode {
    Folder {
        name: String,
        path: PathBuf,
        children: Vec<CollectionNode>,
    },
    Request {
        /// Display name — file stem (e.g. "get-users").
        name: String,
        path: PathBuf,
    },
}

impl CollectionNode {
    pub fn name(&self) -> &str {
        match self {
            CollectionNode::Folder { name, .. } => name,
            CollectionNode::Request { name, .. } => name,
        }
    }
}

// ── Directory helpers ──────────────────────────────────────────────────────────

/// Returns ~/Documents/Makako/, creating it if necessary.
pub fn makako_root_dir() -> PathBuf {
    let dir = dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Makako");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Returns ~/Documents/Makako/default/, creating it if necessary.
pub fn default_collection_dir() -> PathBuf {
    let dir = makako_root_dir().join("default");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// ── Tree loading ───────────────────────────────────────────────────────────────

/// Reads `root` recursively and builds a sorted collection tree.
/// Folders come before requests; each group sorted alphabetically.
/// Files named `env.json` are skipped (reserved for environment variables).
pub fn load_collection_tree(root: &Path) -> Vec<CollectionNode> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return vec![];
    };

    let mut folders: Vec<CollectionNode> = vec![];
    let mut requests: Vec<CollectionNode> = vec![];

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        if path.is_dir() {
            folders.push(CollectionNode::Folder {
                children: load_collection_tree(&path),
                name,
                path,
            });
        } else if path.extension().is_some_and(|e| e == "json") && name != "env" {
            requests.push(CollectionNode::Request { name, path });
        }
    }

    folders.sort_by(|a, b| a.name().cmp(b.name()));
    requests.sort_by(|a, b| a.name().cmp(b.name()));
    folders.extend(requests);
    folders
}

// ── Environment variables ─────────────────────────────────────────────────────

/// Loads `env.json` from `dir`. Returns an empty map if the file doesn't exist
/// or fails to parse.
///
/// Format: a flat JSON object, e.g. `{ "base_url": "https://api.example.com" }`
pub fn load_env(dir: &Path) -> HashMap<String, String> {
    let path = dir.join("env.json");
    let Ok(data) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Replaces every `{{key}}` occurrence in `text` with the matching value from
/// `env`. Unknown variables are left as-is.
pub fn interpolate(text: &str, env: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in env {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
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
