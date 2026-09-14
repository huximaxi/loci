//! loci-wal — the egress-receipt trust primitive (PUBLIC).
//!
//! An append-only, payload-bytes-free, hash-chained write-ahead log of what left
//! the device, plus an Ed25519-signed proof bundle a third party verifies
//! offline. Prevention is Loci's process-boundary law; this crate is the *proof*.
//!
//! Boundary (Kerckhoffs): everything here is publishable — the format, the
//! verifier, the signing algorithm. Only the private signing key (custody) and
//! the disclosure policy live in the private core.
//!
//! Honest guarantees:
//! - The live WAL hash-chain orders frames and breaks on *piecemeal* edits; it
//!   is NOT keyed tamper-evidence (a whole re-chain or tail edit is not caught
//!   without a key — HMAC live-seal is deferred).
//! - A `ProofBundle` proves "the holder of `signer_pubkey`'s secret signed this
//!   contiguous run." Provenance ("this is *your* palace") needs a pinned key
//!   via `verify_against`.
//! - `payload_hash`/`byte_count` are local-audit-only and never travel in a
//!   bundle (an unsalted content hash is a confirmation oracle).

pub mod frame;
pub mod proof;
pub mod wal;

pub use frame::{ChainError, Chained, EgressClass, Frame, GENESIS};
pub use proof::{ProofBundle, ProofFrame, VerifyError, BUNDLE_VERSION, SIG_ALGORITHM};
pub use wal::{record_egress, Wal};

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn tmp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("loci_wal_{}_{}.jsonl", name, std::process::id()))
    }

    fn mkframe(host: &str) -> Frame {
        Frame {
            seq: 0,
            ts: "2026-07-13T00:00:00Z".into(),
            event_type: "provider_request".into(),
            egress_class: EgressClass::ExternalCloud,
            dest_host: host.into(),
            prev_frame_hash: String::new(),
            payload_hash: Some(Frame::hash_payload(b"the prompt bytes")),
            byte_count: Some(16),
            consent_ref: None,
        }
    }

    /// Build a properly-chained WAL and return the read-back frames.
    fn chained(name: &str, hosts: &[&str]) -> (std::path::PathBuf, Vec<Frame>) {
        let path = tmp(name);
        let _ = std::fs::remove_file(&path);
        let wal = Wal::open(&path);
        for h in hosts {
            wal.append(mkframe(h)).unwrap();
        }
        let frames = wal.read().unwrap();
        (path, frames)
    }

    #[test]
    fn wal_chain_intact_then_tamper_detected() {
        let (path, mut frames) = chained("chain", &["api.anthropic.com"; 3]);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].seq, 0);
        assert_eq!(frames[2].seq, 2);
        assert_eq!(frames[0].prev_frame_hash, GENESIS);
        assert!(Wal::verify_full(&frames).is_ok());
        // Editing an interior frame breaks the next link.
        frames[1].dest_host = "evil.example".into();
        assert!(matches!(Wal::verify_full(&frames), Err(ChainError::BrokenLink(_))));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn proof_roundtrips_and_verifies_against_pinned_key() {
        let (path, frames) = chained("proof", &["a.com", "b.com"]);
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let pk_hex = hex::encode(sk.verifying_key().to_bytes());
        let bundle = ProofBundle::mint(&frames, &sk);
        assert!(bundle.verify().is_ok());
        assert!(bundle.verify_against(&pk_hex).is_ok());
        // Tamper a frame -> chain or signature rejects.
        let mut t = bundle.clone();
        t.frames[0].dest_host = "evil.example".into();
        assert!(t.verify().is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn forged_key_self_verifies_but_fails_provenance() {
        // M5: verify() alone is NOT provenance.
        let (path, frames) = chained("forge", &["a.com", "b.com"]);
        let genuine_pk = hex::encode(SigningKey::from_bytes(&[1u8; 32]).verifying_key().to_bytes());
        let attacker = SigningKey::from_bytes(&[2u8; 32]);
        let forged = ProofBundle::mint(&frames, &attacker);
        assert!(forged.verify().is_ok()); // internally valid for the embedded key
        assert_eq!(forged.verify_against(&genuine_pk), Err(VerifyError::KeyMismatch));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bundle_is_payload_free() {
        let (path, frames) = chained("pii", &["a.com"]);
        let bundle = ProofBundle::mint(&frames, &SigningKey::from_bytes(&[9u8; 32]));
        let json = serde_json::to_string(&bundle).unwrap();
        for banned in ["\"payload\"", "payload_hash", "byte_count"] {
            assert!(!json.contains(banned), "bundle must not carry {banned}");
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bundle_contiguity_enforced() {
        let (path, frames) = chained("range", &["a.com", "b.com", "c.com"]);
        let mut t = ProofBundle::mint(&frames, &SigningKey::from_bytes(&[5u8; 32]));
        t.frames.remove(1); // gap -> range/contiguity/signature all reject
        assert!(t.verify().is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn record_egress_writes_a_classed_frame() {
        let path = tmp("rec");
        let _ = std::fs::remove_file(&path);
        record_egress(&path, "2026-07-14T00:00:00Z".into(), "chat", EgressClass::ExternalCloud, "api.anthropic.com", b"the prompt", None).unwrap();
        record_egress(&path, "2026-07-14T00:01:00Z".into(), "local_inference", EgressClass::Local, "localhost", b"", None).unwrap();
        let frames = Wal::open(&path).read().unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].egress_class, EgressClass::ExternalCloud);
        assert_eq!(frames[1].egress_class, EgressClass::Local);
        assert!(frames[0].payload_hash.is_some());
        assert!(Wal::verify_full(&frames).is_ok());
        let _ = std::fs::remove_file(&path);
    }
}
