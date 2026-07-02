//! Palace detection + read primitives for the CLI.
//!
//! Mirrors the desktop's dual-layout acceptance:
//!   * legacy:        PALACE.md/CLAUDE.md at root + `_palace/` subdir holding rooms.
//!   * rooms-at-root: PALACE.md/CLAUDE.md at root + sibling dirs each holding CLAUDE.md.
//!
//! Re-expressed for the CLI in stdlib + std::fs. No shared crate with the desktop:
//! the public CLI is a separate door into the same shape.

use std::fs;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &["_palace", "node_modules", "target", "cron"];

pub enum Layout {
    PalaceSubdir,
    RoomsAtRoot,
}

pub struct Palace {
    pub root: PathBuf,
    pub scan_root: PathBuf,
    pub layout: Layout,
}

pub struct Room {
    pub name: String,
    pub path: PathBuf,
    pub crystal_count: usize,
}

/// Resolution order: explicit override, then `$LOCI_PALACE`, then walk up from cwd.
pub fn detect(override_path: Option<PathBuf>) -> Option<Palace> {
    if let Some(p) = override_path {
        return validate(&p);
    }
    if let Some(env) = std::env::var_os("LOCI_PALACE") {
        if let Some(p) = validate(Path::new(&env)) {
            return Some(p);
        }
    }
    let mut cwd = std::env::current_dir().ok()?;
    loop {
        if let Some(p) = validate(&cwd) {
            return Some(p);
        }
        if !cwd.pop() {
            return None;
        }
    }
}

pub fn validate(root: &Path) -> Option<Palace> {
    if !root.is_dir() {
        return None;
    }
    if !(root.join("PALACE.md").exists() || root.join("CLAUDE.md").exists()) {
        return None;
    }
    let palace_dir = root.join("_palace");
    if palace_dir.is_dir() {
        return Some(Palace {
            root: root.to_path_buf(),
            scan_root: palace_dir,
            layout: Layout::PalaceSubdir,
        });
    }
    if has_room_at_root(root) {
        return Some(Palace {
            root: root.to_path_buf(),
            scan_root: root.to_path_buf(),
            layout: Layout::RoomsAtRoot,
        });
    }
    None
}

fn has_room_at_root(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if should_skip(&entry.file_name().to_string_lossy()) {
            continue;
        }
        let p = entry.path();
        if p.is_dir() && p.join("CLAUDE.md").exists() {
            return true;
        }
    }
    false
}

fn should_skip(name: &str) -> bool {
    name.starts_with('.') || SKIP_DIRS.contains(&name)
}

pub fn list_rooms(p: &Palace) -> Vec<Room> {
    let mut rooms = Vec::new();
    let Ok(entries) = fs::read_dir(&p.scan_root) else {
        return rooms;
    };
    let mut dirs: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    dirs.sort_by_key(|e| e.file_name());
    for entry in dirs {
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip(&name) {
            continue;
        }
        let room_path = entry.path();
        if !room_path.is_dir() || !room_path.join("CLAUDE.md").exists() {
            continue;
        }
        let crystal_count = count_md_files(&room_path);
        rooms.push(Room {
            name,
            path: room_path,
            crystal_count,
        });
    }
    rooms
}

pub fn count_md_files(dir: &Path) -> usize {
    let mut count = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
                count += 1;
            }
        }
    }
    count
}

/// Find a crystal (`.md` file) by slug. If `room` is given, search only that room.
/// Slug = filename stem, case-insensitive. Returns up to a handful of matches so
/// the caller can disambiguate without re-scanning.
pub fn find_crystal(p: &Palace, slug: &str, room: Option<&str>) -> Vec<PathBuf> {
    let slug_lc = slug.to_lowercase();
    let mut hits = Vec::new();
    let roots: Vec<PathBuf> = if let Some(r) = room {
        let path = p.scan_root.join(r);
        if path.is_dir() {
            vec![path]
        } else {
            return hits;
        }
    } else {
        list_rooms(p).into_iter().map(|r| r.path).collect()
    };
    for r in roots {
        walk_for_slug(&r, &slug_lc, &mut hits);
    }
    hits
}

fn walk_for_slug(dir: &Path, slug_lc: &str, hits: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let p = entry.path();
        if p.is_dir() {
            walk_for_slug(&p, slug_lc, hits);
        } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if stem.to_lowercase() == slug_lc {
                    hits.push(p);
                }
            }
        }
    }
}

/// Return the newest `.md` file under a path named like a handover.
/// Convention: filename contains "HANDOVER" (any case) OR lives in a `handovers/` dir.
pub fn latest_handover(p: &Palace) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    let mut stack = vec![p.root.clone()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_md = path.extension().and_then(|e| e.to_str()) == Some("md");
            if !is_md {
                continue;
            }
            let looks_like_handover = name.to_uppercase().contains("HANDOVER")
                || d.file_name()
                    .map(|n| n.to_string_lossy().to_lowercase() == "handovers")
                    .unwrap_or(false);
            if !looks_like_handover {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    match &best {
                        Some((t, _)) if modified <= *t => {}
                        _ => best = Some((modified, path)),
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}
