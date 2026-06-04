# LOAD_TEST_PLAN.md

## Wave 2 Day 5 ロードテスト計画書

**実行予定日**: 2026-05-30  
**テスト期間**: 40分  
**テストツール**: k6 (Grafana Labs)  
**テスト環境**: Production-like (Docker Compose)

---

## 目次

1. [テスト概要](#1-テスト概要)
2. [テストシナリオ](#2-テストシナリオ)
3. [成功基準](#3-成功基準)
4. [実行手順](#4-実行手順)
5. [期待結果](#5-期待結果)
6. [分析・アクション](#6-分析アクション)

---

## 1. テスト概要

### 目的

Wave 2 Day 4 の実装（DBインデックス化、Prometheusメトリクス）が本番環境での性能要件を満たしているか検証し、**本番デプロイの Go/No-Go 判定**を実施する。

### テストゴール

```
✅ p95 レイテンシ: < 100ms
✅ エラー率: < 1%
✅ ピークスループット: > 500 req/s
✅ メモリ安定性: < 200MB
```

### テスト構成

```
期間: 40分（合計）
VU (Virtual Users): 10 → 100 → 0 段階的変化
総リクエスト数: 25,000+ リクエスト
```

---

## 2. テストシナリオ

### Phase 1: Warmup（5分）

**目的**: システムの初期化、メトリクス収集開始

**VU 推移**: 0 → 10 (5分間)

**負荷レベル**: 軽（50-60 req/s）

**テスト内容**:
- ヘルスチェック（/health）
- 認証テスト（POST /api/v1/auth/login）
- 基本的なファイルリスト取得

**期待結果**:
```
期待 p95: 20-30ms
期待エラー率: 0%
期待スループット: 50-60 req/s
```

---

### Phase 2: Standard Load（15分）

**目的**: 通常負荷での安定性確認

**VU 推移**:
- Ramp up: 10 → 50 VU (5分間)
- Hold: 50 VU (10分間)

**負荷レベル**: 中（150-300 req/s）

**テスト内容**:
- ファイルリスト取得（pagination 含む）
- メトリクス取得（/api/v1/metrics）
- 混合ワークロード

**期待結果**:
```
期待 p95: 40-60ms
期待エラー率: < 0.5%
期待スループット: 150-300 req/s
```

---

### Phase 3: Peak Load（10分）

**目的**: ピーク負荷での性能確認

**VU 推移**:
- Ramp up: 50 → 100 VU (5分間)
- Hold: 100 VU (5分間)

**負荷レベル**: 高（400-500 req/s）

**テスト内容**:
- 全エンドポイント（auth, files, health, metrics）
- 高並行度での一貫性確認

**期待結果**:
```
期待 p95: 60-90ms
期待 p99: < 200ms
期待エラー率: < 1%
期待スループット: 400-500 req/s ← GO判定の鍵
```

---

### Phase 4: Cooldown（5分）

**目的**: システムのリカバリ確認

**VU 推移**: 100 → 0 VU (5分間)

**負荷レベル**: 段階的低下

**テスト内容**:
- ダウンスケール時の安定性
- メモリリークがないことの確認

**期待結果**:
```
期待 p95: 60-90ms（ピーク時点）→ 30-40ms（終了時）
メモリ: > 200MB → < 150MB（回復確認）
```

---

## 3. 成功基準

### 絶対基準（MUST）

| メトリクス | 目標値 | 判定 |
|-----------|--------|------|
| **p95 レイテンシ** | < 100ms | ✅/❌ |
| **エラー率** | < 1% | ✅/❌ |
| **ピークスループット** | > 500 req/s | ✅/❌ |

**判定**: 全て ✅ で **GO → 本番デプロイ進行**

### 重要基準（SHOULD）

| メトリクス | 目標値 | 判定 |
|-----------|--------|------|
| **p99 レイテンシ** | < 200ms | ✅/⚠️ |
| **ピーク CPU** | < 70% | ✅/⚠️ |
| **ピークメモリ** | < 200MB | ✅/⚠️ |

**判定**: 1つ ⚠️ で追加チューニング、2つ以上で **NO-GO**

---

## 4. 実行手順

### Step 1: 事前準備（デプロイ前）

```bash
# 1. k6 インストール確認
k6 --version
# expected: k6 v0.45.0+

# 2. テストスクリプト確認
ls -lh tests/load/wave2_day5_load.js

# 3. サーバー起動（別ターミナル）
cargo run --release
# expected: Server listening on 127.0.0.1:8080

# 4. ヘルスチェック確認
curl http://localhost:8080/health
# expected: {"status":"healthy",...}
```

### Step 2: ロードテスト実行

```bash
# テスト実行（40分間）
k6 run tests/load/wave2_day5_load.js \
  --vus 10 \
  --duration 40m \
  --out json=load-test-results.json

# 並列実行オプション（CloudレベルでのテストならUI付き）
k6 cloud tests/load/wave2_day5_load.js
```

### Step 3: リアルタイム監視

**別ウィンドウで監視**:

```bash
# メトリクスエンドポイント監視（5秒ごと）
watch -n 5 'curl -s http://localhost:8080/api/v1/metrics | \
  grep -E "http_requests_total|http_request_duration_seconds|active_connections"'

# Prometheus ダッシュボード確認
# http://localhost:9090 (if running)

# Grafana ダッシュボード確認
# http://localhost:3000 (if running)
```

### Step 4: テスト完了後の分析

```bash
# 結果ファイル生成（自動）
ls -lh load-test-results.json

# 結果の簡易分析
k6 run tests/load/wave2_day5_load.js --summary-export=summary.json
```

---

## 5. 期待結果

### Warmup Phase (5分, 0→10 VU)

```
Requests:          3,000
Pass Rate:         100%
Error Rate:        0%
p50 Latency:       12ms
p95 Latency:       25ms
p99 Latency:       35ms
Min Latency:       2ms
Max Latency:       150ms
Throughput:        50-60 req/s
CPU Usage:         5-10%
Memory Usage:      80-100MB
```

### Standard Load Phase (15分, 10→50 VU)

```
Requests:          10,500
Pass Rate:         99.5%
Error Rate:        < 0.5%
p50 Latency:       35ms
p95 Latency:       50ms     ← Target: < 100ms ✅
p99 Latency:       80ms
Throughput:        150-300 req/s
CPU Usage:         20-35%
Memory Usage:      120-150MB
```

### Peak Load Phase (10分, 50→100 VU)

```
Requests:          8,000
Pass Rate:         99%
Error Rate:        < 1%     ← Target: < 1% ✅
p50 Latency:       55ms
p95 Latency:       75ms     ← Target: < 100ms ✅
p99 Latency:       150ms    ← Target: < 200ms ✅
Min Latency:       1ms
Max Latency:       350ms
Throughput:        500+ req/s  ← Target: > 500 req/s ✅
CPU Usage:         55-70%   ← Target: < 70% ✅
Memory Usage:      180-200MB  ← Target: < 200MB ✅
```

### Cooldown Phase (5分, 100→0 VU)

```
Requests:          3,500
p95 Latency:       30-50ms （段階的に低下）
Memory Recovery:   200MB → 150MB（正常な回復）
```

### 全体統計

```
Total Requests:    25,000
Total Duration:    40 minutes
Pass Rate:         99.2%
Error Rate:        0.8%      ← < 1% で合格 ✅
Avg Throughput:    625 req/s ← > 500 req/s で合格 ✅
p95 Overall:       60ms      ← < 100ms で合格 ✅
```

---

## 6. 分析・アクション

### シナリオ A: 全目標達成（GO判定）

```
✅ p95 < 100ms
✅ Error rate < 1%
✅ Throughput > 500 req/s

アクション:
  → 直ちに Phase 1 カナリアリリース開始
  → チーム通知：Go Decision
  → LOAD_TEST_RESULTS.md 作成
```

### シナリオ B: p95は達成、エラー率が1-2%（条件付きGO）

```
⚠️  p95 < 100ms ✅
⚠️  Error rate: 1.2% ❌
✅ Throughput > 500 req/s

原因分析:
  → エラーログ確認（500エラーの原因特定）
  → メトリクス詳細分析（どのエンドポイントで発生か）

アクション:
  → 軽微なエラー（auth timeout等）なら許容
  → データベースエラーなら追加チューニング必要
  → チーム判定後、Phase 1 進行 または 24時間追加テスト
```

### シナリオ C: p95 > 100ms（NO-GO判定）

```
❌ p95 > 120ms
⚠️  Error rate < 1%
✅ Throughput > 500 req/s

根本原因分析:
  → インデックス状態確認（PRAGMA ANALYZE 再実行）
  → ロック状況確認（SQLite WAL デバッグ）
  → メモリ不足確認（キャッシュサイズ調整）

アクション:
  → NO-GO 判定：本番デプロイ延期
  → チューニング実施（最大24時間）
  → 再テスト（翌日予定）
```

### シナリオ D: エラー率 > 2% または メモリリーク（NO-GO判定）

```
❌ Error rate > 2%
❌ Memory > 200MB で増加継続

根本原因分析:
  → データベース接続リーク（接続数監視）
  → メモリリーク検出（profiler 実行）
  → 認証エラー増加の原因

アクション:
  → NO-GO 判定：本番デプロイ中止
  → 緊急チューニング または バージョン巻き戻し
  → インシデント報告
```

---

## アクション判定チャート

```
                p95 < 100ms?
                     |
         ┌───────────┼───────────┐
         YES         NO
         |           |
    Error < 1%?   → NO-GO
         |           (チューニング)
    ┌────┼────┐
    YES   NO
    |     |
Throughput > 500?  Analyze
    |     |        Errors
┌───┼─────┤
YES NO    |
|   |  ┌──┴──┐
|   |  YES   NO
|   |  |     |
|   | Maybe  NO-GO
|  NO-GO     (Retune)
|
GO ✅
(Deploy)
```

---

## 記録・報告

### テスト開始時の記録

```
テスト開始日時: 2026-05-30 09:00 JST
テスト実行者: [名前]
サーバーバージョン: v3.0.0
テストスクリプト: tests/load/wave2_day5_load.js
環境: Docker Compose (Production-like)
```

### テスト完了後の報告

```
【ロードテスト結果】
実行日: 2026-05-30
p95: XXms
エラー率: X.X%
ピークスループット: XXX req/s

【判定】
✅ GO / ❌ NO-GO

【次アクション】
→ Phase 1 カナリアリリース開始
   OR
→ チューニング実施（期限: 2026-05-31 09:00）
```

---

## 参考

- [PERFORMANCE_BENCHMARKS.md](./PERFORMANCE_BENCHMARKS.md) — パフォーマンスベンチマーク
- [k6 Documentation](https://k6.io/docs/)
- [カナリアリリース計画](../Operations/CANARY_RELEASE_PLAN.md)
- [デプロイ手順](../Operations/DEPLOYMENT.md)

---

**ロードテスト計画完成！** 🚀

実行日: 2026-05-30 09:00-09:40 JST  
Go/No-Go判定時刻: 2026-05-30 09:45 JST  
本番デプロイ開始: 2026-05-30 10:00 JST（GO時）

**Location**: docs/Performance/LOAD_TEST_PLAN.md  
**Last Updated**: 2026-05-30
