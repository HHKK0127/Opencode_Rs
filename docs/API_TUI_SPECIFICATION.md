# OpenCode TUI API 仕様書

---

**最終更新**: 2026-07-18  
**バージョン**: 1.0.0  
**対象**: opencode_tui (Ratatui TUI)

---

## 概要

OpenCode TUI は LLM プロバイダーと直接通信し、チャットセッションを管理します。

### アーキテクチャ

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Ratatui TUI   │────▶│  opencode-llm   │────▶│  LLM Provider   │
│  (opencode_tui) │     │    (crate)      │     │ (Anthropic/OpenAI)│
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

---

## LLM API

### プロバイダー切替

```rust
// Anthropic (デフォルト)
Provider::Anthropic { api_key: env::var("ANTHROPIC_API_KEY") }

// OpenAI 互換
Provider::OpenAICompatible { 
    api_key: env::var("OPENAI_API_KEY"),
    base_url: "https://api.openai.com/v1".to_string(),
}
```

### メッセージ送信

```rust
// チャットメッセージ
ChatMessage {
    role: Role,      // User / Assistant / System
    content: String, // テキスト内容
}

// LLM リクエスト
LlmRequest {
    model: String,              // "claude-sonnet-4-20250514"
    messages: Vec<ChatMessage>, // 会話履歴
    max_tokens: u32,            // 最大トークン数
    temperature: f32,           // 温度パラメータ
    stream: bool,               // ストリーミング有効
}
```

### ストリーミング応答

```rust
// ストリーミングチャンク
StreamChunk {
    content: String,     // 追加テキスト
    finish_reason: Option<FinishReason>,
}

// 完了理由
enum FinishReason {
    Stop,      // 正常終了
    Length,    // トークン制限
    Error,     // エラー
}
```

---

## セッション API

### セッション管理

```rust
// チャットセッション
ChatSession {
    id: String,                // UUID
    title: String,             // セッションタイトル
    messages: Vec<ChatMessage>, // メッセージ履歴
    model: String,             // 使用モデル
    created_at: DateTime,      // 作成日時
    updated_at: DateTime,      // 更新日時
}

// セッション操作
enum SessionAction {
    Create,                    // 新規作成
    Load(String),              // 読み込み
    Save,                      // 保存
    Delete(String),            // 削除
    Rename(String, String),    // リネーム
}
```

---

## ファイル API

### ファイル操作

```rust
// ファイル情報
FileInfo {
    path: PathBuf,         // ファイルパス
    name: String,          // ファイル名
    size: u64,             // ファイルサイズ
    is_dir: bool,          // ディレクトリかどうか
    modified: DateTime,    // 更新日時
}

// ファイル操作
enum FileAction {
    Read(PathBuf),              // 読み込み
    Write(PathBuf, String),     // 書き込み
    List(PathBuf),              // 一覧
    Search(String),             // 検索
}
```

---

## ツール API

### ツールコール

```rust
// ツール定義
ToolDefinition {
    name: String,           // ツール名
    description: String,    // 説明
    parameters: Value,      // JSON Schema
}

// ツールコール
ToolCall {
    id: String,             // コール ID
    name: String,           // ツール名
    arguments: Value,       // 引数 (JSON)
}

// ツール結果
ToolResult {
    call_id: String,        // コール ID
    content: String,        // 結果内容
    is_error: bool,         // エラーかどうか
}
```

---

## 設定 API

### アプリケーション設定

```rust
struct AppConfig {
    // プロバイダー
    provider: ProviderConfig,
    
    // UI 設定
    theme: String,              // テーマ名
    font_size: u16,             // フォントサイズ
    show_sidebar: bool,         // サイドバー表示
    show_tool_panel: bool,      // ツールパネル表示
    
    // エディタ設定
    vim_mode: bool,             // Vim モード
    tab_size: u16,              // タブサイズ
    word_wrap: bool,            // ワードラップ
    
    // モデル設定
    default_model: String,      // デフォルトモデル
    max_tokens: u32,            // 最大トークン数
    temperature: f32,           // 温度
}

struct ProviderConfig {
    provider_type: ProviderType, // Anthropic / OpenAICompatible
    api_key: String,             // API キー
    base_url: Option<String>,    // ベース URL
    model: String,               // モデル名
}
```

---

## イベント API

### イベントバス

```rust
enum AppEvent {
    // UI イベント
    KeyPress(KeyEvent),
    Resize(u16, u16),
    FocusChange(FocusTarget),
    
    // LLM イベント
    StreamStart,
    StreamChunk(StreamChunk),
    StreamEnd(Option<FinishReason>),
    StreamError(String),
    
    // セッションイベント
    SessionCreated(String),
    SessionLoaded(String),
    SessionSaved(String),
    
    // ツールイベント
    ToolCallStarted(ToolCall),
    ToolCallCompleted(ToolResult),
    
    // 通知イベント
    Toast(ToastType, String),
}
```

---

## エラーハンドリング

```rust
enum AppError {
    // LLM エラー
    LlmError(String),
    ProviderNotFound(String),
    InvalidApiKey,
    
    // ファイルエラー
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    IoError(io::Error),
    
    // セッションエラー
    SessionNotFound(String),
    SessionLoadError(String),
    
    // 設定エラー
    ConfigParseError(String),
    InvalidConfig(String),
}
```

---

## レスポンス形式

### 正常レスポンス

```json
{
  "success": true,
  "data": { ... },
  "message": "Operation completed successfully"
}
```

### エラーレスポンス

```json
{
  "success": false,
  "error": {
    "code": "INVALID_API_KEY",
    "message": "The provided API key is invalid",
    "details": { ... }
  }
}
```

---

## パフォーマンス要件

| 項目 | 要件 |
| --- | --- |
| 起動時間 | < 500ms |
| レスポンス表示 | < 100ms |
| メモリ使用量 | < 100MB |
| フレームレート | 60fps |

---

## セキュリティ

- API キーは環境変数で管理
- メモリ内のみに保持（永続化しない）
- ログに API キーを含めない

---

**Made with ❤️**
