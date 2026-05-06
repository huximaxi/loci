#!/usr/bin/env bash
# loci wizard — build script
# v0.2.0 · Hux × Kata
# ─────────────────────────────────────────────────────────────────────────────
# Usage:
#   ./build.sh              — macOS universal .dmg (arm64 + x86_64)
#   ./build.sh --windows    — Windows .exe (cross-compile note: run on Windows)
#   ./build.sh --dev        — dev server only (no bundle)
#
# Output (macOS):
#   src-tauri/target/universal-apple-darwin/release/bundle/dmg/
#     loci wizard_0.2.0_universal.dmg
#
# Output (Windows, run on Windows or CI):
#   src-tauri/target/release/bundle/nsis/
#     loci wizard_0.2.0_x64-setup.exe
# ─────────────────────────────────────────────────────────────────────────────
set -e

VERSION="0.2.0"
APP="loci wizard"

echo ""
echo "  ◆ ${APP} v${VERSION} — build"
echo "  ──────────────────────────────"

# Deps
echo "  ▸ installing dependencies..."
npm install --silent

case "${1:-}" in

  --dev)
    echo "  ▸ starting dev server..."
    npm run tauri:dev
    ;;

  --windows)
    echo "  ▸ building Windows NSIS installer (.exe)..."
    echo "  ╔══════════════════════════════════════════╗"
    echo "  ║  Windows target must run on Windows or   ║"
    echo "  ║  a GitHub Actions windows-latest runner. ║"
    echo "  ║  See: .github/workflows/release.yml      ║"
    echo "  ╚══════════════════════════════════════════╝"
    npm run tauri:build -- --target x86_64-pc-windows-msvc
    echo ""
    echo "  ✦ Windows installer:"
    find src-tauri/target -name "*.exe" -path "*/nsis/*" 2>/dev/null || echo "  (run on Windows to generate)"
    ;;

  *)
    echo "  ▸ building macOS universal binary (arm64 + x86_64)..."
    echo "  ▸ target: universal-apple-darwin"
    echo ""

    # Ensure both targets are installed
    rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true

    npm run tauri:build -- --target universal-apple-darwin

    echo ""
    echo "  ✦ macOS installer:"
    find src-tauri/target -name "*.dmg" 2>/dev/null | head -3
    echo ""
    echo "  ─────────────────────────────────────────"
    echo "  Gatekeeper note: notarization required"
    echo "  for public distribution. See:"
    echo "  https://tauri.app/distribute/sign/apple/"
    echo "  ─────────────────────────────────────────"
    ;;

esac

echo ""
echo "  ◆ build complete."
