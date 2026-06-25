#!/bin/bash

set -e

VERSION=${1:-v2.0.0}
TRAFFIC_PERCENT=${2:-10}
REGISTRY=${3:-registry.example.com}

echo "=========================================="
echo "🚀 Canary Deployment: ${VERSION}"
echo "   Traffic: ${TRAFFIC_PERCENT}%"
echo "=========================================="

# Step 1: Build image
echo ""
echo "[1/6] Building Docker image..."
docker build -t opencode-api:${VERSION} .
echo "✓ Image built: opencode-api:${VERSION}"

# Step 2: Push to registry (optional)
if [ "${REGISTRY}" != "local" ]; then
    echo ""
    echo "[2/6] Pushing to registry..."
    docker tag opencode-api:${VERSION} ${REGISTRY}/opencode-api:${VERSION}
    docker push ${REGISTRY}/opencode-api:${VERSION}
    echo "✓ Pushed to ${REGISTRY}"
else
    echo ""
    echo "[2/6] Skipping registry push (local mode)"
fi

# Step 3: Deploy canary
echo ""
echo "[3/6] Deploying canary (${TRAFFIC_PERCENT}%)..."
export VERSION=${VERSION}
docker-compose -f docker-compose.prod.yml up -d api-canary
echo "✓ Canary deployed"

# Step 4: Wait and health check
echo ""
echo "[4/6] Health check..."
sleep 10
RETRY_COUNT=0
MAX_RETRIES=5

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/health || echo "000")
    if [ "$STATUS" == "200" ]; then
        echo "✓ Health check passed (status: $STATUS)"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -lt $MAX_RETRIES ]; then
        echo "  Retry $RETRY_COUNT/$MAX_RETRIES (status: $STATUS)..."
        sleep 5
    else
        echo "✗ Health check failed after $MAX_RETRIES retries"
        exit 1
    fi
done

# Step 5: Verify metrics
echo ""
echo "[5/6] Verifying metrics..."
METRIC_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8081/metrics)
if [ "$METRIC_STATUS" == "200" ]; then
    echo "✓ Metrics endpoint active"
else
    echo "⚠ Metrics endpoint returned: $METRIC_STATUS"
fi

# Step 6: Summary
echo ""
echo "[6/6] Deployment summary"
echo "=========================================="
echo "✓ Canary Version: ${VERSION}"
echo "✓ Traffic Allocation: ${TRAFFIC_PERCENT}%"
echo "✓ Health Check: PASSED"
echo ""
echo "📊 Monitor at:"
echo "   API (canary): http://localhost:8081"
echo "   API (stable): http://localhost:8082"
echo "   Traefik: http://localhost:8888"
echo "   Grafana: http://localhost:3000"
echo "   Prometheus: http://localhost:9090"
echo ""
echo "🔄 Next steps:"
echo "   1. Monitor metrics for 15-30 minutes"
echo "   2. Check error rates in Grafana"
echo "   3. Gradually increase traffic:"
echo "      - 50%: docker-compose -f docker-compose.prod.yml up -d api-stable --scale api-stable=1"
echo "      - 100%: docker-compose -f docker-compose.prod.yml stop api-stable-legacy"
echo "=========================================="

exit 0
