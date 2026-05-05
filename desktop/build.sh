#!/usr/bin/env bash
set -e

echo "🏛 Building loci desktop app..."

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build for development
echo "🔨 Building Tauri app..."
npm run tauri:build

echo "✅ Build complete!"
echo "Run 'npm run tauri:dev' to start development server"
