//! The exportable proof bundle: an Ed25519-signed, payload-free envelope a third
//! party re-verifies offline with no external tool.
//!
//! Kerckhoffs: the algorithm + verifier are public (here); only the private
//! `SigningKey` (key custody) lives in the private core and is injected into
//! `mint`. A holder of only this crate can fully verify a bundle's structure,
//! chain, and signature.
//!
//! What `verify()` proves: "the holder of `signer_pubkey`'s private key signed
//! this contiguous, correctly-linked run of frames." It does NOT prove the key
//! is *your* palace's — that is provenance, and it requires a pinned/published
//! key via `verify_against`.

use crate::frame::{hash_basis, verify_links, ChainBasis, ChainError, Chained, EgressClass, Frame};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

pub const SIG_ALGORITHM: &str = "ed25519-raw";
pub const BUNDLE_VERSION: u32 = 1;
/// Domain-separation tag: this key must never sign another message type without
/// its own tag (guards cross-protocol replay when the palace key is reused).
const DOMAIN: &[u8] = b"loci-wal-proof-bundle-v1";

/// A frame as it appears in a bundle: the chain basis only. No payload of any
/// kind, no `payload_hash`, no `byte_count` — only what the chain covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofFrame {
    pub seq: u64,
    pub ts: String,
    pub event_type: String,
    pub egress_class: EgressClass,
    pub dest_host: String,
    pub prev_frame_hash: String,
}

impl From<&Frame> for ProofFrame {
    fn from(f: &Frame) -> Self {
        ProofFrame {
            seq: f.seq,
            ts: f.ts.clone(),
            event_type: f.event_type.clone(),
            egress_class: f.egress_class,
            dest_host: f.dest_host.clone(),
            prev_frame_hash: f.prev_frame_hash.clone(),
        }
    }
}

impl ProofFrame {
    fn basis(&self) -> ChainBasis<'_> {
        ChainBasis {
            seq: self.seq,
            ts: &self.ts,
            event_type: &self.event_type,
            egress_class: self.egress_class,
            dest_host: &self.dest_host,
            prev_frame_hash: &self.prev_frame_hash,
        }
    }
}

impl Chained for ProofFrame {
    fn seq(&self) -> u64 {
        self.seq
    }
    fn prev_hash(&self) -> &str {
        &self.prev_frame_hash
    }
    fn chain_hash(&self) -> String {
        hash_basis(&self.basis())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    pub version: u32,
    pub sig_algorithm: String,
    /// Hex Ed25519 public key the signature verifies against.
    pub signer_pubkey: String,
    /// [first_seq, last_seq] covered.
    pub range: [u64; 2],
    pub frames: Vec<ProofFrame>,
    /// Hex Ed25519 signature over DOMAIN || canonical(bundle sans signature).
    pub signature: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerifyError {
    UnsupportedVersion,
    UnsupportedAlgorithm,
    Empty,
    RangeMismatch,
    Chain(ChainError),
    BadKey,
    BadSig,
    /// Embedded key != the expected (pinned) key.
    KeyMismatch,
    SignatureMismatch,
}

impl From<ChainError> for VerifyError {
    fn from(e: ChainError) -> Self {
        VerifyError::Chain(e)
    }
}

impl ProofBundle {
    /// The exact bytes signed/verified: DOMAIN tag || the whole bundle sans
    /// `signature`. `version` and `sig_algorithm` are bound in (no downgrade).
    fn signable(version: u32, algo: &str, pubkey: &str, range: [u64; 2], frames: &[ProofFrame]) -> Vec<u8> {
        let doc = (version, algo, pubkey, range, frames);
        let mut msg = DOMAIN.to_vec();
        msg.extend_from_slice(&serde_json::to_vec(&doc).expect("signable serializes"));
        msg
    }

    /// Mint a signed bundle over `frames`. The key is injected (custody = core).
    pub fn mint(frames: &[Frame], signing_key: &SigningKey) -> ProofBundle {
        let pframes: Vec<ProofFrame> = frames.iter().map(ProofFrame::from).collect();
        let range = [
            frames.first().map(|f| f.seq).unwrap_or(0),
            frames.last().map(|f| f.seq).unwrap_or(0),
        ];
        let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
        let msg = Self::signable(BUNDLE_VERSION, SIG_ALGORITHM, &pubkey_hex, range, &pframes);
        let sig: Signature = signing_key.sign(&msg);
        ProofBundle {
            version: BUNDLE_VERSION,
            sig_algorithm: SIG_ALGORITHM.to_string(),
            signer_pubkey: pubkey_hex,
            range,
            frames: pframes,
            signature: hex::encode(sig.to_bytes()),
        }
    }

    /// Verify structure + chain + signature against the EMBEDDED key. Proves the
    /// signer holds that key's secret — NOT that the key is your palace's. Use
    /// `verify_against` with a pinned key for provenance.
    pub fn verify(&self) -> Result<(), VerifyError> {
        if self.version != BUNDLE_VERSION {
            return Err(VerifyError::UnsupportedVersion);
        }
        if self.sig_algorithm != SIG_ALGORITHM {
            return Err(VerifyError::UnsupportedAlgorithm);
        }
        if self.frames.is_empty() {
            return Err(VerifyError::Empty);
        }
        let first = self.frames.first().unwrap().seq;
        let last = self.frames.last().unwrap().seq;
        if self.range != [first, last] {
            return Err(VerifyError::RangeMismatch);
        }
        verify_links(&self.frames)?;
        let pk = decode_pubkey(&self.signer_pubkey)?;
        let sig = decode_sig(&self.signature)?;
        let msg = Self::signable(self.version, &self.sig_algorithm, &self.signer_pubkey, self.range, &self.frames);
        pk.verify_strict(&msg, &sig)
            .map_err(|_| VerifyError::SignatureMismatch)
    }

    /// Verify AND require the signer key equals `expected_pubkey_hex`. This is the
    /// provenance check a third party runs against your published/pinned key.
    pub fn verify_against(&self, expected_pubkey_hex: &str) -> Result<(), VerifyError> {
        // Hex is case-insensitive; tolerate surrounding whitespace so a genuine
        // match is never false-rejected. The cryptographic check is in verify().
        if !self
            .signer_pubkey
            .trim()
            .eq_ignore_ascii_case(expected_pubkey_hex.trim())
        {
            return Err(VerifyError::KeyMismatch);
        }
        self.verify()
    }
}

fn decode_pubkey(hex_s: &str) -> Result<VerifyingKey, VerifyError> {
    let b = hex::decode(hex_s).map_err(|_| VerifyError::BadKey)?;
    let arr: [u8; 32] = b.as_slice().try_into().map_err(|_| VerifyError::BadKey)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| VerifyError::BadKey)
}

fn decode_sig(hex_s: &str) -> Result<Signature, VerifyError> {
    let b = hex::decode(hex_s).map_err(|_| VerifyError::BadSig)?;
    let arr: [u8; 64] = b.as_slice().try_into().map_err(|_| VerifyError::BadSig)?;
    Ok(Signature::from_bytes(&arr))
}
