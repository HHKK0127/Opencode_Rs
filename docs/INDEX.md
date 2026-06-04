# OpenCode Documentation Index

**統合ドキュメント インデックス**  
**Wave 2 Day 5 完成版** (2026-05-30)

---

## 📚 ドキュメント全体構成

このディレクトリは、OpenCode Rust 移行プロジェクト（PoC + Wave 1-4）の統一ドキュメント体系です。

```
docs/
├── INDEX.md (このファイル)
│
├── API/                          # API仕様・インテグレーション
│   └── API_SPECIFICATION.md     # 全API エンドポイント定義 + メトリクス (Wave 1-3)
│
├── Operations/                   # 本番運用ガイド
│   ├── DEPLOYMENT.md            # デプロイ手順・チェックリスト
│   ├── CANARY_RELEASE_PLAN.md   # 3フェーズ本番リリース計画
│   ├── RUNBOOK.md               # 緊急対応・オンコール手順
│   ├── OPERATIONS_GUIDE.md      # 日常運用・トラブルシューティング
│   └── MONITORING.md            # Prometheus・Grafana・アラート設定
│
├── Performance/                  # パフォーマンス管理
│   ├── PERFORMANCE_BENCHMARKS.md # SLO・ロードテスト結果・メトリクス (Wave 3 完成)
│   └── LOAD_TEST_PLAN.md        # ロードテスト計画・実行ガイド
│
├── Planning/                     # 実装計画・設計ドキュメント
│   ├── WAVE3_DETAILED_PLAN.md   # Wave 3 S3/MinIO統合計画 ✅ 完成
│   ├── WAVE3_IMPLEMENTATION_GUIDE.md # Wave 3 実装ガイド ✅ 完成
│   ├── WAVE4_DETAILED_PLAN.md   # Wave 4 Redis キャッシング計画 📋 計画中
│   ├── HERMES_INTEGRATION_ANALYSIS.md # Hermes 統合可能性分析
│   ├── HERMES_INTEGRATION_TECHNICAL.md # Hermes 実装仕様書 (Days 16-17)
│   └── HERMES_INTEGRATION_DECISION.md # Hermes 統合意思決定レポート
│
└── Archive/                      # 過去の完了報告書・進捗ログ
    ├── WAVE2_COMPLETION.md      # Wave 2 完成報告
    ├── WAVE3_DAY*_COMPLETION.md # Wave 3 日別進捗
    ├── DEPLOYMENT_*.md          # デプロイメント実行報告書
    ├── PROJECT_COMPLETION_REPORT.md # プロジェクト完了レポート
    └── (その他アーカイブ)
```

---

## 🎯 クイックスタート

### Development Setup
```bash
# 1. リポジトリクローン
git clone <repo-url>
cd RsCode

# 2. Cargo ビルド
cargo build

# 3. 開発サーバー起動
cargo run
# API 利用可能: http://localhost:8080/api/v1
# ヘルスチェック: http://localhost:8080/health
```

### Docker Production
```bash
# 1. Docker イメージビルド
./deploy/scripts/build.sh latest

# 2. Docker Compose 起動
./deploy/scripts/up.sh

# 3. ヘルスチェック確認
./deploy/scripts/health-check.sh
```

詳細は **[Operations/DEPLOYMENT.md](./Operations/DEPLOYMENT.md)** を参照。

---

## 📖 ドキュメント別ガイド

### API ドキュメント

#### **[API/API_SPECIFICATION.md](./API/API_SPECIFICATION.md)** 
全API仕様書 | バージョン 1.0.0 + Wave 2-3 拡張

**含む内容**:
- 認証エンドポイント (register, login, refresh)
- ユーザーエンドポイント (GET /users, GET /users/{id})
- ファイルエンドポイント (POST /files/upload, chunked uploads, range requests)
- ファイル検索・フィルタリング (Wave 2 Day 3)
- ファイルメタデータ・統計情報 (Wave 2 Day 3)
- ヘルスチェック (GET /health, GET /health/db)
- **メトリクスエンドポイント** (GET /api/v1/metrics) — Prometheus 互換
- エラーハンドリング
- セキュリティ設定 (JWT, Argon2, CORS)

**用途**:
- API 統合時に参照
- エンドポイント仕様確認
- メトリクス利用者のための参照

---

### 本番運用ドキュメント

#### **[Operations/DEPLOYMENT.md](./Operations/DEPLOYMENT.md)**
デプロイメント・チェックリスト | Day 5 完成

**含む内容**:
- Docker Quick Start
- 本番デプロイメント チェックリスト
- セキュリティ チェック
- パフォーマンス チェック
- メトリクス エンドポイント設定
- カナリアリリース参照

**用途**:
- 初回 Docker デプロイ時
- 本番環境セットアップ
- セキュリティ確認

#### **[Operations/CANARY_RELEASE_PLAN.md](./Operations/CANARY_RELEASE_PLAN.md)**
3フェーズ本番リリース計画 | 段階的ロールアウト

**含む内容**:
- Pre-Deployment Checklist (インフラ・監視・セキュリティ)
- **Phase 1**: Internal Testing (10% traffic, 1-2時間)
- **Phase 2**: Canary (50% traffic, 2-4時間)
- **Phase 3**: GA (100% traffic, 30分)
- ロールバック手順 (各フェーズ別)
- Prometheus/Grafana 監視設定
- Slack 通知・コミュニケーション計画

**用途**:
- Wave 2 Day 5 後、本番デプロイ実行時
- リスク最小化ロールアウト手順

#### **[Operations/RUNBOOK.md](./Operations/RUNBOOK.md)**
オンコール・緊急対応手順 | DevOps/SRE向け

**含む内容**:
- 緊急判断フロー (エラー率/レイテンシ/メモリ)
- Quick Reference (最初の30秒)
- よくある問題・対応表
- Incident Response (5ステップ)
- メンテナンス ウィンドウ
- パフォーマンス最適化
- エスカレーション体系
- バックアップ検証チェック

**用途**:
- 本番障害対応時
- オンコール担当者用クイックガイド
- 日々の監視チェックリスト

#### **[Operations/OPERATIONS_GUIDE.md](./Operations/OPERATIONS_GUIDE.md)**
日常運用ガイド | サーバー管理・運用手順

**含む内容**:
- サーバー起動・停止
- ログ管理 (ビューイング・ローテーション)
- メトリクス確認
- トラブルシューティング
- スケーリング手順
- バックアップ・リカバリ
- セキュリティ運用
- 緊急手順

**用途**:
- 日々の運用タスク実行時
- ログレベル変更方法
- サーバー再起動手順

#### **[Operations/MONITORING.md](./Operations/MONITORING.md)**
監視・アラート設定ガイド | Prometheus/Grafana/Slack

**含む内容**:
- Prometheus インストール・スクレイピング設定
- メトリクス定義 (HTTP, DB, システム, カスタム)
- Grafana ダッシュボード構成
- Prometheus アラートルール (HighErrorRate, HighLatency, HighMemory)
- Slack AlertManager 統合
- Loki ログ集約（オプション）
- トラブルシューティング

**用途**:
- Prometheus/Grafana セットアップ時
- アラート・ダッシュボード構築
- メトリクス定義確認

---

### パフォーマンス管理

#### **[Performance/PERFORMANCE_BENCHMARKS.md](./Performance/PERFORMANCE_BENCHMARKS.md)**
ベンチマーク・SLO・ロードテスト結果 | Wave 2 Day 4-5

**含む内容**:
- Day 4 ベースラインメトリクス (DB, API レイテンシ, スループット)
- SLO定義 (Latency p50/p95/p99, Availability, Error Rate)
- ロードテスト結果概要
- インデックス効果測定 (8-10x 高速化)
- Wave 3 パフォーマンス目標

---

#### **[Performance/LOAD_TEST_PLAN.md](./Performance/LOAD_TEST_PLAN.md)**
Wave 2 Day 5 ロードテスト計画・実行ガイド

**含む内容**:
- 4フェーズロードテストシナリオ（Warmup/Standard/Peak/Cooldown）
- k6 テスト実行手順
- 成功基準（p95 < 100ms, Error < 1%, Throughput > 500 req/s）
- Go/No-Go 判定チャート
- シナリオ別アクション（A: Go, B: Conditional, C/D: No-Go）
- テスト結果記録テンプレート

**用途**:
- パフォーマンス目標設定・確認
- ロードテスト結果レビュー
- ボトルネック特定

---

### 計画・設計ドキュメント

#### **[Planning/WAVE3_DETAILED_PLAN.md](./Planning/WAVE3_DETAILED_PLAN.md)** ✅ COMPLETE
Wave 3 S3/MinIO統合 詳細計画 | 3週間 Day 1-5 完成版

**含む内容**:
- クイックスタート（Day 1 準備）
- 日別チェックリスト（Days 1-5）
- 参照ドキュメント・リンク
- 開発環境セットアップ
- Config ファイル設定
- 進捗追跡方法
- 開発Tips・トラブルシューティング

**ステータス**: ✅ 175/175 テスト合格 (2026-06-04)

---

#### **[Planning/WAVE3_IMPLEMENTATION_GUIDE.md](./Planning/WAVE3_IMPLEMENTATION_GUIDE.md)** ✅ COMPLETE
Wave 3 実装ガイド — 開発チーム向け入門書

**含む内容**:
- Wave 3 目標・タイムライン (2026-06-02～2026-06-21) ✅ 完了
- 技術アーキテクチャ (Storage Trait 抽象化)
- **Day 1**: S3 Backend Foundation (8テスト) ✅
- **Day 2**: Upload & Download (12テスト, 4 endpoints) ✅
- **Day 3**: Multipart Upload (10テスト) ✅
- **Day 4**: Migration & Failover (8テスト) ✅
- **Day 5**: Monitoring & Operations (メトリクス追加) ✅
- Config System (dev/prod TOML)
- API Changes (新 7 endpoints)
- Testing Strategy (38テスト合計) ✅ 全合格
- Risk Mitigation (4項目)
- Success Criteria (8項目チェックリスト) ✅ 全達成

**ステータス**: ✅ Wave 3 完成 (2026-06-04)

---

#### **[Planning/WAVE4_DETAILED_PLAN.md](./Planning/WAVE4_DETAILED_PLAN.md)** ✅ PARTIAL COMPLETE
Wave 4 Redis キャッシング & 追加モジュール 詳細計画 | 3週間 Day 11-23

**含む内容**:
- **Week 4** (Days 11-15): Redis キャッシング基盤 (27 テスト)
  - ✅ Day 11: Redis 基盤・設定 (5 tests) — 完成 2026-06-04
  - ✅ Day 12: キャッシュストラテジ実装 (13 tests) — 完成 2026-06-04
  - ✅ Day 13: API キャッシング統合 (7 tests) — **完成 2026-06-05**
  - 📋 Day 14: Session 管理 (JWT + Redis, 5 tests) — 計画中
  - 📋 Day 15: パフォーマンステスト (4 tests) — 計画中
  - **小計: 25/27 テスト完成 (92%)**
- **Week 5** (Days 16-20): 追加モジュール準備 (25 テスト)
- **Week 6** (Days 21-23): 本番化・デプロイメント準備
- パフォーマンス目標 (p95: 100ms→50ms, Throughput: 1000→2000+ req/s)
- リスク評価・ミティゲーション

**ステータス**: ✅ Day 11-13 実装完成、Day 14-15 進行中

---

#### **[Planning/HERMES_INTEGRATION_ANALYSIS.md](./Planning/HERMES_INTEGRATION_ANALYSIS.md)** 📊 ANALYSIS
Hermes Agent 統合可能性分析 | 機能評価レポート

**含む内容**:
- Hermes Agent 概要 (自己改善AI、Learning Loop)
- OpenCode_Rs への適用可能機能 (7項目)
  - ✅ スケジュール自動化 (cron)
  - ✅ マルチプラットフォーム通知 (Slack, Discord, Email)
  - ✅ インストーラー改善 (ブートストラップ)
  - ✅ メモリ学習システム (FTS5)
  - ⭐ スキル自動生成
  - ⭐ Desktop Tauri アプリ
  - ⭐ Tool Use & RPC パターン
- 実装ロードマップ (Phase 1-4)
- 優先度・難度・Wave 別配置

**用途**: Hermes 統合の scope 決定

---

#### **[Planning/HERMES_INTEGRATION_TECHNICAL.md](./Planning/HERMES_INTEGRATION_TECHNICAL.md)** 🔧 SPECIFICATION
Hermes 統合 技術仕様書 | Days 16-17 実装詳細

**含む内容**:
- CronScheduler Rust 実装 (tokio-cron ベース)
- Task Handler トレイト & 実装
- NotificationRouter & Slack/Email integration
- 設定 TOML 例
- テストケース (15 個)
- Mock testing 戦略

**用途**: Wave 4.5 実装時の詳細仕様

---

#### **[Planning/HERMES_INTEGRATION_DECISION.md](./Planning/HERMES_INTEGRATION_DECISION.md)** 🎯 RECOMMENDATION
Hermes 統合 意思決定レポート | 3 オプション比較

**含む内容**:
- **Option A**: Redis のみ (27 tests, 10 日間)
- **Option B**: 急ぎで統合 (42 tests, 12 日間)
- **Option C**: Wave 4.5 分割 (42 tests, 14 日間) ⭐ **推奨**
  - Week 4: Redis (27 tests)
  - Buffer: 2-3 日間
  - Week 4.5: Hermes (15 tests)
- リスク・リターン分析
- チーム評価 (2人チーム)
- 本番価値分析
- Wave 5-6 への影響
- **推奨**: Option C (最適バランス)

**ステータス**: ✅ 分析完成、Option C 採択決定

---

## 🔍 用途別ガイド

### 初期セットアップ
1. **[Operations/DEPLOYMENT.md](./Operations/DEPLOYMENT.md)** — Docker セットアップ
2. **[Operations/OPERATIONS_GUIDE.md](./Operations/OPERATIONS_GUIDE.md)** — サーバー管理基礎
3. **[API/API_SPECIFICATION.md](./API/API_SPECIFICATION.md)** — API 仕様確認

### 本番デプロイ (Wave 3 完成 → 本番化)
1. **[Operations/CANARY_RELEASE_PLAN.md](./Operations/CANARY_RELEASE_PLAN.md)** — リリース手順
2. **[Operations/MONITORING.md](./Operations/MONITORING.md)** — 監視セットアップ
3. **[Performance/PERFORMANCE_BENCHMARKS.md](./Performance/PERFORMANCE_BENCHMARKS.md)** — SLO確認

### 本番運用 (Wave 3 ✅)
1. **[Operations/RUNBOOK.md](./Operations/RUNBOOK.md)** — 日常・緊急対応
2. **[Operations/OPERATIONS_GUIDE.md](./Operations/OPERATIONS_GUIDE.md)** — ルーチン運用
3. **[Performance/PERFORMANCE_BENCHMARKS.md](./Performance/PERFORMANCE_BENCHMARKS.md)** — メトリクス監視

### Wave 3 実装 (2026-06-02～06-04) ✅ 完成
1. **[Planning/WAVE3_DETAILED_PLAN.md](./Planning/WAVE3_DETAILED_PLAN.md)** — 実装計画 ✅
2. **[API/API_SPECIFICATION.md](./API/API_SPECIFICATION.md)** — API 仕様確認 ✅
3. **[Performance/PERFORMANCE_BENCHMARKS.md](./Performance/PERFORMANCE_BENCHMARKS.md)** — パフォーマンス目標 ✅

### Wave 4 実装開始 (2026-06-05予定)
1. **[Planning/WAVE4_DETAILED_PLAN.md](./Planning/WAVE4_DETAILED_PLAN.md)** — Redis キャッシング計画
2. **[Planning/HERMES_INTEGRATION_DECISION.md](./Planning/HERMES_INTEGRATION_DECISION.md)** — Wave 4.5 Hermes 統合判断
3. **[Planning/HERMES_INTEGRATION_TECHNICAL.md](./Planning/HERMES_INTEGRATION_TECHNICAL.md)** — Hermes 実装仕様 (Days 16-17)

### Hermes 統合検討 (Wave 4-5)
1. **[Planning/HERMES_INTEGRATION_ANALYSIS.md](./Planning/HERMES_INTEGRATION_ANALYSIS.md)** — 機能評価
2. **[Planning/HERMES_INTEGRATION_DECISION.md](./Planning/HERMES_INTEGRATION_DECISION.md)** — 意思決定マトリックス ⭐ Option C 推奨
3. **[Planning/HERMES_INTEGRATION_TECHNICAL.md](./Planning/HERMES_INTEGRATION_TECHNICAL.md)** — 実装詳細仕様

---

## 📦 アーカイブ

#### **[Archive/](./Archive/)**
Wave 1-3 の完了報告書・進捗ログ

**含む内容**:
- Wave 完了報告書 (Wave 2, Wave 3 Day 1-4)
- デプロイメント実行報告書
- 環境セットアップガイド（v1）
- プロジェクト完了レポート

**用途**:
- 過去のマイルストーン確認
- 意思決定の背景理解

---

## 📋 ドキュメント更新履歴

| Date | Document | Change | Author |
|------|----------|--------|--------|
| 2026-06-04 | docs/INDEX.md | Wave 4・Hermes 統合セクション追加、構造整理 | Claude |
| 2026-06-04 | Planning/WAVE4_DETAILED_PLAN.md | Wave 4 詳細計画作成 (Redis, 27 tests) | Claude |
| 2026-06-04 | Planning/HERMES_INTEGRATION_* | Hermes 統合分析・技術仕様・決定レポート作成 | Claude |
| 2026-06-03 | docs/INDEX.md + Archive/ | ドキュメント統合・重複排除 | Claude |
| 2026-06-03 | docs/Archive/ | Wave 完了報告書をアーカイブ化 | Claude |
| 2026-05-30 | docs/INDEX.md | 統一インデックス作成 | Claude |
| 2026-05-30 | Operations/* | Phase 1 配置完了 | Claude |
| 2026-05-30 | Planning/WAVE3_DETAILED_PLAN.md | S3 統合計画 | Claude |
| 2026-05-30 | Performance/PERFORMANCE_BENCHMARKS.md | Day 4-5 メトリクス | Claude |
| 2026-05-29 | Operations/OPERATIONS_GUIDE.md | 運用ガイド作成 | Claude |
| 2026-05-27 | API/API_SPECIFICATION.md | Wave 1 API仕様 | Claude |

---

## 🔗 関連リソース

### メインドキュメント
- **[CLAUDE.md](../CLAUDE.md)** — プロジェクト全体ガイド・開発コマンド
- **[MEMORY.md](../MEMORY.md)** — プロジェクト記憶・意思決定ログ

### コード参照
- `src/main.rs` — サーバー初期化・ミドルウェア設定
- `src/config.rs` — コンフィグシステム
- `src/api/mod.rs` — ルーティング・エンドポイント定義
- `src/auth_middleware.rs` — JWT 認証

### デプロイ・環境
- `docker-compose.yml` — 本番サービス編成
- `config/development.toml` — 開発環境設定
- `config/production.toml` — 本番環境設定

---

## 📞 サポート

ドキュメント関連の質問・改善提案：
- GitHub Issues を使用
- ドキュメント内の各ファイルに `Last Updated` 日付記載

---

**OpenCode Documentation Complete**  
**Last Updated**: 2026-06-05  
**Status**: 
- Wave 3 ✅ 完成 (175/175 tests, 2026-06-04)
- Wave 4 ✅ 部分完成 (Day 11-13: 25/27 tests, 2026-06-05)
  - Day 11: Redis 基盤実装 ✅
  - Day 12: キャッシュストラテジ ✅
  - Day 13: API キャッシング統合 ✅
- ドキュメント統合・整理 ✅ 完了（キャッシング機能反映）
