//! The append-only egress log. JSONL on disk, hash-chained per frame.
//!
//! `append` takes an exclusive advisory lock and reads only the tail to find the
//! tip, so it is O(tail) and safe under concurrent writers (Loci runs parallel
//! agents; without the lock two calls would fork the chain). Single-writer per
//! append is enforced by the lock; readers need no lock.

use crate::frame::{verify_links, ChainError, Chained, EgressClass, Frame, GENESIS};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct Wal {
    path: PathBuf,
}

impl Wal {
    pub fn open(path: impl AsRef<Path>) -> Self {
        Wal {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Read all frames in order, skipping unparseable lines. See `read_counted`.
    pub fn read(&self) -> std::io::Result<Vec<Frame>> {
        self.read_counted().map(|(frames, _)| frames)
    }

    /// Read all frames in order and count the lines that did not parse.
    ///
    /// An unparseable line (typically a crash mid-write) is skipped, not treated
    /// as the end of the log: `append` starts on a fresh line after a fragment and
    /// chains to the last *parseable* frame, so frames written after it are still
    /// part of the chain and must still be read. Stopping at the first bad line
    /// would hide every later frame while the chain check still passed. The count
    /// lets a reader say the log is not clean instead of reporting it as complete.
    /// A torn fragment was never acknowledged to its caller, so skipping it leaves
    /// the chain intact; a damaged line that held an acknowledged frame shows up as
    /// a broken link in `verify_full`.
    pub fn read_counted(&self) -> std::io::Result<(Vec<Frame>, usize)> {
        if !self.path.exists() {
            return Ok((Vec::new(), 0));
        }
        let f = File::open(&self.path)?;
        let mut frames = Vec::new();
        let mut unreadable = 0;
        for line in BufReader::new(f).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Frame>(&line) {
                Ok(fr) => frames.push(fr),
                Err(_) => unreadable += 1,
            }
        }
        Ok((frames, unreadable))
    }

    /// Read only the last complete frame (the tip) without parsing the whole file.
    /// Scans the final window; the tip is the last parseable line.
    fn read_tip(path: &Path) -> std::io::Result<Option<Frame>> {
        if !path.exists() {
            return Ok(None);
        }
        let mut f = File::open(path)?;
        let len = f.seek(SeekFrom::End(0))?;
        if len == 0 {
            return Ok(None);
        }
        let window = len.min(65536);
        f.seek(SeekFrom::Start(len - window))?;
        let mut buf = Vec::with_capacity(window as usize);
        f.take(window).read_to_end(&mut buf)?;
        let text = String::from_utf8_lossy(&buf);
        for line in text.lines().rev() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(fr) = serde_json::from_str::<Frame>(line) {
                return Ok(Some(fr));
            }
        }
        Ok(None)
    }

    /// Append an event, chaining it to the tip under an exclusive lock. The WAL
    /// assigns `seq` + `prev_frame_hash`; those two caller fields are overwritten.
    pub fn append(&self, mut frame: Frame) -> std::io::Result<Frame> {
        let f = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&self.path)?;
        f.lock_exclusive()?;
        let result = (|| {
            // A crash can leave the file ending mid-line. Start on a fresh line, or
            // this frame is glued to the fragment and neither can be parsed.
            let len = f.metadata()?.len();
            if len > 0 {
                let mut last = [0u8; 1];
                let mut r = &f;
                r.seek(SeekFrom::Start(len - 1))?;
                r.read_exact(&mut last)?;
                if last[0] != b'\n' {
                    (&f).write_all(b"\n")?;
                }
            }
            let (seq, prev) = match Self::read_tip(&self.path)? {
                Some(tip) => (tip.seq + 1, tip.chain_hash()),
                None => (0, GENESIS.to_string()),
            };
            frame.seq = seq;
            frame.prev_frame_hash = prev;
            let line = serde_json::to_string(&frame)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            writeln!(&f, "{line}")?;
            f.sync_all()?;
            Ok(frame.clone())
        })();
        let _ = FileExt::unlock(&f);
        result
    }

    /// Verify the FULL live log: it must start at `GENESIS` and be a contiguous,
    /// correctly-linked run. An empty log is vacuously intact.
    pub fn verify_full(frames: &[Frame]) -> Result<(), ChainError> {
        if frames.is_empty() {
            return Ok(());
        }
        if frames[0].prev_hash() != GENESIS {
            return Err(ChainError::BrokenLink(frames[0].seq()));
        }
        verify_links(frames)
    }
}

/// Record one egress event: hash the payload for the local-audit anchor (the
/// payload bytes are discarded), stamp the caller-supplied `ts`, and append
/// (chained + lock-serialized). The single call an egress chokepoint makes.
#[allow(clippy::too_many_arguments)]
pub fn record_egress(
    path: impl AsRef<Path>,
    ts: String,
    event_type: &str,
    egress_class: EgressClass,
    dest_host: &str,
    payload: &[u8],
    consent_ref: Option<String>,
) -> std::io::Result<Frame> {
    Wal::open(path).append(Frame {
        seq: 0,
        ts,
        event_type: event_type.to_string(),
        egress_class,
        dest_host: dest_host.to_string(),
        prev_frame_hash: String::new(),
        payload_hash: Some(Frame::hash_payload(payload)),
        byte_count: Some(payload.len() as u64),
        consent_ref,
    })
}
