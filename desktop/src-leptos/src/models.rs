// Shared types between Leptos views and Tauri backend.
// Field shapes MIRROR the Rust structs in src-tauri/src/main.rs and MUST stay
// in lockstep. Future cleanup: move both into a shared workspace crate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub name: String,
    pub path: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceManifest {
    pub path: String,
    pub rooms: Vec<RoomInfo>,
    pub cron_job_count: usize,
    pub crystal_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestNode {
    pub rel_path: String,
    pub sha256: String,
    pub size: u64,
    pub mtime: f64,
    pub mode: u32,
    pub is_symlink: bool,
    pub primitive_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRelation {
    pub rel_path: String,
    pub sha256: String,
    pub size: u64,
    pub mtime: f64,
    pub mode: u32,
    pub is_symlink: bool,
    pub primitive_class: String,
    pub endpoints: Vec<Option<String>>,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEdge {
    pub rel_path: String,
    pub is_symlink: bool,
    pub symlink_target: String,
    pub size: u64,
    pub primitive_class: String,
    #[serde(default)]
    pub symlink_target_original: Option<String>,
    #[serde(default)]
    pub flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestScope {
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub manifest_version: String,
    pub source_root: String,
    pub dest_root: String,
    pub captured_ts_utc: String,
    pub vocabulary: String,
    pub tree_hash: String,
    pub nodes: Vec<ManifestNode>,
    pub relations: Vec<ManifestRelation>,
    pub edges: Vec<ManifestEdge>,
    pub scope: ManifestScope,
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
}

/// Application-wide active palace. Held in a Leptos signal via provide_context.
#[derive(Debug, Clone, Default)]
pub struct ActivePalace {
    pub path: Option<String>,
    pub manifest: Option<PalaceManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobSnapshot {
    pub job: String,
    pub status: Option<String>,
    pub summary: Option<String>,
    pub last_run: Option<String>,
    pub ciq: Option<f64>,
    pub ciq_delta: Option<f64>,
    pub pulse: Option<String>,
    pub alert_count: Option<usize>,
    // raw is intentionally omitted here. The dashboard only needs the surfaced fields,
    // and dropping `raw` cuts the WASM-side wire payload by an order of magnitude.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverEntry {
    pub filename: String,
    pub mtime: f64,
    pub size: u64,
}
