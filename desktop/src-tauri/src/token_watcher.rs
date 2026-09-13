//! token_watcher: approximate agent-runtime 5-hour session-window status from
//! local Claude transcripts, plus the rain hand-off.
//!
//! Re-expressed for the desktop door; no shared crate with the CLI's
//! `tokens.rs` (same shape, separate door). Read-only except `fire_rain`,
//! which spawns the user's agent runtime detached and returns.
//!
//! Honesty note: quota is plan-dependent and not readable locally. This
//! reports timing and spend, not allowance. Window reconstruction is
//! approximate (transcripts touched in the last 24h).

use serde::Serialize;
use std::collections::HashSet;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const WINDOW_SECS: u64 = 5 * 3600;
const SCAN_HORIZON_SECS: u64 = 24 * 3600;
const FRESH_SECS: u64 = 15 * 60;
const CLOSING_SECS: u64 = 30 * 60;

#[derive(Serialize, Clone, Default)]
pub struct WindowTokens {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
    pub total: u64,
}

#[derive(Serialize, Clone)]
pub struct WindowStatus {
    pub state: String,
    pub signal: String,
    pub window_start_utc: Option<String>,
    pub window_reset_utc: Option<String>,
    pub minutes_remaining: Option<u64>,
    pub tokens: WindowTokens,
    pub messages: usize,
    pub note: String,
}

#[derive(Serialize, Clone)]
pub struct RainStatus {
    pub plants: usize,
    pub last_rain: Option<String>,
    pub last_rain_waterings: Option<usize>,
}

#[tauri::command]
pub fn read_token_window() -> WindowStatus {
    window_status()
}

#[tauri::command]
pub fn read_rain_status(palace_path: String) -> Result<RainStatus, String> {
    let root = PathBuf::from(&palace_path);
    if !root.is_dir() {
        return Err(format!("not a directory: {palace_path}"));
    }
    let garden = crate::palace_scan_root(&root).join("garden");
    let plants = std::fs::read_dir(garden.join("plants"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
                .count()
        })
        .unwrap_or(0);
    let mut last: Option<(String, PathBuf)> = None;
    if let Ok(rd) = std::fs::read_dir(garden.join(".rain")) {
        for e in rd.filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(date) = name
                .strip_prefix("waterings-")
                .and_then(|s| s.strip_suffix(".json"))
            {
                if last.as_ref().map(|(d, _)| date > d.as_str()).unwrap_or(true) {
                    last = Some((date.to_string(), e.path()));
                }
            }
        }
    }
    let (last_rain, last_rain_waterings) = match last {
        Some((date, path)) => {
            let n = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v.as_array().map(|a| a.len()));
            (Some(date), n)
        }
        None => (None, None),
    };
    Ok(RainStatus {
        plants,
        last_rain,
        last_rain_waterings,
    })
}

/// Spawn the user's agent runtime detached with the rain trigger and return.
/// Never auto-fired: the dashboard button behind this is the human nod.
#[tauri::command]
pub fn fire_rain(palace_path: String) -> Result<String, String> {
    let root = PathBuf::from(&palace_path);
    if !root.is_dir() {
        return Err(format!("not a directory: {palace_path}"));
    }
    let mut child = std::process::Command::new("claude")
        .args(["-p", "rain"])
        .current_dir(&root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("could not launch agent runtime `claude`: {e}. Is it on PATH?"))?;
    // Reap the child when the round ends: a dropped handle is never waited on,
    // and each un-reaped exit would sit as a zombie for the app's lifetime.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok("rain fired; the round runs in the background".to_string())
}

// ── window reconstruction ───────────────────────────────────────────────────

fn window_status() -> WindowStatus {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut entries = scan_entries(now);
    entries.sort_by_key(|e| e.0);

    if entries.is_empty() {
        return WindowStatus {
            state: "unknown".into(),
            signal: "unknown".into(),
            window_start_utc: None,
            window_reset_utc: None,
            minutes_remaining: None,
            tokens: WindowTokens::default(),
            messages: 0,
            note: "no local Claude transcripts found in the scan horizon".into(),
        };
    }

    let mut start = entries[0].0 - (entries[0].0 % 3600);
    let mut end = start + WINDOW_SECS;
    let mut tok = WindowTokens::default();
    let mut count = 0usize;
    for (t, u) in &entries {
        if *t >= end {
            start = t - (t % 3600);
            end = start + WINDOW_SECS;
            tok = WindowTokens::default();
            count = 0;
        }
        tok.input += u.input;
        tok.output += u.output;
        tok.cache_creation += u.cache_creation;
        tok.cache_read += u.cache_read;
        tok.total += u.total;
        count += 1;
    }

    let iso = |t: u64| {
        chrono::DateTime::from_timestamp(t as i64, 0)
            .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_default()
    };
    if now < end {
        let remaining = end - now;
        let elapsed = now.saturating_sub(start);
        let signal = if elapsed <= FRESH_SECS {
            "fresh"
        } else if remaining <= CLOSING_SECS {
            "closing"
        } else {
            "open"
        };
        WindowStatus {
            state: "active".into(),
            signal: signal.into(),
            window_start_utc: Some(iso(start)),
            window_reset_utc: Some(iso(end)),
            minutes_remaining: Some(remaining / 60),
            tokens: tok,
            messages: count,
            note: "timing and spend only; quota is plan-dependent and not readable locally".into(),
        }
    } else {
        WindowStatus {
            state: "fresh".into(),
            signal: "fresh".into(),
            window_start_utc: None,
            window_reset_utc: None,
            minutes_remaining: None,
            tokens: WindowTokens::default(),
            messages: 0,
            note: format!("last window closed {}; the next message opens a fresh one", iso(end)),
        }
    }
}

fn data_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(d) = std::env::var("CLAUDE_CONFIG_DIR") {
        v.push(PathBuf::from(d).join("projects"));
    }
    if let Some(home) = dirs::home_dir() {
        v.push(home.join(".config/claude/projects"));
        v.push(home.join(".claude/projects"));
    }
    v.retain(|d| d.is_dir());
    v
}

/// Above this, a single line is drained and discarded rather than buffered:
/// an adversarial file with one very long "line" (or none at all) must not
/// force an unbounded allocation before the `"usage"` filter below runs.
const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

/// Scan a file line-by-line, invoking `on_line` for each line up to
/// `MAX_LINE_BYTES`. Bounds memory at the line level, not just the file
/// level: `BufRead::lines()` still buffers one arbitrarily long line in
/// full before yielding it, which `read_to_string` avoids for the whole
/// file but not for one hostile unterminated line within it.
fn for_each_bounded_line(f: impl std::io::Read, mut on_line: impl FnMut(&str)) {
    let mut reader = std::io::BufReader::new(f);
    let mut buf: Vec<u8> = Vec::new();
    let mut overflowed = false;
    loop {
        let chunk = match reader.fill_buf() {
            Ok(c) => c,
            Err(_) => break,
        };
        if chunk.is_empty() {
            if !buf.is_empty() && !overflowed {
                on_line(&String::from_utf8_lossy(&buf));
            }
            break;
        }
        match chunk.iter().position(|&b| b == b'\n') {
            Some(pos) => {
                if !overflowed && buf.len() + pos <= MAX_LINE_BYTES {
                    buf.extend_from_slice(&chunk[..pos]);
                    on_line(&String::from_utf8_lossy(&buf));
                }
                reader.consume(pos + 1);
                buf.clear();
                overflowed = false;
            }
            None => {
                if !overflowed && buf.len() + chunk.len() <= MAX_LINE_BYTES {
                    buf.extend_from_slice(chunk);
                } else {
                    overflowed = true;
                }
                let n = chunk.len();
                reader.consume(n);
            }
        }
    }
}

fn scan_entries(now: u64) -> Vec<(u64, WindowTokens)> {
    let horizon = now.saturating_sub(SCAN_HORIZON_SECS);
    let mut entries = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for root in data_dirs() {
        let mut stack = vec![root];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.filter_map(|e| e.ok()) {
                let Ok(ft) = e.file_type() else { continue };
                if ft.is_symlink() {
                    // Never traverse through a symlink: an attacker who can write
                    // into the scanned tree could otherwise point one anywhere on
                    // disk and have it read here as if it were a transcript.
                    continue;
                }
                let p = e.path();
                if ft.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                    continue;
                }
                let fresh_enough = e
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() >= horizon)
                    .unwrap_or(false);
                if !fresh_enough {
                    continue;
                }
                let Ok(f) = std::fs::File::open(&p) else { continue };
                for_each_bounded_line(f, |line| {
                    if !line.contains("\"usage\"") {
                        return;
                    }
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { return };
                    let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) else { return };
                    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) else { return };
                    let t = dt.timestamp().max(0) as u64;
                    if t < horizon {
                        return;
                    }
                    let Some(usage) = v.pointer("/message/usage") else { return };
                    let key = format!(
                        "{}:{}",
                        v.pointer("/message/id").and_then(|x| x.as_str()).unwrap_or(""),
                        v.get("requestId").and_then(|x| x.as_str()).unwrap_or("")
                    );
                    if key != ":" && !seen.insert(key) {
                        return;
                    }
                    let g = |k: &str| usage.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
                    entries.push((
                        t,
                        WindowTokens {
                            input: g("input_tokens"),
                            output: g("output_tokens"),
                            cache_creation: g("cache_creation_input_tokens"),
                            cache_read: g("cache_read_input_tokens"),
                            total: g("input_tokens")
                                + g("output_tokens")
                                + g("cache_creation_input_tokens")
                                + g("cache_read_input_tokens"),
                        },
                    ));
                });
            }
        }
    }
    entries
}

#[cfg(test)]
mod bounded_line_tests {
    use super::*;

    fn scan(content: &[u8]) -> Vec<String> {
        let mut out = Vec::new();
        for_each_bounded_line(std::io::Cursor::new(content.to_vec()), |l| out.push(l.to_string()));
        out
    }

    #[test]
    fn normal_lines_pass_through() {
        assert_eq!(scan(b"a\nbb\nccc\n"), vec!["a", "bb", "ccc"]);
    }

    #[test]
    fn trailing_line_without_newline_is_yielded() {
        assert_eq!(scan(b"a\nb"), vec!["a", "b"]);
    }

    #[test]
    fn empty_file_yields_nothing() {
        assert!(scan(b"").is_empty());
    }

    #[test]
    fn oversized_line_is_dropped_not_buffered() {
        let mut content = vec![b'x'; MAX_LINE_BYTES + 1024];
        content.push(b'\n');
        content.extend_from_slice(b"next\n");
        assert_eq!(scan(&content), vec!["next"]);
    }

    #[test]
    fn oversized_line_at_eof_is_dropped() {
        let content = vec![b'x'; MAX_LINE_BYTES + 1024];
        assert!(scan(&content).is_empty());
    }
}
