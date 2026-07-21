# OpenCode TUI ユーザーガイド

---

**最終更新**: 2026-07-18  
**バージョン**: 1.0.0  
**対象**: opencode_tui (Ratatui TUI)

---

## 目次

1. [はじめに](#はじめに)
2. [インストール](#インストール)
3. [起動方法](#起動方法)
4. [基本操作](#基本操作)
5. [チャット機能](#チャット機能)
6. [エディタ機能](#エディタ機能)
7. [設定画面](#設定画面)
8. [スラッシュコマンド](#スラッシュコマンド)
9. [テーマ切替](#テーマ切替)
10. [トラブルシューティング](#トラブルシューティング)

---

## はじめに

OpenCode TUI は、ターミナル上で動作する AI チャットアプリケーションです。GitHub Copilot 風の UI を目指し、以下の機能を提供します:

- LLM との対話（Anthropic / OpenAI 互換）
- ストリーミング応答
- Markdown レンダリング
- Vim キーバインド
- テーマシステム
- ファイル参照

---

## インストール

### 前提条件

- Rust 1.85+ (stable)
- Windows / macOS / Linux
- LLM プロバイダーの API キー

### ビルド方法

```powershell
# リポジトリクローン
git clone https://github.com/HHKK0127/Opencode_Rs.git
cd OpenCode_Rs

# ビルド
cargo build --bin opencode_tui

# バイナリは target/debug/opencode_tui.exe に生成
```

---

## 起動方法

### 基本起動

```powershell
# API キー設定（必須）
$env:ANTHROPIC_API_KEY = "sk-ant-api03-..."

# 起動
cargo run --bin opencode_tui
```

### OpenAI 互換プロバイダー

```powershell
# OpenAI
$env:OPENAI_API_KEY = "sk-..."

# カスタムプロバイダー
$env:OPENAI_API_KEY = "your-key"
$env:OPENAI_BASE_URL = "https://api.your-provider.com/v1"
```

### 環境変数一覧

| 変数名 | 説明 | 必須 |
| --- | --- | --- |
| `ANTHROPIC_API_KEY` | Anthropic API キー | ※1 |
| `OPENAI_API_KEY` | OpenAI API キー | ※1 |
| `OPENAI_BASE_URL` | OpenAI 互換のベース URL | 任意 |

※1: いずれか一方が必要

---

## 基本操作

### 画面構成

```
┌─────────────────────────────────────────────────────────┐
│ [1] Logo │ [2] Status Bar │ [3] Model │ [4] Token Usage │
├─────────────────────────────────────────────────────────┤
│                                                         │
│                   [5] チャット画面                       │
│                   (メッセージ一覧)                       │
│                                                         │
├─────────────────────────────────────────────────────────┤
│                   [6] 入力エリア                         │
│                   (プロンプト)                           │
└─────────────────────────────────────────────────────────┘
```

### キーバインド一覧

#### グローバル

| キー | 操作 |
| --- | --- |
| `Ctrl+Q` | アプリ終了 |
| `Ctrl+K` | 設定画面 |
| `Ctrl+S` | サイドバー表示/非表示 |
| `Ctrl+T` | ツールパネル表示/非表示 |
| `Ctrl+P` | モデルピッカー |
| `F1` | ヘルプ表示 |

#### チャット画面

| キー | 操作 |
| --- | --- |
| `Enter` | メッセージ送信 |
| `Ctrl+M` | マルチライン入力トグル |
| `Up/Down` | スクロール |
| `PageUp/PageDown` | ページスクロール |
| `Ctrl+L` | ツールコール展開/折りたたみ |

#### エディタ

| キー | 操作 |
| --- | --- |
| `Ctrl+E` | エディタ表示/非表示 |
| `Tab` | エディタ ↔ チャット フォーカス切替 |
| `Esc` | Normal モード |
| `i` | Insert モード |

---

## チャット機能

### メッセージ送信

1. 入力エリアにテキストを入力
2. `Enter` で送信
3. ストリーミングで応答が表示される

### マルチライン入力

1. `Ctrl+M` でマルチラインモードに切替
2. 複数行のテキストを入力
3. `Alt+Enter` で送信

### @mention

入力中に `@` を入力すると補完が表示されます:

| mention | 説明 |
| --- | --- |
| `@file` | ファイル参照 |
| `@code` | コード参照 |
| `@history` | 履歴参照 |
| `@model` | モデル切替 |

### ツールコール

LLM がツールを使用した場合:

1. ツールパネルに表示（`Ctrl+T` で切替）
2. `Ctrl+L` で展開/折りたたみ
3. 各ツールの実行結果を確認

---

## エディタ機能

### Vim モード

エディタは Vim スタイルのキーバインドをサポート:

#### Normal モード

| キー | 操作 |
| --- | --- |
| `h` | カーソル左 |
| `j` | カーソル下 |
| `k` | カーソル上 |
| `l` | カーソル右 |
| `0` | 行頭 |
| `$` | 行末 |
| `i` | Insert モード |
| `A` | 行末で Insert モード |

#### オペレータ + モーション

| キー | 操作 |
| --- | --- |
| `dd` | 行削除 |
| `yy` | 行ヤンク |
| `cc` | 行変更 |
| `d$` | 行末まで削除 |
| `d0` | 行頭まで削除 |

#### カウントプレフィックス

| キー | 操作 |
| --- | --- |
| `3j` | 3行下に移動 |
| `5k` | 5行上に移動 |
| `2dd` | 2行削除 |

---

## 設定画面

`Ctrl+K` で設定画面を表示:

### 設定項目

| フィールド | 説明 |
| --- | --- |
| **Provider** | LLM プロバイダー（Anthropic / OpenAI） |
| **API Key** | API キー |
| **Model** | モデル名 |
| **Max Tokens** | 最大トークン数 |
| **Temperature** | 温度パラメータ |
| **Theme** | テーマ |

### 操作方法

| キー | 操作 |
| --- | --- |
| `Tab` / `BackTab` | フィールド移動 |
| `Enter` | 値を編集 |
| `Esc` | 保存して閉じる |

---

## スラッシュコマンド

入力エリアで `/` から始まるコマンド:

| コマンド | 説明 |
| --- | --- |
| `/help` | ヘルプ表示 |
| `/clear` | チャット履歴クリア |
| `/model` | モデル切替 |
| `/files` | ファイル一覧表示 |
| `/compact` | メッセージ圧縮 |

### 使用例

```
/help
→ ヘルプメッセージを表示

/clear
→ チャット履歴をクリア

/model claude-sonnet-4-20250514
→ モデルを切替

/files
→ プロジェクトのファイル一覧を表示
```

---

## テーマ切替

### 利用可能なテーマ

| テーマ名 | 説明 |
| --- | --- |
| `dark` | デフォルトダークテーマ |
| `light` | ライトテーマ |
| `gruvbox-dark` | Gruvbox ダーク |
| `gruvbox-light` | Gruvbox ライト |
| `solarized-dark` | Solarized ダーク |
| `solarized-light` | Solarized ライト |
| `nord` | Nord |
| `catppuccin-mocha` | Catppuccin Mocha |
| `catppuccin-latte` | Catppuccin Latte |
| `tokyo-night` | Tokyo Night |

### 切替方法

1. `Ctrl+K` で設定画面
2. `Tab` で「Theme」フィールドに移動
3. `Enter` でテーマ選択
4. `Esc` で適用

---

## トラブルシューティング

### Q: 起動時にエラーが発生する

**A**: API キーが設定されているか確認してください:

```powershell
echo $env:ANTHROPIC_API_KEY
```

### Q: 応答が返ってこない

**A**: 以下の点を確認:

1. インターネット接続
2. API キーの有効性
3. プロバイダーのステータス

### Q: 文字化けが発生する

**A**: ターミナルのフォントを Nerd Font に変更してください:

- Windows Terminal: 設定 → プロファイル → フォント
- 推奨: JetBrains Mono, Fira Code

### Q: テーマが反映されない

**A**: `Ctrl+K` で設定画面を開き、`Esc` で閉じてください。

### Q: Vim モードで入力できない

**A**: `i` で Insert モードに切替えてください。`Esc` で Normal モードに戻ります。

---

## 参考リンク

- [README.md](../README.md) - プロジェクト概要
- [API 仕様書](./API_TUI_SPECIFICATION.md) - API 詳細
- [GitHub](https://github.com/HHKK0127/Opencode_Rs) - ソースコード

---

**Made with ❤️**
