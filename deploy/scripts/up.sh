#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

cd "$PROJECT_ROOT"

echo "🚀 Starting OpenCode PoC services..."

# Check if .env exists, if not create template
if [ ! -f .env ]; then
    echo "⚠️  .env file not found. Creating from template..."
    cat > .env << EOF
# OpenCode PoC Environment Variables
ENVIRONMENT=production
JWT_SECRET=your-secret-key-here-change-in-production
EOF
    echo "📝 Created .env file - please update JWT_SECRET"
fi

# Start services
docker-compose up -d

echo "✅ Services started"
echo ""
echo "📍 API available at: http://localhost:8080"
echo "🔍 Health check: curl http://localhost:8080/health"
echo ""
echo "View logs: docker-compose logs -f opencode-api"
