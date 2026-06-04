#!/bin/bash
echo "=== Wave 2 本番デプロイ前環境確認 ==="
echo "実行時刻: $(date)"
echo ""

# Docker確認
echo "□ Docker動作確認..."
if docker ps > /dev/null 2>&1; then
    echo "  ✅ Docker動作中"
    docker --version | sed 's/^/     /'
else
    echo "  ❌ Docker未起動/未インストール"
fi
echo ""

# ディスク容量
echo "□ ディスク容量 (/c/Drive)..."
if [ -d "/c/Drive" ]; then
    DF_OUTPUT=$(df -h /c/Drive | tail -1)
    USED=$(echo $DF_OUTPUT | awk '{print $3}')
    TOTAL=$(echo $DF_OUTPUT | awk '{print $2}')
    PERCENT=$(echo $DF_OUTPUT | awk '{print $5}')
    echo "  使用量: $USED / 合計: $TOTAL (使用率: $PERCENT)"
    if [ "${PERCENT%\%}" -gt 80 ]; then
        echo "  ⚠️ 警告: ディスク使用率が80%以上"
    else
        echo "  ✅ ディスク容量OK"
    fi
else
    echo "  ❌ /c/Drive未検出"
fi
echo ""

# メモリ
echo "□ メモリ..."
if command -v free &> /dev/null; then
    free -h | grep Mem | awk '{printf "  使用: %s / 合計: %s\n", $3, $2}'
    echo "  ✅ メモリ確認済"
else
    echo "  ⚠️ メモリ情報取得不可（Linux環境でのみ可能）"
fi
echo ""

# /dataディレクトリ
echo "□ /data ディレクトリ..."
if [ -d "/data" ]; then
    echo "  ✅ /data 存在"
    ls -lad /data | awk '{printf "     所有者: %s:%s, パーミッション: %s\n", $3, $4, $1}'
else
    echo "  ⚠️ /data 未作成"
    echo "     推奨: mkdir -p /data"
fi
echo ""

# ポート確認
echo "□ ポート使用状況..."
PORTS=(8080 8081 8082 3000 9090)
for port in "${PORTS[@]}"; do
    if netstat -tuln 2>/dev/null | grep -q ":$port " || ss -tlnp 2>/dev/null | grep -q ":$port "; then
        echo "  ⚠️ ポート $port: 使用中"
    else
        echo "  ✅ ポート $port: 空き"
    fi
done
echo ""

# 環境変数ファイル
echo "□ 環境変数ファイル確認..."
if [ -f ".env.production" ]; then
    echo "  ✅ .env.production 存在"
    if grep -q "^JWT_SECRET=" .env.production; then
        echo "  ✅ JWT_SECRET: 設定済"
    else
        echo "  ❌ JWT_SECRET: 未設定"
    fi
    if grep -q "^DATABASE_URL=" .env.production; then
        echo "  ✅ DATABASE_URL: 設定済"
    else
        echo "  ⚠️ DATABASE_URL: 未設定"
    fi
else
    echo "  ❌ .env.production: 未作成"
fi
echo ""

# 設定ファイル
echo "□ 設定ファイル確認..."
if [ -f "config/production.toml" ]; then
    echo "  ✅ config/production.toml 存在"
else
    echo "  ❌ config/production.toml 未作成"
fi
echo ""

# Dockerイメージ
echo "□ Dockerイメージ確認..."
if docker images 2>/dev/null | grep -q "opencode-api"; then
    LATEST_TAG=$(docker images --format "{{.Tag}}" opencode-api 2>/dev/null | head -1)
    echo "  ✅ opencode-api イメージ存在 (最新: $LATEST_TAG)"
    if docker images 2>/dev/null | grep "opencode-api" | grep -q "v2.0.0"; then
        echo "  ✅ v2.0.0 イメージ: 存在"
    else
        echo "  ⚠️ v2.0.0 イメージ: 未ビルド（要 cargo build --release + docker build）"
    fi
else
    echo "  ⚠️ opencode-api イメージ: 未ビルド"
fi
echo ""

# Docker Compose
echo "□ Docker Compose ファイル..."
if [ -f "docker-compose.prod.yml" ]; then
    echo "  ✅ docker-compose.prod.yml 存在"
else
    echo "  ❌ docker-compose.prod.yml 未作成"
fi
if [ -f "docker-compose.monitoring.yml" ]; then
    echo "  ✅ docker-compose.monitoring.yml 存在"
else
    echo "  ⚠️ docker-compose.monitoring.yml 未作成"
fi
echo ""

# テストスイート
echo "□ テストスイート確認..."
TEST_FILES=(
    "tests/e2e_production_readiness.rs"
    "tests/performance_test.rs"
    "tests/load_test.rs"
)
for test_file in "${TEST_FILES[@]}"; do
    if [ -f "$test_file" ]; then
        echo "  ✅ $test_file"
    else
        echo "  ❌ $test_file 未作成"
    fi
done
echo ""

# Cargoプロジェクト
echo "□ Cargoプロジェクト確認..."
if [ -f "Cargo.toml" ]; then
    echo "  ✅ Cargo.toml 存在"
    BINARY_PATH="target/release/opencode_poc"
    if [ -f "$BINARY_PATH" ] || [ -f "${BINARY_PATH}.exe" ]; then
        echo "  ✅ リリースバイナリ: ビルド済"
    else
        echo "  ⚠️ リリースバイナリ: 未ビルド（要 cargo build --release）"
    fi
else
    echo "  ❌ Cargo.toml 未検出"
fi
echo ""

echo "=== 確認完了 ==="
echo ""
echo "📋 確認結果:"
echo "  ✅ = OK（デプロイ可能）"
echo "  ⚠️  = 警告（修正推奨、デプロイは可能）"
echo "  ❌ = エラー（修正必須、デプロイ不可）"
