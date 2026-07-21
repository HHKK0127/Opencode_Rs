# OpenCode TUI モジュール構成

---

**最終更新**: 2026-07-18  
**バージョン**: 1.0.0  
**対象**: opencode_tui (Ratatui TUI)

---

## 概要

`opencode_tui.rs`（約6000行）を以下のモジュールに分割します。

---

## モジュール一覧

### 1. `models.rs` — データモデル

**行数**: 約150行  
**責務**: チャットメッセージ、ツールコール、設定のデータ構造

```rust
// 含まれる型
ChatMessage
ChatRole
ToolCall
ToolCallStatus
AppConfig
ProviderConfig
Screen
FocusTarget
```

---

### 2. `theme.rs` — テーマシステム

**行数**: 約400行  
**責務**: テーマ定義、カラーパレット

```rust
// 含まれる型
ThemeColors
Theme
Theme::from_name()
Theme::available_themes()
```

**対応テーマ**:
- dark, light
- gruvbox-dark, gruvbox-light
- solarized-dark, solarized-light
- nord
- catppuccin-mocha, catppuccin-latte
- tokyo-night

---

### 3. `keymap.rs` — キーマップ & Vim

**行数**: 約300行  
**責務**: キーバインディング、VimFSA、InputClassifier

```rust
// 含まれる型
KeyBinding
Keymap<T>
KeymapMatch
KeyAction
VimMode
VimFSA
TextAction
InputCategory
classify_input()
create_vim_keymap()
```

---

### 4. `completion.rs` — 補完エンジン

**行数**: 約200行  
**責務**: ファジー検索、補完管理

```rust
// 含まれる型
CompletionType
CompletionItem
CompletionEngine
FuzzyMatch
fuzzy_score()
```

---

### 5. `components.rs` — UI コンポーネント

**行数**: 約300行  
**責務**: View+Element、Plugin、Toast、Event Bubbling

```rust
// 含まれる型
View (trait)
Element<V>
VStack
HStack
Plugin (trait)
PluginManager
ToastType
Toast
ToastManager
EventResult
BubbleEvent
```

---

### 6. `task_queue.rs` — タスクキュー

**行数**: 約100行  
**責務**: 前景/背景タスク管理

```rust
// 含まれる型
BackgroundTask
TaskStatus
TaskQueue
```

---

### 7. `markdown.rs` — Markdown レンダリング

**行数**: 約100行  
**責務**: コードブロック、見出しの描画

```rust
// 含まれる関数
render_markdown_line()
render_code_block()
```

---

### 8. `sidebar.rs` — チャットサイドバー

**行数**: 約400行  
**責務**: サイドバーの描画と操作

```rust
// 含まれる関数
render_chat_sidebar()
render_sidebar_section()
```

---

### 9. `dashboard.rs` — ダッシュボード

**行数**: 約800行  
**責務**: ダッシュボード画面の描画

```rust
// 含まれる関数
render_dashboard()
render_metrics()
render_chart()
render_table()
```

---

### 10. `approval.rs` — 承認オーバーレイ

**行数**: 約100行  
**責務**: ツール実行承認の UI

```rust
// 含まれる関数
render_approval_overlay()
```

---

### 11. `app.rs` — アプリケーションロジック

**行数**: 約2000行  
**責務**: App 構造体、イベント処理、メインループ

```rust
// 含まれる型/関数
App
App::new()
App::handle_event()
App::tick()
run_app()
```

---

## 依存関係

```
app.rs
├── models.rs
├── theme.rs
├── keymap.rs
├── completion.rs
├── components.rs
├── task_queue.rs
├── markdown.rs
├── sidebar.rs
├── dashboard.rs
└── approval.rs
```

---

## 分割手順

### Phase 1: モジュール作成

1. `src/bin/` ディレクトリにモジュールファイル作成
2. 各セクションを対応するモジュールに移動
3. `mod` 文で宣言

### Phase 2: インポート整理

1. 公開 API を `pub` で宣言
2. 不要な `use` 文を削除
3. 循環依存を解消

### Phase 3: テスト移行

1. 各モジュールにテストを移動
2. `cargo test` で動作確認

### Phase 4: ドキュメント更新

1. モジュールレベルのドキュメント追加
2. README 更新

---

## 注意事項

- `App` 構造体は他のモジュールから参照されるため、最後に分割
- `Theme` は多くのモジュールで使用されるため、先に分割
- テストは各モジュールに分散配置

---

**Made with ❤️**
