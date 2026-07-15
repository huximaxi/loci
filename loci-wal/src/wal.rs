//! The append-only egress log. JSONL on disk, hash-chained per frame.
//!
//! `append` takes an exclusive advisory lock and reads only the tail to find the
//! tip, so it is O(tail) and safe under concurrent writers (Loci runs parallel
//! agents; without the lock two calls would fork the chain). Single-writer per
//! append is enforced by the lock; readers need no lock.

use crate::frame::{verify_links, ChainError, Chained, Frame, GENESIS};
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

    /// Read all frames in order. A trailing unparseable line (crash mid-write) is
    /// tolerated: parsing stops at the first bad line rather than erroring, so a
    /// partial tail cannot brick future appends.
    pub fn read(&self) -> std::io::Result<Vec<Frame>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let f = File::open(&self.path)?;
        let mut frames = Vec::new();
        for line in BufReader::new(f).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Frame>(&line) {
                Ok(fr) => frames.push(fr),
                Err(_) => break,
            }
        }
        Ok(frames)
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
            .append(true)
            .open(&self.path)?;
        f.lock_exclusive()?;
        let result = (|| {
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
