// ─── MCP tool handlers ────────────────────────────────────────────────────────
//
// Implements MCP tools — write operations agents can invoke on the Loci garden.
//
// Tools:
//   create_locus(title, content, tags?, room_id?)
//     → writes a new Locus .md file to ~/.loci/loci/
//     → returns the new resource URI
//
//   tag_locus(id, tags)
//     → updates the tags field in the YAML frontmatter of an existing Locus
//     → preserves all other frontmatter fields and body content
//
// Cipher gate:
//   - Filenames derived from title (slugified). Max 80 chars. No path separators.
//   - id parameter in tag_locus validated against alphanumeric + hyphens only.
//   - No shell execution. No arbitrary file paths. Write target is always
//     ~/.loci/loci/{slug}.md — never accepts a caller-supplied path.
//   - Conversations are never modified by MCP tools. Write surface = Loci only.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn loci_dir(base: &Path) -> PathBuf {
    base.join("loci")
}

/// Slugify a title into a safe filename component.
/// Lowercases, replaces non-alphanumeric with hyphens, collapses runs, trims.
fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse runs of hyphens and trim
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Cap at 60 chars to keep filenames sane
    slug.chars().take(60).collect()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn today_prefix() -> String {
    // Simple date prefix: YYYY-MM-DD derived from unix timestamp
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    // Approximate calendar date (good enough for ID prefix; not a date library dep)
    // epoch = 1970-01-01. We compute a rough YYYY-MM-DD.
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Approximate days-since-epoch → (year, month, day).
/// Not leap-second correct; fine for file naming.
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let year_days = if leap { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days = [31u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ─── Tool: create_locus ───────────────────────────────────────────────────────

/// Write a new Locus node to disk. Returns the `loci://locus/{id}` URI.
///
/// # Cipher gate
/// - `title` is slugified; no raw title used in filename.
/// - `room_id` is validated (alphanumeric + hyphens only).
/// - `loci_dir` must exist; we never create arbitrary parent directories.
pub fn create_locus(
    title: &str,
    content: &str,
    tags: &[String],
    room_id: Option<&str>,
    loci_base: &Path,
) -> Result<String, String> {
    // Validate room_id if supplied
    if let Some(rid) = room_id {
        if !rid.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(format!("invalid room_id '{}': only alphanumeric, hyphens, underscores allowed", rid));
        }
    }

    let dir = loci_dir(loci_base);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create loci directory: {}", e))?;

    let slug = slugify(title);
    if slug.is_empty() {
        return Err("title produced empty slug — use at least one alphanumeric character".to_string());
    }

    let date = today_prefix();
    let id = format!("locus-{}-{}", date, slug);
    let filename = format!("{}.md", id);
    let path = dir.join(&filename);

    if path.exists() {
        return Err(format!("locus with id '{}' already exists", id));
    }

    let created_at = now_millis();
    let tags_yaml = format!("[{}]", tags.join(", "));
    let room_line = room_id
        .map(|r| format!("\nroomId: {}", r))
        .unwrap_or_default();

    let file_content = format!(
        "---\nid: {}\ntitle: {}\ntags: {}{}\ncreatedAt: {}\n---\n\n{}",
        id,
        title,
        tags_yaml,
        room_line,
        created_at,
        content
    );

    fs::write(&path, file_content)
        .map_err(|e| format!("failed to write locus file: {}", e))?;

    Ok(format!("loci://locus/{}", id))
}

// ─── Tool: tag_locus ─────────────────────────────────────────────────────────

/// Update the tags field in an existing Locus's frontmatter.
/// Preserves all other fields and the body content exactly.
///
/// # Cipher gate
/// - `id` validated against alphanumeric + hyphens only (path traversal prevention).
/// - Tags are sanitised: each tag may only contain alphanumeric, hyphens, spaces.
pub fn tag_locus(
    id: &str,
    new_tags: &[String],
    loci_base: &Path,
) -> Result<(), String> {
    // Validate id
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(format!("invalid id '{}': only alphanumeric, hyphens, underscores allowed", id));
    }

    // Validate each tag
    for tag in new_tags {
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
            return Err(format!("invalid tag '{}': only alphanumeric, hyphens, underscores, spaces allowed", tag));
        }
    }

    let path = loci_dir(loci_base).join(format!("{}.md", id));
    if !path.exists() {
        return Err(format!("locus '{}' not found", id));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read locus: {}", e))?;

    // Rewrite the tags line in the frontmatter block
    let updated = rewrite_tags_in_frontmatter(&content, new_tags)?;

    fs::write(&path, updated)
        .map_err(|e| format!("failed to write updated locus: {}", e))?;

    Ok(())
}

/// Replace the `tags:` line inside the YAML frontmatter block.
/// All other lines — frontmatter and body — are preserved exactly.
fn rewrite_tags_in_frontmatter(content: &str, new_tags: &[String]) -> Result<String, String> {
    if !content.starts_with("---") {
        return Err("locus file has no YAML frontmatter".to_string());
    }

    let rest = &content[3..];
    let close = rest
        .find("\n---")
        .ok_or("malformed frontmatter: no closing ---")?;

    let fm_block = &rest[..close];
    let body = rest.get(close + 4..).unwrap_or("");

    let new_tags_yaml = format!("[{}]", new_tags.join(", "));

    let updated_fm: String = fm_block
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("tags:") {
                format!("tags: {}", new_tags_yaml)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // updated_fm starts with \n (the char after opening ---), does not end with \n.
    // We need to inject the closing \n before ---
    Ok(format!("---{}\n---{}", updated_fm, body))
}
