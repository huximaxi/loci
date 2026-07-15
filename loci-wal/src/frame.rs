//! The egress WAL frame + the hash-chain machinery.
//!
//! A frame records *that* a call left the device and *where to*, never the
//! content (payload-bytes-free). The chain is computed over a fixed **basis**
//! (`ChainBasis`) — exactly the fields that also travel in an exported proof
//! bundle — so a bundle's chain is reconstructible offline by a third party.
//!
//! Honest scope of the chain (no key): it orders frames and makes *piecemeal*
//! edits (mid-history mutation, reordering, a dropped interior frame) break
//! linkage. It does NOT, on its own, detect a tail-frame edit, tail truncation,
//! or a wholesale re-chain from genesis — nothing anchors the live log without
//! a key. Keyed live-seal (HMAC) is deferred; the Ed25519 proof bundle is the
//! cryptographic tamper-evidence, and only at export.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// How far a call travels. The single predicate every gate + receipt consults.
/// (Public here; the class-to-disclosure policy lives in the private core.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EgressClass {
    /// Never leaves the device (e.g. a local Ollama call).
    Local,
    /// A cloud LLM / external API call.
    ExternalCloud,
    /// An outbound message to a channel.
    ChannelEgress,
    /// A profile / identity write to a remote.
    ProfileWrite,
}

pub const GENESIS: &str = "genesis";

/// The fields that are hash-chained AND carried in an exported bundle. The chain
/// hashes exactly these bytes, so a bundle's internal chain verifies offline.
/// Content anchors (`payload_hash`, `byte_count`) are deliberately NOT here: an
/// unsalted hash of a low-entropy payload is a content-confirmation oracle, so
/// they stay in the live log for the operator's own audit and never travel.
#[derive(Serialize)]
pub(crate) struct ChainBasis<'a> {
    pub seq: u64,
    pub ts: &'a str,
    pub event_type: &'a str,
    pub egress_class: EgressClass,
    pub dest_host: &'a str,
    pub prev_frame_hash: &'a str,
}

pub(crate) fn hash_basis(b: &ChainBasis) -> String {
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(b).expect("chain basis serializes"));
    hex::encode(h.finalize())
}

/// One egress event in the live WAL. Payload-bytes-free by construction.
/// `payload_hash`/`byte_count` are LOCAL-AUDIT-ONLY: excluded from both the
/// chain and any exported bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub seq: u64,
    /// Caller-supplied ISO-8601 timestamp (no clock dependency in the primitive).
    pub ts: String,
    pub event_type: String,
    pub egress_class: EgressClass,
    pub dest_host: String,
    /// Hex hash of the previous frame's basis, or `GENESIS` for the first.
    pub prev_frame_hash: String,
    /// LOCAL-AUDIT-ONLY. Hex SHA-256 of the payload, for the operator's own eyes.
    /// Never chained, never exported (oracle risk). The payload itself is never kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    /// LOCAL-AUDIT-ONLY. Never chained, never exported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_ref: Option<String>,
}

impl Frame {
    pub(crate) fn basis(&self) -> ChainBasis<'_> {
        ChainBasis {
            seq: self.seq,
            ts: &self.ts,
            event_type: &self.event_type,
            egress_class: self.egress_class,
            dest_host: &self.dest_host,
            prev_frame_hash: &self.prev_frame_hash,
        }
    }

    /// Hash over the chain basis only (never the local-audit extras).
    pub fn chain_hash(&self) -> String {
        hash_basis(&self.basis())
    }

    /// Hash a payload for the local-audit `payload_hash`. The bytes are discarded.
    pub fn hash_payload(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        hex::encode(h.finalize())
    }
}

/// Anything that participates in the hash chain (a live `Frame` or a bundle `ProofFrame`).
pub trait Chained {
    fn seq(&self) -> u64;
    fn prev_hash(&self) -> &str;
    fn chain_hash(&self) -> String;
}

impl Chained for Frame {
    fn seq(&self) -> u64 {
        self.seq
    }
    fn prev_hash(&self) -> &str {
        &self.prev_frame_hash
    }
    fn chain_hash(&self) -> String {
        Frame::chain_hash(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChainError {
    /// No frames to check.
    Empty,
    /// Linkage broke: this frame's `prev_frame_hash` != predecessor's `chain_hash`.
    BrokenLink(u64),
    /// Seqs are not contiguous (a frame is missing or duplicated); first offender.
    NonContiguous(u64),
}

/// Verify internal linkage of a contiguous run: each frame's `prev_frame_hash`
/// equals the predecessor's `chain_hash`, and seqs increment by 1. Does NOT
/// anchor to genesis (a bundle is a sub-range), so it proves the run is
/// self-consistent + correctly ordered — not that the whole history is intact.
pub fn verify_links<T: Chained>(frames: &[T]) -> Result<(), ChainError> {
    if frames.is_empty() {
        return Err(ChainError::Empty);
    }
    for w in frames.windows(2) {
        if w[1].seq() != w[0].seq() + 1 {
            return Err(ChainError::NonContiguous(w[1].seq()));
        }
        if w[1].prev_hash() != w[0].chain_hash() {
            return Err(ChainError::BrokenLink(w[1].seq()));
        }
    }
    Ok(())
}
