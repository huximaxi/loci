// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct DetectionResult {
    found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rooms: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    crystal_count: Option<usize>,
    suggestion: String,
}

#[tauri::command]
fn detect_palace(search_path: String) -> DetectionResult {
    let path = Path::new(&search_path);

    // 1. Check for loci palace (already migrated)
    if let Some(home) = dirs::home_dir() {
        let loci_path = home.join(".loci");
        if loci_path.exists() && loci_path.join("CLAUDE.md").exists() {
            return DetectionResult {
                found: true,
                kind: Some("loci".to_string()),
                path: Some(loci_path.to_string_lossy().to_string()),
                rooms: Some(detect_rooms(&loci_path)),
                crystal_count: Some(count_crystals(&loci_path)),
                suggestion: "You already have a loci palace at ~/.loci/".to_string(),
            };
        }
    }

    // 2. Check for _palace/ pattern (Vesper × Hux)
    let palace_dir = path.join("_palace");
    if palace_dir.exists() {
        return DetectionResult {
            found: true,
            kind: Some("mempalace".to_string()),
            path: Some(palace_dir.to_string_lossy().to_string()),
            rooms: Some(detect_rooms(&palace_dir)),
            crystal_count: Some(count_crystals(&palace_dir)),
            suggestion: "Found Vesper × Hux memory palace. Ready to migrate to loci format.".to_string(),
        };
    }

    // 3. Check for mila-mempalace/ pattern
    let mila_dir = path.join("mila-mempalace");
    if mila_dir.exists() {
        return DetectionResult {
            found: true,
            kind: Some("mila-mempalace".to_string()),
            path: Some(mila_dir.to_string_lossy().to_string()),
            rooms: Some(detect_rooms(&mila_dir)),
            crystal_count: Some(count_crystals(&mila_dir)),
            suggestion: "Found Mila's memory palace. Ready to migrate to loci format.".to_string(),
        };
    }

    // 4. Check for karpathy pattern (LLM folder structure)
    let llm_dir = path.join("LLM");
    if llm_dir.exists() && llm_dir.join("CLAUDE.md").exists() {
        return DetectionResult {
            found: true,
            kind: Some("karpathy".to_string()),
            path: Some(llm_dir.to_string_lossy().to_string()),
            rooms: None,
            crystal_count: Some(count_md_files(&llm_dir)),
            suggestion: "Found Karpathy-style LLM folder. Ready to migrate to loci format.".to_string(),
        };
    }

    // 5. No palace found
    DetectionResult {
        found: false,
        kind: None,
        path: None,
        rooms: None,
        crystal_count: None,
        suggestion: "No memory palace detected. Would you like to create one?".to_string(),
    }
}

#[tauri::command]
fn migrate_to_loci(source_path: String) -> Result<String, String> {
    let source = Path::new(&source_path);

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let dest = home.join(".loci");

    // Create destination if it doesn't exist
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create .loci directory: {}", e))?;

    // Copy all contents recursively
    copy_dir_recursive(source, &dest)?;

    // Create loci manifest if it doesn't exist
    let manifest_path = dest.join("loci.json");
    if !manifest_path.exists() {
        let manifest = serde_json::json!({
            "version": "1.0",
            "migrated_from": source.to_string_lossy(),
            "migrated_at": chrono::Utc::now().to_rfc3339(),
        });
        fs::write(manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
            .map_err(|e| format!("Failed to create manifest: {}", e))?;
    }

    Ok(dest.to_string_lossy().to_string())
}

// Helper: count rooms (subdirectories)
fn detect_rooms(path: &Path) -> usize {
    if let Ok(entries) = fs::read_dir(path) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .count()
    } else {
        0
    }
}

// Helper: count crystals (CLAUDE.md + .md files)
fn count_crystals(path: &Path) -> usize {
    count_md_files(path)
}

// Helper: count all .md files recursively
fn count_md_files(path: &Path) -> usize {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                count += count_md_files(&entry_path);
            } else if entry_path.extension().and_then(|s| s.to_str()) == Some("md") {
                count += 1;
            }
        }
    }

    count
}

// Helper: recursive directory copy
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![detect_palace, migrate_to_loci])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
