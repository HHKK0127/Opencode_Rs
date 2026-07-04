# OpenCode Desktop - Zed UI System 適用計画

## 📊 Zed から学ぶべき主要な設計原則

### 1. **セマンティック色体系（Semantic Color System）**

**Zed の方法:**
- `#FF0000` ではなく `elementBackground`, `textMuted` という意味のある名前
- 明暗モード対応が容易
- 設計思想が一貫性を保つ

**OpenCode Desktop への適用:**
```css
/* 現在のシステム（改善前） */
--v2-grey-1100: #161616;
--bg-layer-1: #121212;

/* 改善後：セマンティック層を追加 */
:root {
  /* Tier 1: 基本トーン */
  --v2-grey-1100: #161616;
  
  /* Tier 2: セマンティック（意図） */
  --color-surface-background: var(--v2-grey-1100);
  --color-element-background: var(--v2-grey-1000);
  --color-element-hover: rgba(255, 255, 255, 0.06);
  --color-element-active: rgba(255, 255, 255, 0.10);
  
  /* Tier 3: コンポーネント用途 */
  --color-button-primary-bg: var(--color-element-background);
  --color-button-primary-hover: var(--color-element-hover);
  --color-input-border: var(--v2-border-border-base);
  --color-input-focus: var(--v2-border-border-focus);
}
```

### 2. **エレベーション（Z-index）システム**

**Zed の方法:**
```rust
ElevationIndex::Surface       // Z-0: 基本面（パネル、リスト）
ElevationIndex::Elevated      // Z+1: ポップオーバー、モーダル
ElevationIndex::Modal         // Z+2: 最前面モーダル
```

**OpenCode Desktop への適用:**
```css
/* Elevation定義 */
--z-surface: 0;           /* パネル、タブ、サイドバー */
--z-elevated: 100;        /* ポップオーバー、ドロップダウン */
--z-modal: 200;           /* モーダルダイアログ */
--z-tooltip: 300;         /* ツールチップ */

/* 対応するシャドウ */
--shadow-elevated: 0 4px 12px rgba(0, 0, 0, 0.3);
--shadow-modal: 0 8px 24px rgba(0, 0, 0, 0.5);
```

### 3. **コンポーネント構成パターン（Trait Composition）**

**Zed の方法：**
- 継承ではなく、トレイトを組み合わせる
- 各トレイトは単一の責務（Button, Clickable, Disableable等）
- 複数トレイトの実装で機能を追加

**React への適用パターン：**
```typescript
// 基本: カスタムフック（トレイト相当）
const useClickable = (props: ClickableProps) => ({
  onClick: props.onClick,
  onKeyDown: handleKeyDown,
  role: 'button',
  tabIndex: 0,
});

const useToggleable = (state: boolean, onChange: (v: boolean) => void) => ({
  checked: state,
  onChange,
  role: 'checkbox',
});

// コンポーネント実装：複数フックの組み合わせ
interface ButtonProps extends ClickableProps {
  variant?: 'primary' | 'secondary';
  disabled?: boolean;
}

function Button({ variant = 'primary', disabled, ...props }: ButtonProps) {
  const clickable = useClickable(props);
  
  return (
    <button
      {...clickable}
      disabled={disabled}
      className={`btn btn-${variant}`}
    />
  );
}

// トグルボタン: 複数トレイトの組み合わせ
interface ToggleButtonProps extends ClickableProps {
  value: boolean;
  onChange: (v: boolean) => void;
}

function ToggleButton({ value, onChange, ...clickProps }: ToggleButtonProps) {
  const clickable = useClickable({
    ...clickProps,
    onClick: () => onChange(!value),
  });
  const toggleable = useToggleable(value, onChange);
  
  return (
    <button
      {...clickable}
      {...toggleable}
      className={`toggle-btn ${value ? 'active' : ''}`}
    />
  );
}
```

### 4. **ビルダーパターン（Fluent API）**

**Zed の方法：**
```rust
div()
  .flex()
  .h_full()
  .bg(colors.surface)
  .p_4()
  .rounded_lg()
  .on_click(...)
```

**React/TypeScript への適用：**

```typescript
// 方法 1: CSS クラスベース（現在）
<div className="flex h-full bg-surface p-4 rounded-lg">

// 方法 2: Builder パターン + Tailwind（改善）
const Box = (props: BoxProps) => {
  const classes = [
    props.flex && 'flex',
    props.h_full && 'h-full',
    props.bg && `bg-${props.bg}`,
    props.p && `p-${props.p}`,
    props.rounded && `rounded-${props.rounded}`,
  ].filter(Boolean).join(' ');
  
  return <div className={classes} {...props} />;
};

// 使用例
<Box flex h_full bg="surface" p={4} rounded="lg" onClick={...} />

// さらに良い: 流暢な API（Builder）
class BoxBuilder {
  private styles: Record<string, string> = {};
  
  flex() { this.styles.display = 'flex'; return this; }
  h_full() { this.styles.height = '100%'; return this; }
  bg(color: string) { this.styles.background = `var(--color-${color})`; return this; }
  p(size: number) { this.styles.padding = `${size}px`; return this; }
  rounded(size: string) { this.styles.borderRadius = `var(--radius-${size})`; return this; }
  onClick(handler: () => void) { this.handler = handler; return this; }
  
  render() {
    return <div style={this.styles} onClick={this.handler} />;
  }
}

// 使用例
new BoxBuilder()
  .flex()
  .h_full()
  .bg('surface')
  .p(4)
  .rounded('lg')
  .onClick(handleClick)
  .render()
```

### 5. **スタイル適用の優先度システム**

**Zed の方法（優先度 低→高）:**
1. デフォルトスタイル
2. テーマトークン
3. コンポーネント固有スタイル
4. ユーザーカスタマイズ
5. 状態オーバーライド（hover, active, disabled）

**React への適用：**

```typescript
interface StyledProps {
  // 基本：コンポーネントのデフォルト
  variant?: 'primary' | 'secondary';
  
  // ユーザーカスタマイズ
  className?: string;
  style?: React.CSSProperties;
  
  // 状態
  disabled?: boolean;
  hovered?: boolean;
  active?: boolean;
}

function Button({ variant = 'primary', className, style, disabled, ...props }: StyledProps) {
  // Tier 1: デフォルトスタイル + テーマ
  const baseClasses = ['button', `button-${variant}`];
  
  // Tier 2: コンポーネント固有スタイル
  const componentStyle = {
    padding: '8px 16px',
    borderRadius: '6px',
  };
  
  // Tier 3: ユーザーカスタマイズ
  const userStyle = style || {};
  
  // Tier 4: 状態オーバーライド
  const stateStyle = disabled ? { opacity: 0.5, cursor: 'not-allowed' } : {};
  
  // 優先度順に merge
  const finalStyle = {
    ...componentStyle,
    ...userStyle,
    ...stateStyle,
  };
  
  return (
    <button
      className={[...baseClasses, className].filter(Boolean).join(' ')}
      style={finalStyle}
      disabled={disabled}
      {...props}
    />
  );
}
```

---

## 🚀 実装ロードマップ

### **Phase 1: セマンティック色体系への移行（1-2時間）**

✅ 目標：CSS 変数をセマンティックに再構成

**実装内容:**
```bash
tasks:
  1. src/styles/theme.css を拡張
     - Tier 2: セマンティック層を追加
     - Tier 3: コンポーネント用色を追加
  
  2. 既存コンポーネントの CSS を更新
     - --v2-grey-* から --color-* へ参照を切り替え
     - Sidebar.css, ChatContainer.css, Composer.css を更新
  
  3. テーマファイルを整理
     - src/styles/theme-tokens.css（新）
     - src/styles/theme-colors.css（新）
     - src/styles/theme-elevation.css（新）
```

### **Phase 2: コンポーネント構成パターンの導入（1-2時間）**

✅ 目標：再利用可能な UI フック・パターン化

**実装内容:**
```bash
tasks:
  1. src/hooks/ ディレクトリ作成
     - useClickable.ts（クリック可能パターン）
     - useHoverable.ts（ホバー状態）
     - useFocusable.ts（フォーカス管理）
  
  2. 既存コンポーネントをリファクタ
     - Composer を useClickable パターンで再実装
     - Button 系を統一ファクトリで実装
  
  3. エレベーション管理
     - src/styles/elevations.ts（Z-index 定義）
     - 自動シャドウ適用
```

### **Phase 3: ビルダーパターン（オプション・高度）**

✅ 目標：流暢な API でコンポーネント記述

**実装内容:**
```bash
tasks:
  1. src/components/builders/ ディレクトリ作成
     - BoxBuilder クラス
     - FlexBuilder クラス
     - ComponentBuilder 基底クラス
  
  2. マクロ化（TypeScript）
     - styled-components 系のユーティリティ
     - または Tailwind CSS 統合
```

---

## 📝 優先度付き実装チェックリスト

### **High Priority（すぐに実装）**

- [ ] **Phase 1.1:** CSS セマンティック変数追加
  - 例：`--color-element-hover`, `--color-surface-background`
  - 既存変数 `--v2-*` から参照

- [ ] **Phase 1.2:** コンポーネント CSS 更新
  - Sidebar.css → 新色変数を使用
  - ChatContainer.css → 新色変数を使用
  - Composer.css → 新色変数を使用

- [ ] **Phase 2.1:** useClickable フック実装
  ```typescript
  export const useClickable = (props: ClickableProps) => ({
    onClick: props.onClick,
    onKeyDown: handleKeyDown,
    role: 'button',
    tabIndex: 0,
  });
  ```

### **Medium Priority（次セッション）**

- [ ] **Phase 2.2:** Sidebar コンポーネント再構成
- [ ] **Phase 2.3:** 統一ボタンコンポーネント作成
- [ ] **Phase 3.1:** エレベーション CSS 追加

### **Low Priority（将来）**

- [ ] **Phase 3.2:** ビルダーパターン実装
- [ ] **Phase 3.3:** Tailwind CSS 完全統合
- [ ] **Phase 4:** Tauri + Rust 移行時に GPUI 参考化

---

## 📚 適用例：Sidebar の改善

**改善前:**
```typescript
// src/components/Sidebar.tsx
export const Sidebar: React.FC = () => (
  <aside data-component="sidebar">
    <div data-slot="sidebar-header">
      <h1>OpenCode</h1>
    </div>
    ...
  </aside>
)
```

**改善後（セマンティック色）:**
```typescript
// src/components/Sidebar.tsx
export const Sidebar: React.FC = () => (
  <aside 
    data-component="sidebar"
    style={{
      background: 'var(--color-surface-background)',
      borderRight: '1px solid var(--color-border-base)',
    }}
  >
    <div data-slot="sidebar-header">
      <h1>OpenCode</h1>
    </div>
    ...
  </aside>
)
```

**さらに改善（Builder パターン）:**
```typescript
// 将来の実装
export const Sidebar: React.FC = () => (
  <Box
    component="aside"
    flex_col()
    bg("surface")
    border_right(1)
    border_color("base")
    h_full()
    w(260)
  >
    <SidebarHeader />
    <SessionList />
    <SidebarFooter />
  </Box>
)
```

---

## 🎯 成功指標

**Phase 1 完了時:**
- ✅ CSS セマンティック変数が定義されている
- ✅ 全コンポーネント CSS が新変数を使用
- ✅ テーマ変更時の一貫性が向上

**Phase 2 完了時:**
- ✅ 再利用可能なフックが 3 個以上実装
- ✅ コンポーネント重複コードが 30% 削減
- ✅ 新しいコンポーネント追加が 2 倍高速化

**Phase 3 完了時:**
- ✅ ビルダーパターンで記述量が 50% 削減
- ✅ UI の一貫性が 90% 以上改善
- ✅ Tauri 移行時の参考実装が完成

---

## 🔗 参考ドキュメント

- Zed UI 完全分析：`docs/ZED_UI_ARCHITECTURE_ANALYSIS.md`
- Zed 参考ガイド：`docs/ZED_UI_QUICK_REFERENCE.md`
- OpenCode Desktop 現在設定：`src/styles/theme.css`

**次のステップ：Phase 1 を実装しましょう！** 🚀
