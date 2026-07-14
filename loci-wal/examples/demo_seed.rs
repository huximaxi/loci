//! Dev demo (NOT a shipped feature): seed a WAL with sample egress frames and
//! mint a proof bundle, so `loci audit` / `loci wal verify` have real data to
//! read. Real minting uses the core-custodied key, never this throwaway one.
//!
//! Usage: cargo run --example demo_seed -- <wal_path> <bundle_path>

use ed25519_dalek::SigningKey;
use loci_wal::{EgressClass, Frame, ProofBundle, Wal};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wal_path = args.get(1).cloned().unwrap_or_else(|| "/tmp/loci-demo-wal.jsonl".to_string());
    let bundle_path = args.get(2).cloned().unwrap_or_else(|| "/tmp/loci-demo-bundle.json".to_string());
    let _ = std::fs::remove_file(&wal_path);

    let wal = Wal::open(&wal_path);
    let samples = [
        (EgressClass::Local, "localhost", "local_inference", 0u64),
        (EgressClass::ExternalCloud, "api.anthropic.com", "provider_request", 420),
        (EgressClass::ExternalCloud, "api.anthropic.com", "provider_request", 380),
        (EgressClass::ChannelEgress, "zulip.internal", "channel_post", 88),
    ];
    for (i, (class, host, ev, bytes)) in samples.iter().enumerate() {
        wal.append(Frame {
            seq: 0,
            ts: format!("2026-07-13T09:0{i}:00Z"),
            event_type: (*ev).to_string(),
            egress_class: *class,
            dest_host: (*host).to_string(),
            prev_frame_hash: String::new(),
            payload_hash: Some(Frame::hash_payload(b"demo payload")),
            byte_count: Some(*bytes),
            consent_ref: None,
        })
        .unwrap();
    }

    let frames = wal.read().unwrap();
    let sk = SigningKey::from_bytes(&[42u8; 32]);
    let bundle = ProofBundle::mint(&frames, &sk);
    std::fs::write(&bundle_path, serde_json::to_string_pretty(&bundle).unwrap()).unwrap();

    println!("seeded {} frames -> {wal_path}", frames.len());
    println!("minted bundle -> {bundle_path}");
    println!("signer_pubkey {}", bundle.signer_pubkey);
}
