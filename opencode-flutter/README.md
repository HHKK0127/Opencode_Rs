# OpenCode Flutter UI

Flutter ベースの GUI クライアントです。`opencode_tui` (Ratatui) とは別実装で並行運用します。

## アーキテクチャ

```
lib/
├── app.dart                    # BLoCプロバイダー設定 + AuthGate
├── main.dart                   # エントリポイント（Hive初期化）
├── blocs/
│   ├── auth/                   # AuthBloc（認証状態管理）
│   └── launcher/               # LauncherBloc（モード切替・Start/Stop/Reload）
├── models/                     # json_serializable対応モデル
├── repositories/               # API/ローカルストレージ抽象化層
├── services/                   # DioClient + ApiService
├── screens/                    # 画面群
└── utils/constants.dart        # 設定定数
```

### 技術スタック
- 状態管理: **flutter_bloc + equatable**
- HTTP: **dio**
- ローカル保存: **Hive / hive_flutter**
- UI: **Material 3**（ダークテーマ）
- コード生成: **json_serializable + build_runner**

## Prerequisites

- Flutter SDK 3.3+
- Windows Developer Mode 有効（プラグイン使用時）

## Setup

```bash
cd opencode-flutter
flutter pub get
dart run build_runner build --delete-conflicting-outputs
```

## Run

```bash
flutter run -d windows
```

## Build

```bash
flutter build windows --release
```

出力: `build/windows/x64/runner/Release/opencode_flutter.exe`

## Optional compile-time flags

```bash
flutter run -d windows --dart-define=API_BASE_URL=http://localhost:8080
flutter run -d windows --dart-define=CORE_API_BASE_URL=http://localhost:4096
flutter run -d windows --dart-define=ALLOW_TEST_LOGIN_BYPASS=true
```

## 機能一覧

### 認証
- ログイン画面（`/api/v1/auth/login`）
- テストモードログインバイパス
- Hiveによるトークン永続化

### ランチャー画面
- 実行モード切替: **OpenCode** / **AI Terminal**
- 共通操作: **Start** / **Stop** / **Reload** ボタン
- ステータス表示: `Running` / `Stopped` / `Error`
- バックエンドヘルスチェック（OpenCode API / Core API）

### AI Terminal
- セッション作成（`POST /v2/session`）
- プロンプト送信（`POST /v2/session/{id}/prompt`）
- メッセージ取得（`GET /v2/session/{id}/message`）

### ファイルブラウザ
- ファイル一覧表示（ページング対応）
- リフレッシュ機能

## バックエンド要件

| サービス | デフォルトURL | 用途 |
|---------|-------------|------|
| OpenCode API | `http://localhost:8080` | 認証・ファイル・ヘルス |
| OpenCode Core | `http://localhost:4096` | AI Terminal セッション |
