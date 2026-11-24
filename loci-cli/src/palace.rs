//! Palace detection + read primitives for the CLI.
//!
//! Accepts three layouts:
//!   * legacy:        marker at root + `_palace/` subdir holding rooms.
//!   * rooms-dir:     marker at root + `rooms/` subdir holding rooms
//!                    (the shape the templates kit and the setup guides build).
//!   * rooms-at-root: marker at root + sibling dirs each holding CLAUDE.md.
//!
//! A marker is `WORKFOLDER.md`, `PALACE.md` or `CLAUDE.md` (see `MARKERS`).
//!
//! `rooms/` is checked before rooms-at-root because an explicit rooms directory
//! is the stronger signal.
//!
//! Plain stdlib + std::fs, no shared crate.

use std::fs;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &["_palace", "node_modules", "target", "cron"];

pub enum Layout {
    PalaceSubdir,
    RoomsDir,
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

/// The root marker files, canonical spelling first. `WORKFOLDER.md` is the 2.0 name; `PALACE.md`
/// and `CLAUDE.md` stay valid for good, with no warning. All three are read the same way.
pub const MARKERS: [&str; 3] = ["WORKFOLDER.md", "PALACE.md", "CLAUDE.md"];

pub fn is_marker(file_name: &str) -> bool {
    MARKERS.contains(&file_name)
}

/// An environment variable is set but names no workfolder. Detection stops here instead of
/// falling back to the current directory, so a typo never silently points the CLI at a different
/// workfolder. `var` is the variable that was read, so the message names the right one.
pub struct BadEnv {
    pub var: &'static str,
    pub path: PathBuf,
}

/// Resolution order: explicit override (`--workfolder`, alias `--palace`), then `$LOCI_WORKFOLDER`,
/// then `$LOCI_PALACE`, then walk up from cwd. An empty variable counts as unset. When both
/// variables are set, `$LOCI_WORKFOLDER` wins and nothing is printed about the other.
pub fn detect(override_path: Option<PathBuf>) -> Result<Option<Palace>, BadEnv> {
    detect_from(
        override_path,
        std::env::var_os("LOCI_WORKFOLDER"),
        std::env::var_os("LOCI_PALACE"),
        std::env::current_dir().ok(),
    )
}

fn detect_from(
    override_path: Option<PathBuf>,
    workfolder_env: Option<std::ffi::OsString>,
    palace_env: Option<std::ffi::OsString>,
    cwd: Option<PathBuf>,
) -> Result<Option<Palace>, BadEnv> {
    if let Some(p) = override_path {
        return Ok(validate(&p));
    }
    let env = workfolder_env
        .filter(|e| !e.is_empty())
        .map(|e| ("LOCI_WORKFOLDER", e))
        .or_else(|| {
            palace_env
                .filter(|e| !e.is_empty())
                .map(|e| ("LOCI_PALACE", e))
        });
    if let Some((var, env)) = env {
        let path = PathBuf::from(env);
        return match validate(&path) {
            Some(palace) => Ok(Some(palace)),
            None => Err(BadEnv { var, path }),
        };
    }
    let Some(mut cwd) = cwd else {
        return Ok(None);
    };
    loop {
        if let Some(p) = validate(&cwd) {
            return Ok(Some(p));
        }
        if !cwd.pop() {
            return Ok(None);
        }
    }
}

pub fn validate(root: &Path) -> Option<Palace> {
    if !root.is_dir() {
        return None;
    }
    if !MARKERS.iter().any(|m| root.join(m).exists()) {
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
    let rooms_dir = root.join("rooms");
    if rooms_dir.is_dir() && has_room_in(&rooms_dir) {
        return Some(Palace {
            root: root.to_path_buf(),
            scan_root: rooms_dir,
            layout: Layout::RoomsDir,
        });
    }
    if has_room_in(root) {
        return Some(Palace {
            root: root.to_path_buf(),
            scan_root: root.to_path_buf(),
            layout: Layout::RoomsAtRoot,
        });
    }
    None
}

/// True when `dir` holds at least one non-skipped subdirectory with a CLAUDE.md.
fn has_room_in(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Fresh scratch dir per test, keyed by name + pid so parallel tests never collide.
    fn scratch(name: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("loci_cli_palace_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn room(dir: &Path, name: &str) {
        let r = dir.join(name);
        fs::create_dir_all(&r).unwrap();
        fs::write(r.join("CLAUDE.md"), "# room\n").unwrap();
        fs::write(r.join("note.md"), "crystal\n").unwrap();
    }

    #[test]
    fn palace_subdir_layout() {
        let root = scratch("subdir");
        fs::write(root.join("PALACE.md"), "# palace\n").unwrap();
        room(&root.join("_palace"), "ideas-room");
        let p = validate(&root).expect("palace detected");
        assert!(matches!(p.layout, Layout::PalaceSubdir));
        assert_eq!(p.scan_root, root.join("_palace"));
        assert_eq!(list_rooms(&p).len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rooms_dir_layout_as_built_by_the_kit() {
        let root = scratch("roomsdir");
        fs::write(root.join("CLAUDE.md"), "# master\n").unwrap();
        fs::create_dir_all(root.join("soul")).unwrap();
        fs::write(root.join("soul").join("SOUL.md"), "# soul\n").unwrap();
        fs::create_dir_all(root.join("_templates")).unwrap();
        fs::write(root.join("_templates").join("CLAUDE-master.md"), "# tpl\n").unwrap();
        room(&root.join("rooms"), "work-room");
        room(&root.join("rooms"), "writing-room");
        let p = validate(&root).expect("kit palace detected");
        assert!(matches!(p.layout, Layout::RoomsDir));
        assert_eq!(p.scan_root, root.join("rooms"));
        let names: Vec<String> = list_rooms(&p).into_iter().map(|r| r.name).collect();
        assert_eq!(names, vec!["work-room", "writing-room"]);
        assert_eq!(find_crystal(&p, "note", Some("work-room")).len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rooms_at_root_layout() {
        let root = scratch("atroot");
        fs::write(root.join("CLAUDE.md"), "# master\n").unwrap();
        room(&root, "creative-room");
        room(&root, "research-room");
        let p = validate(&root).expect("palace detected");
        assert!(matches!(p.layout, Layout::RoomsAtRoot));
        assert_eq!(p.scan_root, root);
        assert_eq!(list_rooms(&p).len(), 2);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn empty_rooms_dir_is_not_a_palace() {
        let root = scratch("emptyrooms");
        fs::write(root.join("CLAUDE.md"), "# master\n").unwrap();
        fs::create_dir_all(root.join("rooms")).unwrap();
        assert!(validate(&root).is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn bad_env_stops_instead_of_falling_back_to_cwd() {
        let good = scratch("env_good");
        fs::write(good.join("CLAUDE.md"), "# master\n").unwrap();
        room(&good, "work-room");
        let missing = good.join("no-such-palace");
        // cwd holds a valid palace, but the env var names a bad path: stop, do not fall back.
        let r = detect_from(
            None,
            None,
            Some(missing.clone().into_os_string()),
            Some(good.clone()),
        );
        match r {
            Err(BadEnv { var, path }) => {
                assert_eq!(path, missing);
                assert_eq!(var, "LOCI_PALACE");
            }
            _ => panic!("a bad LOCI_PALACE must not fall back to the cwd palace"),
        }
        // Empty env counts as unset: the cwd walk finds the palace.
        let r = detect_from(None, None, Some("".into()), Some(good.clone()));
        assert!(matches!(r, Ok(Some(_))));
        // A good env wins over cwd; an explicit override wins over env.
        assert!(matches!(
            detect_from(None, None, Some(good.clone().into_os_string()), None),
            Ok(Some(_))
        ));
        assert!(matches!(
            detect_from(
                Some(missing.clone()),
                None,
                Some(good.clone().into_os_string()),
                None
            ),
            Ok(None)
        ));
        let _ = fs::remove_dir_all(&good);
    }

    #[test]
    fn workfolder_md_is_a_marker_like_the_other_two() {
        for marker in ["WORKFOLDER.md", "PALACE.md", "CLAUDE.md"] {
            let root = scratch("marker");
            fs::write(root.join(marker), "# w\n").unwrap();
            room(&root.join("rooms"), "work-room");
            assert!(validate(&root).is_some(), "{marker} must mark a workfolder");
            let _ = fs::remove_dir_all(&root);
        }
        // The marker alone, with no room, is still not a workfolder.
        let root = scratch("marker_alone");
        fs::write(root.join("WORKFOLDER.md"), "# w\n").unwrap();
        assert!(validate(&root).is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn workfolder_env_wins_and_the_old_name_still_works() {
        let good = scratch("env_pair");
        fs::write(good.join("WORKFOLDER.md"), "# w\n").unwrap();
        room(&good, "work-room");
        let other = scratch("env_pair_other");
        fs::write(other.join("CLAUDE.md"), "# o\n").unwrap();
        room(&other, "other-room");
        let os = |p: &PathBuf| Some(p.clone().into_os_string());
        // Either name alone resolves.
        let a = detect_from(None, os(&good), None, None)
            .ok()
            .flatten()
            .expect("LOCI_WORKFOLDER");
        let b = detect_from(None, None, os(&good), None)
            .ok()
            .flatten()
            .expect("LOCI_PALACE");
        assert_eq!(a.root, b.root);
        // Both set: the new name wins, with no error about the old one.
        let both = detect_from(None, os(&good), os(&other), None)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(both.root, good);
        // An empty new name falls through to the old one.
        let r = detect_from(None, Some("".into()), os(&other), None)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(r.root, other);
        // A bad new name stops, naming itself, even though the old one is good.
        let missing = good.join("nope");
        match detect_from(None, os(&missing), os(&good), None) {
            Err(BadEnv { var, .. }) => assert_eq!(var, "LOCI_WORKFOLDER"),
            _ => panic!("a bad LOCI_WORKFOLDER must not fall through to LOCI_PALACE"),
        }
        let _ = fs::remove_dir_all(&good);
        let _ = fs::remove_dir_all(&other);
    }

    #[test]
    fn no_marker_file_is_not_a_palace() {
        let root = scratch("nomarker");
        room(&root.join("rooms"), "work-room");
        assert!(validate(&root).is_none());
        let _ = fs::remove_dir_all(&root);
    }
}
