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
| **opencode-electron** (Desktop App) | SolidJS + Electron | ✅ Phase 2 Complete — Phase 3 進行中 |
| **opencode_tui** (TUI) | Rust (Ratatui) | ✅ 121/121 タスク完了 |

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
├── opencode-flutter/       # デスクトップアプリ (Flutter GUI) 🆕
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
| Phase 2 | ✅ Done | 認証画面 UI + ログイン/ログアウト + JWT 永続化 |
| Phase 3 | 🔄 In Progress | ダッシュボード + ファイルブラウザ |
| Phase 4-7 | ⏳ Pending | エディタ、テスト、リリース |

### 起動方法 / How to Run

```bash
cd opencode-electron
npm install
npm run dev
# → Electron ウィンドウ起動 (http://localhost:5173)
```

> **Note**: Repository is now located at `C:\Drive\Cargo\OpenCode_Rs`. Git operations should be done from this path.

---

## 🖥️ Ratatui TUI (Rust) / Terminal UI

`opencode_tui` は Ratatui ベースのモダンなターミナル UI です。

### 機能一覧 / Features

| 機能 | 説明 |
| --- | --- |
| **チャット画面** | メッセージ一覧 + 入力エリア |
| **ストリーミング応答** | 文字単位のインクリメンタル表示 |
| **Markdown レンダリング** | コードハイライト、テーブル、リスト対応 |
| **Diff 色付け** | コードブロック内の +/- 行を色分け |
| **テーマシステム** | 10テーマ（dark, light, gruvbox, solarized, nord, catppuccin, tokyo-night） |
| **モデルピッカー** | Ctrl+P で LLM モデル切替 |
| **サイドバー** | Ctrl+S でチャット履歴表示 |
| **ツールパネル** | Ctrl+T でツールコール表示 |
| **エディタ** | Ctrl+E でインラインエディタ |
| **@mention 補完** | @file, @code, @history, @model |
| **スラッシュコマンド** | /help, /clear, /model, /files, /compact |
| **Vim キーバインド** | hjkl, オペレータ+モーション対応 |
| **ファジー検索** | サブシーケンススコアリング |
| **シマーアニメーション** | 読み込み中のエフェクト |
| **トークン使用量表示** | 応答速度 (tok/s) + 使用量 |

### 起動方法 / How to Run

```powershell
cd C:\Drive\Cargo\OpenCode_Rs

# API キー設定
$env:ANTHROPIC_API_KEY = "sk-ant-..."

# ビルド & 実行
cargo run --bin opencode_tui

# または直接実行
.\target\debug\opencode_tui.exe
```

### キーバインド / Key Bindings

| キー | 操作 |
| --- | --- |
| `Enter` | プロンプト送信 |
| `Ctrl+K` | 設定画面 |
| `Ctrl+Q` | 終了 |
| `Ctrl+S` | サイドバー表示/非表示 |
| `Ctrl+T` | ツールパネル表示/非表示 |
| `Ctrl+L` | ツールコール展開/折りたたみ |
| `Ctrl+E` | エディタ表示/非表示 |
| `Ctrl+M` | マルチライン入力トグル |
| `Ctrl+P` | モデルピッカー |
| `Tab` | フォーカス切替 |
| `Up/Down` | スクロール |
| `F1` | ヘルプ表示 |

### Vim モード / Vim Mode

エディタ表示時に Vim モードが有効になります:

| モード | キー | 操作 |
| --- | --- | --- |
| Normal | `h/j/k/l` | カーソル移動 |
| Normal | `0` / `$` | 行頭/行末 |
| Normal | `i` | Insert モード |
| Normal | `A` | 行末で Insert |
| Normal | `d` + `motion` | 削除 |
| Normal | `y` + `motion` | ヤンク |
| Insert | `Esc` | Normal モード |

### スラッシュコマンド / Slash Commands

| コマンド | 説明 |
| --- | --- |
| `/help` | ヘルプ表示 |
| `/clear` | チャット履歴クリア |
| `/model` | モデル切替 |
| `/files` | ファイル一覧表示 |
| `/compact` | メッセージ圧縮 |

### Warp-inspired アーキテクチャ / Architecture

TUI は以下のパターンで構築されています:

| パターン | 説明 |
| --- | --- |
| **Keymap matcher** | Trie ベースのマルチキーシーケンスマッチング |
| **InputClassifier** | キーイベントを7カテゴリに分類 |
| **PositionCache** | レイアウト位置キャッシュ |
| **CompletionEngine** | マルチタイプ補完管理 |
| **TaskQueue** | 前景/背景タスク分離 |
| **View+Element** | UI コンポジション |
| **VimFSA** | Vim 有限状態機械 |
| **Plugin System** | プラグインライフサイクル管理 |
| **Toast Notification** | 通知システム |
| **Event Bubbling** | イベント伝播 |

---

## 📱 Flutter UI (Desktop GUI)

Flutter 版は Ratatui TUI と分離して運用します。

```bash
cd opencode-flutter
flutter pub get
flutter run -d windows
```

実装済み:
- Login (`POST /api/v1/auth/login`)
- Test mode bypass
- Files list (`GET /api/v1/files`)

詳細: `opencode-flutter/README.md`

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

## 🗺️ ロードマップ / Roadmap

OpenCode_Rs は現在 **3つのプロジェクト** を並行開発しています。
主軸は **opencode-electron (デスクトップアプリ)** の Phase 2-7 完了です。

Three projects running in parallel. The main focus is completing **opencode-electron Phases 2-7**.

### 全体マイルストーン / Global Milestones

```mermaid
gantt
    title OpenCode_Rs 全体ロードマップ
    dateFormat  YYYY-MM-DD
    axisFormat  %m/%d

    section 🖥️ Electron Desktop (Main)
    Phase 2: 認証画面                           :done, 2026-07-08, 5d
    Phase 3: ダッシュボード                     :active, 2026-07-14, 7d
    Phase 4: エディタ (Monaco)                  :2026-07-22, 8d
    Phase 5: Electron 機能                      :2026-08-01, 7d
    Phase 6: テスト・品質                        :2026-08-09, 7d
    Phase 7: リリース                            :2026-08-17, 5d

    section 🦀 Backend (opencode_poc)
    メンテナンス・安定運用                      :done, 2026-07-01, 60d

    section 📡 Desktop Server (opencode-core)
    Phase 2: 単体テスト                         :2026-08-01, 7d
    Phase 3: Provider 管理                      :2026-08-09, 7d
    Phase 4: ツール実行エンジン                  :2026-08-17, 7d
```

### プロジェクト別ロードマップ / Per-Project Roadmap

#### 🖥️ opencode-electron (Desktop App) — メイン開発中

Electron + SolidJS で OpenCode デスクトップクライアントを構築。

| Phase | マイルストーン / Milestone | 日付 / Date | 状態 |
|:-----:|---------------------------|:----------:|:----:|
| 0 | セキュリティ修正済み基盤 / Scaffold | ~Jul 03 | ✅ Done |
| 1 | Electron 起動確認 / Boot & Verify | ~Jul 08 | ✅ Done |
| **2** | **認証画面 / Login + JWT** | **Jul 08-13** | **✅ Done** |
| **3** | **ダッシュボード / Dashboard + File Browser** | **Jul 14-21** | **🔄 Active** |
| 4 | コードエディタ / Monaco Editor + Tabs | Jul 22-31 | ⏳ |
| 5 | Electron機能 / Menu + Tray + Auto-update | Aug 01-07 | ⏳ |
| 6 | テスト・品質 / Vitest + Playwright | Aug 08-15 | ⏳ |
| 7 | リリース / electron-builder + GitHub Releases | Aug 16-22 | ⏳ |

**🔗 詳細**: `opencode-electron/README.md`

#### 🦀 opencode_poc (API Server) — 完了・安定運用中

Wave 1-5 全完了 (229/229 tests ✅)。**本番移行 GO** — 新規機能追加なし、メンテナンスのみ。

| Task | Status |
|------|:------:|
| 既存機能の安定運用 / Stable operations | ✅ Active |
| 必要に応じたバグ修正 / Bug fixes as needed | ⏳ |

#### 📡 opencode-core (Desktop Server) — V2 API Phase 1 完了

Rust で OpenCode Desktop サーバープロトコルを再実装。V2 API の基本エンド＋イベント配信まで完了。

| Phase | マイルストーン / Milestone | 状態 |
|:-----:|---------------------------|:----:|
| 1 | V2 API 全エンドポイント + SSE + モックLLM | ✅ Done |
| 2 | 単体テスト追加 / Unit tests | ⏳ |
| 3 | Provider API キー管理 / Provider key management | ⏳ |
| 4 | ツール実行エンジン / Tool execution engine | ⏳ |

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

## 🗂️ リポジトリ場所 / Repository Location

**C: ドライブ** (`C:\Drive\Cargo\OpenCode_Rs`) が現在のメイン作業ディレクトリです。

The repository is now located at `C:\Drive\Cargo\OpenCode_Rs`.

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
