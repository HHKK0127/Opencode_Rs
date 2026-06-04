#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

echo "🔨 Building OpenCode PoC Docker image..."

cd "$PROJECT_ROOT"

# Get version from git tag or use 'latest'
VERSION=${1:-latest}

# Build Docker image
docker build \
    -t opencode-api:$VERSION \
    -t opencode-api:latest \
    -f Dockerfile \
    .

echo "✅ Docker image built successfully"
echo "   Image: opencode-api:$VERSION"
echo "   Latest: opencode-api:latest"
