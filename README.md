# 🚀 OpenCode_Rs

**Rust バックエンド + マルチフロントエンド構成の AI 開発ツールプラットフォーム**

> 日本語 / English

---

## 📋 プロジェクト概要 / Project Overview

OpenCode_Rs は、大規模 TypeScript アプリケーション「OpenCode」(43K行) を Strangler Fig パターンで段階的に Rust へ移行するプロジェクトです。

OpenCode_Rs is a project to incrementally migrate the large-scale TypeScript application "OpenCode" (43K lines) to Rust using the Strangler Fig pattern.

| Component | Language | Status |
|-----------|----------|--------|
| **opencode_poc** (API Server) | Rust (Actix-web) | ✅ Wave 5 Complete — Production Ready |
| **opencode-core** (Desktop Server) | Rust (Actix-web) | ✅ V2 API Phase 1 Complete |
| **opencode-desktop** (Web Frontend) | React 19 + TypeScript | ⏳ Legacy (統合予定) |
| **opencode-electron** (Desktop App) | SolidJS + Electron | ✅ Phase 1 Complete — Phase 2 進行中 |

---

## 📦 プロジェクト構成 / Project Structure

```
OpenCode_Rs/
├── src/                    # opencode_poc: メイン API サーバー
│   ├── api/                #   RESTful エンドポイント
│   ├── cache/              #   Redis キャッシュ層
│   ├── storage/            #   S3/MinIO ストレージ
│   ├── auth_middleware.rs  #   JWT 認証
│   └── main.rs             #   サーバーエントリポイント
│
├── opencode-core/          # OpenCode Desktop サーバー (Rust)
│   └── src/
│       ├── api/            # V1+V2 API エンドポイント
│       ├── server.rs       # OpenCodeServer 構造体
│       └── bin/server.rs   # バイナリエントリポイント
│
├── opencode-desktop/       # Web フロントエンド (React, legacy)
├── opencode-electron/      # デスクトップアプリ (SolidJS + Electron) 🆕
│
├── config/                 # TOML 設定ファイル
├── deploy/                 # デプロイスクリプト
├── docs/                   # ドキュメント
├── k8s/                    # Kubernetes マニフェスト
├── tests/                  # 統合テスト
│
├── AGENTS.md               # AI エージェント向け設定ファイル
├── Dockerfile              # マルチステージビルド
└── docker-compose.yml      # サービスオーケストレーション
```

---

## 🖥️ デスクトップアプリ (opencode-electron) / Desktop App

### 現状 / Current Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 | ✅ Done | セキュリティ修正・技術スタック確定 |
| Phase 1 | ✅ Done | Electron 起動確認 (Vite 8.1 + SolidJS) |
| Phase 2 | 🔄 In Progress | 認証画面 UI + バックエンド接続 |
| Phase 3 | ⏳ Pending | ファイルエクスプローラー + コードエディタ |
| Phase 4-7 | ⏳ Pending | メニュー、テスト、リリース |

### 起動方法 / How to Run

```bash
cd opencode-electron
npm install
npm run dev
# → Electron ウィンドウ起動 (http://localhost:5173)
```

> **Note**: Development is centered on the `C:` drive (`C:\Drive\Cargo\OpenCode_Rs`). The `G:` drive is a Google Drive sync backup.

---

## 🦀 バックエンド / Backend (opencode_poc)

### 起動 / Run

```bash
# PostgreSQL が必要です (Docker Compose)
docker-compose up -d postgres redis

# 環境変数設定 / Set environment variables
$env:DATABASE_URL="postgresql://opencode:opencode_password@localhost:5432/opencode_dev"
$env:JWT_SECRET="your-secret-key-here"

# 起動 / Start server
cargo run --release
# → http://127.0.0.1:8080
```

### 主要エンドポイント / Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/auth/login` | JWT ログイン / JWT login |
| `POST` | `/api/v1/auth/register` | ユーザー登録 / User registration |
| `GET` | `/api/v1/files` | ファイル一覧 / List files (pagination) |
| `POST` | `/api/v1/files/upload` | ファイルアップロード / Upload file |
| `GET` | `/api/v1/files/{id}/download` | ダウンロード / Download (Range support) |
| `GET` | `/health` | ヘルスチェック / Health check |

### テスト / Tests

```bash
# 全テスト実行 / Run all tests
cargo test

# 特定クレート / Specific crate
cargo test -p opencode-core

# バックトレース付き / With backtrace
RUST_BACKTRACE=1 cargo test
```

---

## 📊 完了 Wave / Completed Waves

| Wave | Content | Tests |
|------|---------|-------|
| Wave 1 | JWT Auth + Middleware + DB | 30 ✅ |
| Wave 2 | File API + Chunked Upload + Search | 47 ✅ |
| Wave 3 | S3/MinIO Cloud Storage | 45 ✅ |
| Wave 4 | Redis Cache + Session Management | 107 ✅ |
| Wave 5 | Production + K8s + CI/CD + Canary | 18 ✅ |
| **Total** | | **229/229 ✅** |

### 本番対応 / Production Features
- ✅ PostgreSQL 16 + SQLx 0.7
- ✅ JWT HS256 + Argon2id パスワードハッシング
- ✅ S3/MinIO 互換ストレージ
- ✅ Redis キャッシュ (グレースフルフォールバック)
- ✅ Kubernetes デプロイメント + Canary Release
- ✅ Docker マルチステージビルド (~150MB)
- ✅ CI/CD (GitHub Actions)
- ✅ 構造化ロギング (tracing)

---

## 🔧 開発環境 / Development Setup

### 必要条件 / Prerequisites
- Rust 1.85+ (stable)
- Node.js 20+
- Docker Desktop (PostgreSQL + Redis)
- (Optional) k6 for load testing

### 設定 / Configuration

環境変数 / Environment variables (`.env`):

```bash
JWT_SECRET=your-secret-key-here
DATABASE_URL=postgresql://opencode:opencode_password@localhost:5432/opencode_dev
REDIS_URL=redis://:test_password@localhost:6379
RUST_LOG=info
ENVIRONMENT=development
```

または config TOML / Or config TOML (default: `config/development.toml`):

```bash
ENVIRONMENT=production  # → loads config/production.toml
```

---

## 🌐 本番デプロイ / Production Deployment

```bash
# Docker ビルド / Docker build
./deploy/scripts/build.sh latest

# サービス起動 / Start services
./deploy/scripts/up.sh

# ヘルスチェック / Health check
./deploy/scripts/health-check.sh

# 停止 / Stop
./deploy/scripts/down.sh
```

Kubernetes (Docker Desktop 組み込み / built-in):
```bash
kubectl apply -k k8s/
kubectl port-forward -n opencode service/opencode-api-lb 8090:80
```

---

## 📁 関連ドキュメント / Related Docs

| Document | Description |
|----------|-------------|
| `opencode-electron/README.md` | Electron アプリ詳細 + ロードマップ / Electron app details |
| `AGENTS.md` | AI エージェント設定 (Rust アーキテクチャ詳細) / AI agent config |
| `docs/INDEX.md` | ドキュメントナビゲーションハブ / Doc navigation hub |
| `docs/MEMORY.md` | プロジェクト意思決定ログ / Decision log |
| `docs/API/API_SPECIFICATION.md` | API 仕様書 / API specification |

---

## 🗂️ データ同期 / Data Sync

**C: ドライブ (一次)** → **G: ドライブ (Google Drive バックアップ)**

```bash
# G: ドライブへ同期 (node_modules 除外)
robocopy C:\Drive\Cargo\OpenCode_Rs G:\マイドライブ\Cargo\OpenCode_Rs /MIR /XD node_modules .git vendor .tmp.driveupload target
```

---

## 🤝 コントリビューション / Contributing

1. Fork する / Fork the repo
2. フィーチャーブランチを作成 / Create feature branch (`git checkout -b feature/amazing-feature`)
3. 変更をコミット / Commit changes (`git commit -m 'feat: add amazing feature'`)
4. プッシュ / Push (`git push origin feature/amazing-feature`)
5. Pull Request を作成 / Open a Pull Request

---

## 📝 ライセンス / License

MIT

---

**Made with ❤️**
