#!/bin/bash

# Wave 4 Day 15 パフォーマンステスト実行スクリプト
# 用途: 4つの k6 テストを順次実行し、結果を収集

set -e

# 設定
BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
REDIS_HOST="${REDIS_HOST:-localhost}"
REDIS_PORT="${REDIS_PORT:-6379}"
TEST_DIR="tests/load"
RESULTS_DIR="test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 色出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Wave 4 Day 15 パフォーマンステスト実行${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "実行日時: $(date)"
echo "API URL: $BASE_URL"
echo "Redis: $REDIS_HOST:$REDIS_PORT"
echo ""

# ステップ 1: 環境チェック
echo -e "${YELLOW}[1/5] 環境チェック...${NC}"

# API ヘルスチェック
if ! curl -s "$BASE_URL/health" | grep -q "healthy"; then
    echo -e "${RED}❌ API ヘルスチェック失敗${NC}"
    echo "以下で確認:"
    echo "  curl $BASE_URL/health"
    exit 1
fi
echo -e "${GREEN}✅ API ヘルスチェック: OK${NC}"

# Redis 接続確認
if ! redis-cli -h $REDIS_HOST -p $REDIS_PORT ping &>/dev/null; then
    echo -e "${RED}❌ Redis 接続失敗${NC}"
    echo "Redis が起動していることを確認:"
    echo "  redis-cli -h $REDIS_HOST -p $REDIS_PORT ping"
    exit 1
fi
echo -e "${GREEN}✅ Redis: OK${NC}"

# k6 確認
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}❌ k6 がインストールされていません${NC}"
    exit 1
fi
echo -e "${GREEN}✅ k6: $(k6 --version)${NC}"

echo ""

# ステップ 2: 結果ディレクトリ作成
echo -e "${YELLOW}[2/5] 結果ディレクトリ準備...${NC}"
mkdir -p "$RESULTS_DIR/$TIMESTAMP"
echo -e "${GREEN}✅ $RESULTS_DIR/$TIMESTAMP${NC}"
echo ""

# ステップ 3: テスト実行
echo -e "${YELLOW}[3/5] パフォーマンステスト実行...${NC}"
echo ""

# Test 1: キャッシュ効率検証
echo -e "${BLUE}=== Test 1: キャッシュ効率検証 ===${NC}"
TEST_START=$(date +%s)
if k6 run "$TEST_DIR/wave4_cache_efficiency.js" \
  --env BASE_URL=$BASE_URL \
  --out json="$RESULTS_DIR/$TIMESTAMP/test1_cache.json" \
  2>&1 | tee "$RESULTS_DIR/$TIMESTAMP/test1_cache.log"; then
    echo -e "${GREEN}✅ Test 1 完了${NC}"
else
    echo -e "${RED}❌ Test 1 失敗${NC}"
    exit 1
fi
TEST_END=$(date +%s)
echo "実行時間: $((TEST_END - TEST_START)) 秒"
echo ""

# Test 2: セッション並行負荷
echo -e "${BLUE}=== Test 2: セッション並行負荷テスト ===${NC}"
TEST_START=$(date +%s)
if k6 run "$TEST_DIR/wave4_session_concurrent.js" \
  --env BASE_URL=$BASE_URL \
  --out json="$RESULTS_DIR/$TIMESTAMP/test2_session.json" \
  2>&1 | tee "$RESULTS_DIR/$TIMESTAMP/test2_session.log"; then
    echo -e "${GREEN}✅ Test 2 完了${NC}"
else
    echo -e "${RED}❌ Test 2 失敗${NC}"
    exit 1
fi
TEST_END=$(date +%s)
echo "実行時間: $((TEST_END - TEST_START)) 秒"
echo ""

# Test 3: Redis統合性能
echo -e "${BLUE}=== Test 3: Redis統合性能テスト ===${NC}"
TEST_START=$(date +%s)
if k6 run "$TEST_DIR/wave4_redis_integration.js" \
  --env BASE_URL=$BASE_URL \
  --out json="$RESULTS_DIR/$TIMESTAMP/test3_redis.json" \
  2>&1 | tee "$RESULTS_DIR/$TIMESTAMP/test3_redis.log"; then
    echo -e "${GREEN}✅ Test 3 完了${NC}"
else
    echo -e "${RED}❌ Test 3 失敗${NC}"
    exit 1
fi
TEST_END=$(date +%s)
echo "実行時間: $((TEST_END - TEST_START)) 秒"
echo ""

# Test 4: E2E統合シナリオ
echo -e "${BLUE}=== Test 4: エンドツーエンド統合シナリオ ===${NC}"
TEST_START=$(date +%s)
if k6 run "$TEST_DIR/wave4_e2e_flow.js" \
  --env BASE_URL=$BASE_URL \
  --out json="$RESULTS_DIR/$TIMESTAMP/test4_e2e.json" \
  2>&1 | tee "$RESULTS_DIR/$TIMESTAMP/test4_e2e.log"; then
    echo -e "${GREEN}✅ Test 4 完了${NC}"
else
    echo -e "${RED}❌ Test 4 失敗${NC}"
    exit 1
fi
TEST_END=$(date +%s)
echo "実行時間: $((TEST_END - TEST_START)) 秒"
echo ""

# ステップ 4: 結果収集
echo -e "${YELLOW}[4/5] テスト結果の集約...${NC}"

# 結果サマリーファイル生成
cat > "$RESULTS_DIR/$TIMESTAMP/RESULTS_SUMMARY.md" << 'EOF'
# Wave 4 Day 15 パフォーマンステスト 結果

**実施日**: $(date)

## テスト概要
- **対象**: Wave 4 Redis キャッシング + セッション管理
- **テストスイート**: 4 Tests
- **実行環境**: $(uname -a)

## Test 1: キャッシュ効率検証

### 成功基準
- [ ] キャッシュヒット率 ≥ 85%
- [ ] ヒット時レイテンシ < 1ms
- [ ] ミス時レイテンシ < 50ms

### 結果
- ヒット率: ____%
- 平均レイテンシ: ___ms
- 判定: [ ] PASS [ ] FAIL

---

## Test 2: セッション並行負荷

### 成功基準
- [ ] セッション作成レイテンシ < 10ms
- [ ] セッション検証レイテンシ < 2ms
- [ ] 同時セッション 10,000+ 対応
- [ ] 成功率 > 99.5%

### 結果
- 最大 VU: ____
- 平均レイテンシ: ___ms
- 成功率: ____%
- 判定: [ ] PASS [ ] FAIL

---

## Test 3: Redis統合性能

### 成功基準
- [ ] p95 レイテンシ < 50ms
- [ ] Redis コマンドレイテンシ < 5ms
- [ ] エラー率 < 0.1%

### 結果
- p95: ___ms
- p99: ___ms
- エラー率: ___%
- 判定: [ ] PASS [ ] FAIL

---

## Test 4: E2E統合シナリオ

### 成功基準
- [ ] 総フロー時間 < 500ms
- [ ] 各ステップ p95 < 50ms
- [ ] フロー完了率 ≥ 99%

### 結果
- 平均フロー時間: ___ms
- 成功率: ____%
- 判定: [ ] PASS [ ] FAIL

---

## 総合判定

| 基準 | 結果 |
|------|------|
| MUST 全達成 | [ ] GO [ ] NO-GO |
| SHOULD 達成 | [ ] ✅ [ ] ⚠️ |

### 判定内容
- [ ] **GO**: 全 MUST 基準達成 → Wave 5 本番化準備へ
- [ ] **Conditional GO**: MUST 達成、SHOULD 未達 → Wave 5 で最適化
- [ ] **NO-GO**: MUST 未達成 → 追加チューニング必要

---

## 詳細ログ
- test1_cache.json
- test2_session.json
- test3_redis.json
- test4_e2e.json

EOF

echo -e "${GREEN}✅ 結果サマリー作成: $RESULTS_DIR/$TIMESTAMP/RESULTS_SUMMARY.md${NC}"
echo ""

# ステップ 5: 最終報告
echo -e "${YELLOW}[5/5] 最終報告...${NC}"
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Wave 4 Day 15 パフォーマンステスト完了${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "📊 結果ディレクトリ: $RESULTS_DIR/$TIMESTAMP"
echo ""
echo "📈 結果確認方法:"
echo "  1. サマリーを確認:"
echo "     cat $RESULTS_DIR/$TIMESTAMP/RESULTS_SUMMARY.md"
echo ""
echo "  2. 詳細ログを確認:"
echo "     cat $RESULTS_DIR/$TIMESTAMP/test1_cache.log"
echo "     cat $RESULTS_DIR/$TIMESTAMP/test2_session.log"
echo "     cat $RESULTS_DIR/$TIMESTAMP/test3_redis.log"
echo "     cat $RESULTS_DIR/$TIMESTAMP/test4_e2e.log"
echo ""
echo "  3. JSON 結果を分析:"
echo "     cat $RESULTS_DIR/$TIMESTAMP/test1_cache.json | jq '.metrics'"
echo ""
echo "📝 次のステップ:"
echo "  - LOAD_TEST_RESULTS_WAVE4.md に結果を記載"
echo "  - 成功基準と照合"
echo "  - Go/No-Go 判定"
echo "  - Wave 5 本番化準備へ進行"
echo ""
