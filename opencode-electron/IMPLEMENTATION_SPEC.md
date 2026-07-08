# OpenCode Desktop - Phase 0 実装仕様書

**目的**: セキュリティ修正済みの SolidJS + Electron プロジェクト scaffold 生成  
**対象フェーズ**: Phase 0（セキュリティ修正・技術スタック確定）  
**見積時間**: 1.5 時間

---

## 📋 必要なファイル一覧

### プロジェクト設定（6ファイル）
1. `package.json` — 依存関係（Zustand なし、SolidJS + Electron）
2. `.env.example` — 環境変数テンプレート（暗号化キー等）
3. `tsconfig.json` — TypeScript 設定（SolidJS 対応）
4. `tsconfig.node.json` — Node.js 用 TypeScript 設定
5. `vite.config.ts` — Vite + vite-plugin-solid + vite-plugin-electron
6. `.gitignore` — Git 除外ファイル

### Electron メインプロセス（1ファイル）
7. `electron/main/index.ts` — BrowserWindow 作成、IPC ハンドラー、electron-store 初期化

### Electron Preload（1ファイル）
8. `electron/preload/index.ts` — contextBridge API、型定義

### Renderer（SolidJS）（3ファイル）
9. `src/renderer/index.html` — CSP メタタグ含む HTML
10. `src/renderer/main.tsx` — SolidJS エントリーポイント
11. `src/renderer/App.tsx` — ルート定義

### 状態管理（SolidJS Store）（2ファイル）
12. `src/renderer/store/auth.ts` — 認証状態（SolidJS createStore）
13. `src/renderer/store/ui.ts` — UI 状態（ダークモード等）

### API 層（2ファイル）
14. `src/renderer/services/api.ts` — HTTP クライアント
15. `src/renderer/types/api.ts` — API レスポンス型定義

### ドキュメント（2ファイル）
16. `README.md` — プロジェクト説明、セットアップ手順
17. `.env` — 開発用環境変数（gitignore）

**合計**: 17 ファイル

---

## 🔐 セキュリティ要件（Phase 0 で解決）

### 1. 暗号化キーの環境変数化
- **問題**: ソースコードにハードコーディング
- **解決**: `process.env.OPENCODE_ENCRYPTION_KEY` 使用
- **実装場所**: `electron/main/index.ts`, `.env.example`
- **要件**:
  ```typescript
  const ENCRYPTION_KEY = process.env.OPENCODE_ENCRYPTION_KEY;
  if (!ENCRYPTION_KEY && IS_PRODUCTION) {
    dialog.showErrorBox('Configuration Error', '...');
    app.quit();
  }
  ```

### 2. sandbox: true 設定
- **問題**: `sandbox: false` でセキュリティ弱体化
- **解決**: `webPreferences.sandbox: true` に変更
- **実装場所**: `electron/main/index.ts`
- **要件**:
  ```typescript
  webPreferences: {
    sandbox: true, // ← true に変更
    contextIsolation: true,
    nodeIntegration: false
  }
  ```

### 3. CSP（Content Security Policy）
- **問題**: XSS 攻撃のリスク
- **解決**: `index.html` に CSP メタタグ追加
- **実装場所**: `src/renderer/index.html`
- **要件**:
  ```html
  <meta http-equiv="Content-Security-Policy" 
    content="default-src 'self'; 
             connect-src 'self' http://localhost:8080 ws://localhost:8080; 
             script-src 'self'; 
             style-src 'self' 'unsafe-inline'; 
             img-src 'self' data: https:;">
  ```

### 4. IPC チャネルの型安全化
- **問題**: 全キーに対して操作可能
- **解決**: 特定キーのみ許可
- **実装場所**: `electron/preload/index.ts`
- **要件**:
  ```typescript
  type StoreKey = 'token' | 'refreshToken' | 'user' | 'windowBounds';
  
  contextBridge.exposeInMainWorld('electronAPI', {
    store: {
      get: (key: StoreKey) => ipcRenderer.invoke('store:get', key),
      set: (key: StoreKey, value: any) => ipcRenderer.invoke('store:set', key, value),
      delete: (key: StoreKey) => ipcRenderer.invoke('store:delete', key)
    }
  });
  ```

---

## 📦 技術スタック詳細

### 依存関係（package.json）
```json
{
  "dependencies": {
    "@solidjs/router": "^0.15.0",
    "@tanstack/solid-query": "^5.0.0",
    "electron-log": "^5.0.0",
    "electron-store": "^9.0.0",
    "solid-js": "^1.9.0"
  },
  "devDependencies": {
    "dotenv": "^16.4.0",
    "electron": "^31.0.0",
    "electron-builder": "^25.0.0",
    "typescript": "^5.4.0",
    "vite": "^5.0.0",
    "vite-plugin-electron": "^0.29.0",
    "vite-plugin-solid": "^2.10.0"
  }
}
```

**重要**: ❌ `zustand` は含めない（SolidJS Store を使用）

---

## 📁 ディレクトリ構造

```
opencode-electron/
├── electron/
│   ├── main/
│   │   └── index.ts          # メインプロセス（BrowserWindow、IPC）
│   └── preload/
│       └── index.ts          # Preload スクリプト（contextBridge）
│
├── src/renderer/
│   ├── index.html            # HTML + CSP メタタグ
│   ├── main.tsx              # SolidJS エントリーポイント
│   ├── App.tsx               # ルート定義
│   ├── store/
│   │   ├── auth.ts           # 認証状態（SolidJS createStore）
│   │   └── ui.ts             # UI 状態
│   ├── services/
│   │   └── api.ts            # HTTP クライアント
│   └── types/
│       └── api.ts            # API 型定義
│
├── package.json              # 依存関係（Zustand なし）
├── .env.example              # 環境変数テンプレート
├── .env                      # 開発用環境変数（gitignore）
├── tsconfig.json             # TypeScript 設定
├── tsconfig.node.json        # Node.js 用 TS 設定
├── vite.config.ts            # Vite 設定
├── .gitignore                # Git 除外
└── README.md                 # プロジェクト説明
```

---

## 🔧 各ファイルの仕様

### 1. package.json
- **scripts**: `dev`, `build`, `build:win`, `build:mac`, `build:linux`
- **dependencies**: SolidJS, Solid Query, electron-store, electron-log（Zustand なし）
- **devDependencies**: Electron 31, Vite, TypeScript, vite-plugin-solid, vite-plugin-electron, dotenv
- **build**: electron-builder 設定（appId, productName, targets）

### 2. .env.example
```bash
OPENCODE_ENCRYPTION_KEY=your-secure-random-key-minimum-32-characters
NODE_ENV=development
OPENCODE_API_URL=http://localhost:8080
```

### 3. tsconfig.json
- **compilerOptions**: `target: ESNext`, `jsx: preserve`, `jsxImportSource: solid-js`
- **include**: `["src", "electron"]`
- **strict**: `true`, `noUnusedLocals: true`, `noUnusedParameters: true`

### 4. vite.config.ts
- **plugins**: `solid()`, `electron([main, preload])`
- **resolve.alias**: `@: ./src`
- **build.target**: `esnext`

### 5. electron/main/index.ts
- **暗号化キー**: `process.env.OPENCODE_ENCRYPTION_KEY` 使用、本番で未設定時エラー
- **electron-store**: StoreSchema 型定義、encryptionKey 設定
- **BrowserWindow**: sandbox:true, contextIsolation:true, nodeIntegration:false
- **IPC ハンドラー**: `store:get`, `store:set`, `store:delete`（型安全）
- **ウィンドウ状態保存**: `windowBounds` を store に保存

### 6. electron/preload/index.ts
- **contextBridge**: `electronAPI` を window に公開
- **型定義**: `StoreKey = 'token' | 'refreshToken' | 'user' | 'windowBounds'`
- **API**: `store.get()`, `store.set()`, `store.delete()`, `platform`, `versions`
- **global 型拡張**: `declare global { interface Window { electronAPI: ElectronAPI } }`

### 7. src/renderer/index.html
- **CSP メタタグ**: `connect-src` に `http://localhost:8080` 許可
- **charset**: UTF-8
- **viewport**: `width=device-width, initial-scale=1.0`
- **div#root**: SolidJS マウントポイント
- **script**: `<script type="module" src="./main.tsx"></script>`

### 8. src/renderer/main.tsx
- **imports**: `render` from `solid-js/web`, `Router`, `QueryClientProvider`
- **QueryClient**: `staleTime: 5分`, `retry: 3回`, `refetchOnWindowFocus: false`
- **render**: `<QueryClientProvider><Router><App /></Router></QueryClientProvider>`

### 9. src/renderer/App.tsx
- **imports**: `Routes`, `Route` from `@solidjs/router`
- **ルート**: `/` (仮の Hello OpenCode 表示)
- **後続**: Phase 2 で LoginPage, DashboardPage 追加

### 10. src/renderer/store/auth.ts
- **createStore**: `solid-js/store` の `createStore` 使用
- **AuthState**: `token`, `refreshToken`, `user`, `isAuthenticated`
- **authActions**: `setToken()`, `setUser()`, `logout()`
- **export**: `{ authState, authActions }`

### 11. src/renderer/store/ui.ts
- **createStore**: UI 状態（`sidebarOpen`, `darkMode`）
- **uiActions**: `toggleSidebar()`, `toggleDarkMode()`

### 12. src/renderer/services/api.ts
- **API_BASE_URL**: `http://localhost:8080/api/v1`
- **fetchWithAuth**: タイムアウト 10 秒、401 時リフレッシュ処理（Phase 2 で実装）
- **api.auth.login**: `POST /auth/login`（Phase 2 で実装）
- **api.files.list**: `GET /files`（Phase 3 で実装）

### 13. src/renderer/types/api.ts
```typescript
// API レスポンス型
export interface LoginResponse {
  token: string;
  refresh_token: string;
  expires_in: number;
}

export interface FileItem {
  id: string;
  filename: string;
  size: number;
  mime_type: string;
  created_at: string;
}

export interface FilesResponse {
  files: FileItem[];
  total: number;
  page: number;
  per_page: number;
}

export interface ErrorResponse {
  error: string;
  message: string;
}
```

### 14. .gitignore
```
node_modules/
dist/
dist-electron/
release/
*.log
.env
.DS_Store
```

### 15. README.md
- **プロジェクト説明**: OpenCode Desktop (SolidJS + Electron)
- **セットアップ手順**: `npm install`, `.env` 作成, `npm run dev`
- **セキュリティ**: 暗号化キー設定方法
- **ビルド**: `npm run build:win` 等

---

## ✅ Phase 0 完成条件

以下がすべて満たされたら Phase 0 完了：

1. ✅ 17 ファイルすべて作成完了
2. ✅ `npm install` エラーなし
3. ✅ セキュリティ 3 項目修正済み:
   - 暗号化キー環境変数化
   - sandbox: true
   - CSP メタタグ
4. ✅ Zustand → SolidJS Store 移行
5. ✅ 型定義完備（ElectronAPI, StoreKey, API レスポンス）
6. ✅ レビュースコア: B+ → A-

---

## 🚀 Phase 1 への引き継ぎ事項

Phase 0 完了後、Phase 1 で以下を実装：

- **タスク**: `npm run dev` で Electron ウィンドウ起動
- **表示内容**: "Hello OpenCode" が表示される
- **動作確認**: DevTools でコンソールエラーなし
- **バックエンド接続**: `curl http://localhost:8080/health` 成功確認

---

## 📝 実装時の注意事項

### TypeScript 型安全性
- すべての関数に戻り値の型を明示
- `any` の使用を最小限に（`StoreSchema` の `value` のみ）
- Preload の `ElectronAPI` インターフェースを `global.d.ts` に配置

### セキュリティチェックリスト
- [ ] 暗号化キーが環境変数から読み込まれている
- [ ] 本番環境で暗号化キー未設定時にエラーメッセージ表示
- [ ] sandbox: true 設定済み
- [ ] CSP メタタグが適切に設定されている
- [ ] IPC チャネルが型安全（StoreKey 限定）
- [ ] DevTools が本番環境で無効化されている

### エラーハンドリング
- Main プロセス: `uncaughtException` でエラーダイアログ表示
- Renderer プロセス: ErrorBoundary（Phase 2 で実装）
- API 呼び出し: try-catch でエラーハンドリング（Phase 2 で実装）

---

## 📞 代理 AI への依頼事項

上記仕様に従って、以下の 17 ファイルを生成してください：

1. package.json
2. .env.example
3. tsconfig.json
4. tsconfig.node.json
5. vite.config.ts
6. .gitignore
7. electron/main/index.ts
8. electron/preload/index.ts
9. src/renderer/index.html
10. src/renderer/main.tsx
11. src/renderer/App.tsx
12. src/renderer/store/auth.ts
13. src/renderer/store/ui.ts
14. src/renderer/services/api.ts
15. src/renderer/types/api.ts
16. README.md
17. .env（開発用）

**重要**: 
- セキュリティ 3 項目（暗号化キー環境変数化、sandbox:true、CSP）を必ず実装
- Zustand は使用しない（SolidJS Store のみ）
- 型定義を厳密に（ElectronAPI, StoreKey, API レスポンス）
