# OpenCode TUI パフォーマンスプロファイリング

---

**最終更新**: 2026-07-18  
**バージョン**: 1.0.0  
**対象**: opencode_tui (Ratatui TUI)

---

## 概要

TUI のパフォーマンスを測定・最適化するためのガイドラインです。

---

## パフォーマンス要件

| 項目 | 目標値 | 現状 |
| --- | --- | --- |
| 起動時間 | < 500ms | 測定中 |
| フレームレート | 60fps | 測定中 |
| メモリ使用量 | < 100MB | 測定中 |
| レスポンス表示 | < 100ms | 測定中 |
| レイアウト計算 | < 1ms | 測定中 |

---

## プロファイリング方法

### 1. cargo flamegraph

```powershell
# インストール
cargo install flamegraph

# プロファイリング実行
cargo flamegraph --bin opencode_tui

# 結果: flamegraph.svg が生成される
```

### 2. perf (Linux)

```bash
# プロファイリング実行
perf record --call-graph dwarf ./target/debug/opencode_tui

# 結果表示
perf report
```

### 3. Windows Performance Analyzer

```powershell
# プロファイリング実行
wpr -start GeneralProfile -start CPU

# アプリ実行
.\target\debug\opencode_tui.exe

# 停止
wpr -stop output.etl

# 結果表示
wpa output.etl
```

---

## ボトルネック箇所

### 1. レンダリング

**問題**: 毎フレームの再計算

**対策**:
- PositionCache でレイアウト位置をキャッシュ
- 差分更新（変更があった部分のみ再描画）

### 2. Markdown パース

**問題**: 大きなコードブロックのパース遅延

**対策**:
- 行数制限（100行で打ち切り）
- 遅延パース（表示時のみパース）

### 3. LLM ストリーミング

**問題**: チャンク処理のオーバーヘッド

**対策**:
- バッファリング（複数チャンクをまとめて処理）
- 非同期処理（UI ブロックなし）

### 4. ファジー検索

**問題**: 大量アイテムの検索遅延

**対策**:
- インデックス作成
- 結果キャッシュ

---

## 最適化最適化済み機能

### PositionCache

```rust
struct PositionCache {
    chat_height: u16,
    input_height: u16,
    sidebar_width: u16,
    tool_panel_width: u16,
    terminal_size: (u16, u16),
}

impl PositionCache {
    fn invalidate(&mut self) {
        self.terminal_size = (0, 0);
    }
    
    fn is_valid_for(&self, size: (u16, u16)) -> bool {
        self.terminal_size == size
    }
}
```

### TaskQueue

```rust
struct TaskQueue {
    pending: VecDeque<BackgroundTask>,
    running: HashMap<String, TaskStatus>,
    completed: Vec<(String, TaskStatus)>,
}
```

### CompletionEngine

```rust
struct CompletionEngine {
    items: Vec<CompletionItem>,
    selected_index: usize,
    completion_type: Option<CompletionType>,
}
```

---

## ベンチマーク

### レンダリングベンチマーク

```rust
#[cfg(test)]
mod benchmarks {
    use test::Bencher;
    
    #[bench]
    fn bench_render_chat(b: &mut Bencher) {
        let app = App::new();
        let mut terminal = setup_terminal();
        
        b.iter(|| {
            terminal.draw(|frame| {
                render_chat(frame, frame.area(), &app);
            }).unwrap();
        });
    }
    
    #[bench]
    fn bench_render_sidebar(b: &mut Bencher) {
        let app = App::new();
        let mut terminal = setup_terminal();
        
        b.iter(|| {
            terminal.draw(|frame| {
                render_sidebar(frame, frame.area(), &app);
            }).unwrap();
        });
    }
}
```

### ファジー検索ベンチマーク

```rust
#[bench]
fn bench_fuzzy_search(b: &mut Bencher) {
    let items = generate_test_items(1000);
    let engine = CompletionEngine::new();
    
    b.iter(|| {
        engine.update("test query", &items);
    });
}
```

---

## メモリ使用量

### 測定方法

```powershell
# Windows
Get-Process opencode_tui | Select-Object WorkingSet64

# Linux
ps -o rss= -p <pid>
```

### メモリ最適化

1. **メッセージ履歴**: 古いメッセージをアーカイブ
2. **テーマキャッシュ**: テーマカラーをキャッシュ
3. **画像データ**: Base64 エンコードを避ける

---

## CPU 使用率

### 測定方法

```powershell
# Windows
Get-Process opencode_tui | Select-Object CPU

# Linux
top -p <pid>
```

### CPU 最適化

1. **イベントループ**: 適切なスリープ時間設定
2. **レンダリング**: 差分更新
3. **パース**: 遅延パース

---

## 今後の課題

| 説明 | 優先度 | 状態 |
| --- | --- | --- |
| フレームレート監視 | 高 | ⏳ |
| メモリリーク検出 | 高 | ⏳ |
| レンダリング最適化 | 中 | ⏳ |
| パース最適化 | 中 | ⏳ |

---

**Made with ❤️**
