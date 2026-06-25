#!/usr/bin/env bash
# rollback.sh — Immediately roll back canary deployment
#
# Usage:
#   ./rollback.sh           # rollback canary to 0 replicas
#   ./rollback.sh stable    # also rollback stable to previous revision

set -euo pipefail

NAMESPACE="opencode"
TARGET="${1:-canary}"

echo "🔄 Rolling back — target: $TARGET"

case "$TARGET" in
  canary)
    echo "   Scaling canary to 0..."
    kubectl -n "$NAMESPACE" scale deployment opencode-api-canary --replicas=0
    echo "   Restoring stable to 2 replicas..."
    kubectl -n "$NAMESPACE" scale deployment opencode-api --replicas=2
    kubectl -n "$NAMESPACE" rollout status deployment/opencode-api --timeout=60s
    echo "✅ Canary rolled back. 100% traffic on stable."
    ;;
  stable)
    echo "   Rolling back stable deployment to previous revision..."
    kubectl -n "$NAMESPACE" rollout undo deployment/opencode-api
    kubectl -n "$NAMESPACE" scale deployment opencode-api-canary --replicas=0
    kubectl -n "$NAMESPACE" rollout status deployment/opencode-api --timeout=120s
    echo "✅ Stable rolled back to previous revision."
    ;;
  *)
    echo "Usage: $0 [canary|stable]"
    exit 1
    ;;
esac

# Print current state
echo ""
echo "Current pod state:"
kubectl -n "$NAMESPACE" get pods -l app=opencode-api -o wide
