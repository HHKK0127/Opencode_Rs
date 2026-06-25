#!/usr/bin/env bash
# canary-status.sh — Show current canary / stable pod distribution and Go/No-Go metrics

set -euo pipefail
NAMESPACE="${NAMESPACE:-opencode}"

echo "═══════════════════════════════════════════════"
echo "  OpenCode Canary Release Status"
echo "═══════════════════════════════════════════════"

# Pod counts
STABLE=$(kubectl -n "$NAMESPACE" get deployment opencode-api \
    -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)
CANARY=$(kubectl -n "$NAMESPACE" get deployment opencode-api-canary \
    -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)
TOTAL=$((STABLE + CANARY))

if [ "$TOTAL" -gt 0 ]; then
    PCT=$((CANARY * 100 / TOTAL))
    echo "  Traffic split: ${STABLE} stable / ${CANARY} canary (${PCT}% canary)"
else
    echo "  No deployments found in namespace '$NAMESPACE'"
fi

echo ""
echo "  Pods:"
kubectl -n "$NAMESPACE" get pods -l app=opencode-api \
    -o custom-columns='NAME:.metadata.name,TRACK:.metadata.labels.track,STATUS:.status.phase,READY:.status.containerStatuses[0].ready' \
    2>/dev/null || echo "  (kubectl not available)"

echo ""
echo "  Health endpoints:"
for POD in $(kubectl -n "$NAMESPACE" get pods -l app=opencode-api -o jsonpath='{.items[*].metadata.name}' 2>/dev/null); do
    READY=$(kubectl -n "$NAMESPACE" exec "$POD" -- \
        wget -qO- http://localhost:8080/api/v1/health/ready 2>/dev/null | grep -o '"ready":[^,}]*' || echo '"ready":unknown')
    echo "    $POD: $READY"
done

echo ""
echo "  Next action:"
if [ "$CANARY" -eq 0 ]; then
    echo "    No canary running. Start with: ./k8s/canary/promote.sh 10"
elif [ "$PCT" -le 10 ]; then
    echo "    Canary at 10%. Promote to 50%: ./k8s/canary/promote.sh 50"
    echo "    Rollback:                       ./k8s/canary/rollback.sh"
elif [ "$PCT" -le 50 ]; then
    echo "    Canary at 50%. Promote to 100%: ./k8s/canary/promote.sh 100"
    echo "    Rollback:                        ./k8s/canary/rollback.sh"
else
    echo "    Canary at 100%. Finalize stable: ./k8s/canary/promote.sh 100"
fi
echo "═══════════════════════════════════════════════"
