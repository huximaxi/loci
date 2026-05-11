// ─── MCP resource handlers ────────────────────────────────────────────────────
//
// Implements `loci://` URI scheme for MCP resources.
//
// Resource scheme:
//   loci://locus/{id}           → single Locus node
//   loci://room/{id}/loci       → all Loci in a Room
//   loci://search?q={query}     → keyword search over Loci titles + content
//
// Storage layout read by this module:
//   ~/.loci/loci/{id}.md        → individual Locus files
//
// File format (YAML frontmatter + Markdown body):
//   ---
//   id: locus-2026-05-09-sovereignty
//   title: Cognitive Sovereignty
//   tags: [sovereignty, loci]
//   roomId: research
//   createdAt: 1715246400000
//   ---
//
//   Markdown body here.
//
// Cipher gate: this module reads ONLY from ~/.loci/loci/. It NEVER touches
// ~/.loci/conversations/ or any IndexedDB data. THREAT-01 enforced at this layer.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ─── Types ───────────────────────────────────────────────────────────────────

/// A Locus node as represented in the MCP resource response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: String,
    pub metadata: LocusMetadata,
}

/// Metadata block embedded in every MCP resource response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocusMetadata {
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    pub created_at: u64,
}

/// Parsed YAML frontmatter from a Locus .md file.
#[derive(Debug, Default)]
struct LocusFrontmatter {
    id: String,
    title: String,
    tags: Vec<String>,
    room_id: Option<String>,
    created_at: u64,
}

// ─── Internal: frontmatter parser ────────────────────────────────────────────
//
// Intentionally minimal — no external YAML crate. The frontmatter schema is
// controlled by us; a hand-rolled parser is safer than a general YAML parser
// (no type coercion, no arbitrary key injection).

fn parse_frontmatter(content: &str) -> (LocusFrontmatter, &str) {
    let mut fm = LocusFrontmatter::default();

    if !content.starts_with("---") {
        return (fm, content);
    }

    // Find closing ---
    let rest = &content[3..];
    let close = match rest.find("\n---") {
        Some(pos) => pos,
        None => return (fm, content),
    };

    let fm_block = &rest[..close];
    // Body starts after the closing ---\n
    let body = rest.get(close + 4..).unwrap_or("").trim_start_matches('\n');

    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "id" => fm.id = val.to_string(),
                "title" => fm.title = val.to_string(),
                "roomId" => fm.room_id = Some(val.to_string()),
                "createdAt" => {
                    fm.created_at = val.parse().unwrap_or(0);
                }
                "tags" => {
                    // Accept: [sovereignty, loci] or [sovereignty, loci] with spaces
                    let inner = val.trim_matches(|c| c == '[' || c == ']');
                    fm.tags = inner
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    (fm, body)
}

fn file_to_resource(path: &Path) -> Option<McpResource> {
    let content = fs::read_to_string(path).ok()?;
    let (fm, body) = parse_frontmatter(&content);

    // Derive id from filename if frontmatter is missing it
    let id = if fm.id.is_empty() {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        fm.id.clone()
    };

    let title = if fm.title.is_empty() {
        id.clone()
    } else {
        fm.title.clone()
    };

    Some(McpResource {
        uri: format!("loci://locus/{}", id),
        name: title,
        mime_type: "text/markdown".to_string(),
        text: body.to_string(),
        metadata: LocusMetadata {
            tags: fm.tags,
            room_id: fm.room_id,
            created_at: fm.created_at,
        },
    })
}

fn loci_dir(base: &Path) -> PathBuf {
    base.join("loci")
}

/// Returns true if the given room_id is exposed given the allowlist.
///
/// Cipher gate:
/// - Empty allowlist = all rooms exposed (default, opt-in restriction)
/// - Unroomed loci (room_id = None) are always accessible
/// - Room-assigned loci are accessible only if their room_id is in the allowlist
fn room_is_exposed(room_id: Option<&str>, expose_rooms: &[String]) -> bool {
    if expose_rooms.is_empty() {
        return true;
    }
    match room_id {
        None => true, // unroomed loci always accessible
        Some(rid) => expose_rooms.iter().any(|r| r.as_str() == rid),
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Return all Locus nodes as a flat list of MCP resource descriptors
/// (uri + name + mimeType only — no text body, per MCP resources/list spec).
pub fn list_loci(loci_base: &Path, expose_rooms: &[String]) -> Vec<serde_json::Value> {
    let dir = loci_dir(loci_base);
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x == "md")
                .unwrap_or(false)
        })
        .filter_map(|e| file_to_resource(&e.path()))
        .filter(|r| room_is_exposed(r.metadata.room_id.as_deref(), expose_rooms))
        .map(|r| {
            serde_json::json!({
                "uri": r.uri,
                "name": r.name,
                "mimeType": r.mime_type,
                "metadata": r.metadata,
            })
        })
        .collect()
}

/// Return a single Locus node by ID (full text included).
pub fn read_locus(id: &str, loci_base: &Path, expose_rooms: &[String]) -> Option<McpResource> {
    // Sanitise id: only allow alphanumeric, hyphens, underscores.
    // Prevents path traversal (Cipher gate).
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return None;
    }

    let path = loci_dir(loci_base).join(format!("{}.md", id));
    let resource = file_to_resource(&path)?;
    if !room_is_exposed(resource.metadata.room_id.as_deref(), expose_rooms) {
        return None;
    }
    Some(resource)
}

/// Return all Loci belonging to a specific Room.
pub fn list_room_loci(room_id: &str, loci_base: &Path, expose_rooms: &[String]) -> Vec<McpResource> {
    let dir = loci_dir(loci_base);
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x == "md")
                .unwrap_or(false)
        })
        .filter_map(|e| file_to_resource(&e.path()))
        .filter(|r| r.metadata.room_id.as_deref() == Some(room_id))
        .filter(|r| room_is_exposed(r.metadata.room_id.as_deref(), expose_rooms))
        .collect()
}

/// Simple keyword search over Locus titles and content.
/// Returns up to 20 results, ranked by match count (title matches weighted 2x).
pub fn search_loci(query: &str, loci_base: &Path, expose_rooms: &[String]) -> Vec<McpResource> {
    let dir = loci_dir(loci_base);
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };

    // Normalise query tokens
    let tokens: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|t| t.to_string())
        .collect();

    if tokens.is_empty() {
        return vec![];
    }

    let mut scored: Vec<(u32, McpResource)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x == "md")
                .unwrap_or(false)
        })
        .filter_map(|e| file_to_resource(&e.path()))
        .filter(|r| room_is_exposed(r.metadata.room_id.as_deref(), expose_rooms))
        .filter_map(|r| {
            let title_lower = r.name.to_lowercase();
            let text_lower = r.text.to_lowercase();
            let score: u32 = tokens
                .iter()
                .map(|t| {
                    let in_title = if title_lower.contains(t.as_str()) { 2u32 } else { 0 };
                    let in_body = text_lower.matches(t.as_str()).count() as u32;
                    in_title + in_body
                })
                .sum();
            if score > 0 { Some((score, r)) } else { None }
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(20).map(|(_, r)| r).collect()
}
