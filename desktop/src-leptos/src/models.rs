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
    /// Companion name from PALACE.md, or None for legacy palaces.
    #[serde(default)]
    pub companion: Option<String>,
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

/// Slim schema-panel payload from `read_manifest_summary`: counts + meta only,
/// so the dashboard doesn't deserialize the whole node graph in WASM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSummary {
    pub manifest_version: String,
    pub vocabulary: String,
    pub captured_ts_utc: String,
    pub tree_hash: String,
    pub node_count: usize,
    pub relation_count: usize,
    pub edge_count: usize,
}

/// Application-wide active palace. Held in a Leptos signal via provide_context.
#[derive(Debug, Clone, Default)]
pub struct ActivePalace {
    pub path: Option<String>,
    pub manifest: Option<PalaceManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobSnapshot {
    // Filesystem dir key: stable identity for detail lookups. Distinct from
    // `job`, the content-derived display label which may contain '/'.
    pub key: String,
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

// ─── rc.3 cockpit ────────────────────────────────────────────────────────────

/// A self-contained palace-map instrument discovered at the palace surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PalaceMapEntry {
    pub key: String,
    pub file: String,
    pub label: String,
    pub badge: Option<String>,
}

/// One row of the tools gate-ledger (`tools.items` in the palace map JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolLedgerEntry {
    pub id: String,
    pub label: Option<String>,
    pub kind: Option<String>,
    pub room: Option<String>,
    pub source: Option<String>,
    pub license: Option<String>,
    pub quarantine_state: Option<String>,
    pub gate_read: Option<String>,
}

/// One alert surfaced by alert-watcher-daily. Parsed from the `alerts` array in
/// that job's state.json (lazily, via read_cron_detail). Read-only for now: the
/// "solve/action" affordance is deferred (write-path).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertItem {
    pub job: String,
    pub condition: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub posted: bool,
}

/// Mirrors the InferenceStatus struct returned by `check_inference_available`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStatus {
    pub has_local: bool,
    pub ollama_running: bool,
    pub has_claude: bool,
    pub local_models: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestlogItem {
    pub done: bool,
    pub title: String,
    pub body: String,
    // The `## Heading` track this quest sits under in TASKS.md.
    pub track: String,
}

// ── palace-update: the delta checker ──────────────────────────────────────────
// Mirrors UpdateReport / DeltaEntry / DeltaItem in src-tauri/src/main.rs. Field
// names MUST stay in lockstep (snake_case, no rename: these are command returns).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaItem {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub version: String,
    pub date: String,
    pub is_candidate: bool,
    pub items: Vec<DeltaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateReport {
    /// None when the palace predates the methodology anchor.
    pub local_version: Option<String>,
    pub latest_version: String,
    /// "current" | "behind" | "unknown" | "unavailable"
    pub status: String,
    pub entries: Vec<DeltaEntry>,
    pub include_candidates: bool,
}
