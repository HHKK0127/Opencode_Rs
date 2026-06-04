#!/bin/bash

BASE_URL=${1:-http://localhost:8080}

echo "🏥 Health Check: $BASE_URL"
echo ""

# Check main health endpoint
echo "Checking /health..."
HEALTH=$(curl -s -w "\n%{http_code}" "$BASE_URL/health")
HTTP_CODE=$(echo "$HEALTH" | tail -n1)
BODY=$(echo "$HEALTH" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Health endpoint: OK ($HTTP_CODE)"
    echo "Response: $BODY"
else
    echo "❌ Health endpoint: FAILED ($HTTP_CODE)"
    echo "Response: $BODY"
    exit 1
fi

echo ""

# Check database connectivity
echo "Checking /health/db..."
DB_HEALTH=$(curl -s -w "\n%{http_code}" "$BASE_URL/health/db")
DB_HTTP_CODE=$(echo "$DB_HEALTH" | tail -n1)
DB_BODY=$(echo "$DB_HEALTH" | head -n-1)

if [ "$DB_HTTP_CODE" = "200" ]; then
    echo "✅ Database health: OK ($DB_HTTP_CODE)"
    echo "Response: $DB_BODY"
else
    echo "❌ Database health: FAILED ($DB_HTTP_CODE)"
    echo "Response: $DB_BODY"
fi

echo ""
echo "✅ Health check completed"
