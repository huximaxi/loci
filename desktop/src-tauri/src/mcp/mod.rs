// ─── loci MCP server module (1B) ─────────────────────────────────────────────
//
// Implements the Model Context Protocol (MCP) JSON-RPC 2.0 server for Loci.
// Exposes Locus nodes and Rooms as MCP resources, and write tools so AI agents
// (Goose, Continue.dev, Claude Code) can query and extend the knowledge garden.
//
// Cipher gate (non-negotiable):
//   - Server binds to 127.0.0.1 only. Never 0.0.0.0.
//   - THREAT-01: Conversation objects are NEVER exposed. Locus (user-authored) only.
//   - All responses carry `X-Loci-Content-Trust: user-authored`.
//   - Port range validated: 1024–65535.
//
// Entry point from main.rs:
//   start_mcp_server(port, loci_dir) → Result<u16, String>
//   stop_mcp_server()               → Result<(), String>

pub mod resources;
pub mod server;
pub mod tools;
