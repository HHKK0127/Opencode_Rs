#!/usr/bin/env bash
# promote.sh — Canary traffic promotion: 10% → 50% → 100%
#
# Usage:
#   ./promote.sh 10    # Phase 1: 1 canary / 9 stable  (10%)
#   ./promote.sh 50    # Phase 2: 5 canary / 5 stable  (50%)
#   ./promote.sh 100   # Phase 3: promote to stable, remove canary

set -euo pipefail

NAMESPACE="opencode"
PHASE="${1:-10}"

check_canary_health() {
    local errors
    errors=$(kubectl -n "$NAMESPACE" get pods -l track=canary \
        -o jsonpath='{.items[*].status.containerStatuses[*].ready}' 2>/dev/null)
    if echo "$errors" | grep -q "false"; then
        echo "❌ Canary pods not ready — aborting"
        exit 1
    fi
    echo "✅ Canary pods healthy"
}

case "$PHASE" in
  10)
    echo "🚀 Phase 1: 10% canary traffic (1 canary / 9 stable)"
    kubectl -n "$NAMESPACE" scale deployment opencode-api --replicas=9
    kubectl -n "$NAMESPACE" scale deployment opencode-api-canary --replicas=1
    check_canary_health
    echo "⏳ Observe for 1-2 hours before promoting to 50%"
    ;;
  50)
    echo "🚀 Phase 2: 50% canary traffic (5 canary / 5 stable)"
    check_canary_health
    kubectl -n "$NAMESPACE" scale deployment opencode-api --replicas=5
    kubectl -n "$NAMESPACE" scale deployment opencode-api-canary --replicas=5
    echo "⏳ Observe for 2-4 hours before promoting to 100%"
    ;;
  100)
    echo "🚀 Phase 3: 100% — promoting canary to stable"
    check_canary_health
    # Update stable image to canary image
    CANARY_IMAGE=$(kubectl -n "$NAMESPACE" get deployment opencode-api-canary \
        -o jsonpath='{.spec.template.spec.containers[0].image}')
    echo "   Updating stable image to: $CANARY_IMAGE"
    kubectl -n "$NAMESPACE" set image deployment/opencode-api "opencode-api=$CANARY_IMAGE"
    kubectl -n "$NAMESPACE" scale deployment opencode-api --replicas=2
    # Wait for stable rollout
    kubectl -n "$NAMESPACE" rollout status deployment/opencode-api --timeout=120s
    # Remove canary
    kubectl -n "$NAMESPACE" scale deployment opencode-api-canary --replicas=0
    echo "✅ Canary promotion complete. Stable deployment updated."
    ;;
  *)
    echo "Usage: $0 <10|50|100>"
    exit 1
    ;;
esac
