//! tokens: approximate Claude Code 5-hour session-window status from local
//! transcripts. Read-only: scans `~/.claude/projects/**/*.jsonl` (plus
//! `$CLAUDE_CONFIG_DIR` and `~/.config/claude`) for usage entries and
//! reconstructs the current rolling window. No network.
//!
//! Honesty note: the provider's actual quota is plan-dependent and not
//! readable locally. This module reports timing and spend, not allowance.
//! Window reconstruction is approximate: only transcripts touched in the
//! last 24h are scanned, which is enough to place the current window but
//! not to audit history.

use serde::Serialize;
use std::collections::HashSet;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const WINDOW_SECS: u64 = 5 * 3600;
const SCAN_HORIZON_SECS: u64 = 24 * 3600;
/// Signal thresholds: a window is "fresh" for its first 15 minutes
/// (max headroom) and "closing" in its last 30 (use it or lose it).
const FRESH_SECS: u64 = 15 * 60;
const CLOSING_SECS: u64 = 30 * 60;

#[derive(Serialize, Clone, Default)]
pub struct Tokens {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
    pub total: u64,
}

#[derive(Serialize, Clone)]
pub struct WindowStatus {
    /// "active" (inside a window), "fresh" (no active window; the next
    /// message opens one), or "unknown" (no local transcript data found).
    pub state: String,
    /// "fresh" | "open" | "closing" | "unknown": the rain signal.
    pub signal: String,
    pub window_start_utc: Option<String>,
    pub window_reset_utc: Option<String>,
    pub minutes_remaining: Option<u64>,
    pub tokens: Tokens,
    pub messages: usize,
    pub files_scanned: usize,
    pub note: String,
}

pub fn window_status() -> WindowStatus {
    let now = epoch_now();
    let (mut entries, files_scanned) = scan_entries(now);
    entries.sort_by_key(|e| e.0);

    if entries.is_empty() {
        return WindowStatus {
            state: "unknown".into(),
            signal: "unknown".into(),
            window_start_utc: None,
            window_reset_utc: None,
            minutes_remaining: None,
            tokens: Tokens::default(),
            messages: 0,
            files_scanned,
            note: "no local Claude transcripts found in the scan horizon".into(),
        };
    }

    // Rolling 5h blocks: a block opens at the first activity after the
    // previous block ends, floored to the hour (UTC).
    let mut start = floor_hour(entries[0].0);
    let mut end = start + WINDOW_SECS;
    let mut tok = Tokens::default();
    let mut count = 0usize;
    for (t, u) in &entries {
        if *t >= end {
            start = floor_hour(*t);
            end = start + WINDOW_SECS;
            tok = Tokens::default();
            count = 0;
        }
        tok.input += u.input;
        tok.output += u.output;
        tok.cache_creation += u.cache_creation;
        tok.cache_read += u.cache_read;
        tok.total += u.total;
        count += 1;
    }

    let note = "timing and spend only; quota is plan-dependent and not readable locally".into();
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
            window_start_utc: Some(epoch_to_iso(start)),
            window_reset_utc: Some(epoch_to_iso(end)),
            minutes_remaining: Some(remaining / 60),
            tokens: tok,
            messages: count,
            files_scanned,
            note,
        }
    } else {
        WindowStatus {
            state: "fresh".into(),
            signal: "fresh".into(),
            window_start_utc: None,
            window_reset_utc: None,
            minutes_remaining: None,
            tokens: Tokens::default(),
            messages: 0,
            files_scanned,
            note: format!("last window closed {}; the next message opens a fresh one", epoch_to_iso(end)),
        }
    }
}

// ── scanning ────────────────────────────────────────────────────────────────

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs_v = Vec::new();
    if let Ok(d) = std::env::var("CLAUDE_CONFIG_DIR") {
        dirs_v.push(PathBuf::from(d).join("projects"));
    }
    if let Some(home) = dirs::home_dir() {
        dirs_v.push(home.join(".config/claude/projects"));
        dirs_v.push(home.join(".claude/projects"));
    }
    dirs_v.retain(|d| d.is_dir());
    dirs_v
}

fn scan_entries(now: u64) -> (Vec<(u64, Tokens)>, usize) {
    let horizon = now.saturating_sub(SCAN_HORIZON_SECS);
    let mut entries = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut files = 0usize;

    for root in data_dirs() {
        let mut stack = vec![root];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.filter_map(|e| e.ok()) {
                let p = e.path();
                if p.is_dir() {
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
                files += 1;
                // Stream line-by-line: transcripts can run to hundreds of MB,
                // and read_to_string would hold a whole file in memory per scan.
                let Ok(f) = std::fs::File::open(&p) else { continue };
                for line in std::io::BufReader::new(f).lines() {
                    let Ok(line) = line else { break };
                    if !line.contains("\"usage\"") {
                        continue;
                    }
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
                    let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) else { continue };
                    let Some(t) = parse_rfc3339(ts) else { continue };
                    if t < horizon {
                        continue;
                    }
                    let Some(usage) = v.pointer("/message/usage") else { continue };
                    let key = format!(
                        "{}:{}",
                        v.pointer("/message/id").and_then(|x| x.as_str()).unwrap_or(""),
                        v.get("requestId").and_then(|x| x.as_str()).unwrap_or("")
                    );
                    if key != ":" && !seen.insert(key) {
                        continue;
                    }
                    let g = |k: &str| usage.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
                    let tok = Tokens {
                        input: g("input_tokens"),
                        output: g("output_tokens"),
                        cache_creation: g("cache_creation_input_tokens"),
                        cache_read: g("cache_read_input_tokens"),
                        total: g("input_tokens")
                            + g("output_tokens")
                            + g("cache_creation_input_tokens")
                            + g("cache_read_input_tokens"),
                    };
                    entries.push((t, tok));
                }
            }
        }
    }
    (entries, files)
}

// ── time, stdlib-only ───────────────────────────────────────────────────────

pub fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn floor_hour(t: u64) -> u64 {
    t - (t % 3600)
}

/// Parse RFC3339 ("2026-07-02T11:51:03.123Z" or with ±HH:MM offset) to epoch seconds.
fn parse_rfc3339(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() < 20 {
        return None;
    }
    let num = |r: std::ops::Range<usize>| -> Option<i64> { s.get(r)?.parse().ok() };
    let (y, mo, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (h, mi, sec) = (num(11..13)?, num(14..16)?, num(17..19)?);
    // Skip fractional seconds, then read the offset.
    let mut i = 19;
    if b.get(19) == Some(&b'.') {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    let offset_secs: i64 = match b.get(i) {
        Some(&b'Z') | Some(&b'z') => 0,
        Some(&b'+') | Some(&b'-') => {
            let sign = if b[i] == b'+' { 1 } else { -1 };
            let oh: i64 = s.get(i + 1..i + 3)?.parse().ok()?;
            let om: i64 = s.get(i + 4..i + 6)?.parse().ok()?;
            sign * (oh * 3600 + om * 60)
        }
        _ => return None,
    };
    let days = days_from_civil(y, mo, d);
    let secs = days * 86400 + h * 3600 + mi * 60 + sec - offset_secs;
    u64::try_from(secs).ok()
}

/// Howard Hinnant's days-from-civil: days since 1970-01-01 for a proleptic
/// Gregorian date.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn epoch_to_iso(t: u64) -> String {
    // Inverse civil conversion, UTC.
    let days = (t / 86400) as i64;
    let rem = t % 86400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}
