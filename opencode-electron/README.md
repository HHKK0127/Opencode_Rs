# OpenCode Desktop

SolidJS + Electron デスクトップアプリケーション

OpenCode_Rs バックエンドと連携する開発者向けデスクトップクライアント。

## ステータス

**Phase 1 完了 (2026-07-08)** ✅

- ✅ Phase 0: セキュリティ修正済み scaffold (Response Envelope 対応)
- ✅ Phase 1: Electron セットアップ + 起動確認

## 技術スタック

| カテゴリ | 技術 | バージョン |
|---------|------|-----------|
| Frontend | SolidJS | 1.9.x |
| Language | TypeScript | 5.4.x (strict mode) |
| Desktop | Electron | 31.x |
| State | SolidJS Store + TanStack Solid Query | 5.x |
| Build | Vite | 5.x |
| Plugin | vite-plugin-electron | 0.29.x |
| Storage | electron-store (暗号化対応) | 9.x |

## 必要環境

- **Node.js**: >= 20.11.0
- **npm**: >= 10.2.4
- **OS**: Windows 10+, macOS 11+, Ubuntu 20.04+

## セットアップ

### 1. 依存関係インストール

```bash
cd opencode-electron
npm install
```

### 2. 環境変数設定

`.env.example` を参考に `.env` を作成:

```bash
cp .env.example .env
```

**重要**: `OPENCODE_ENCRYPTION_KEY` は本番環境で必須です。

生成方法:

```bash
# PowerShell
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"

# OpenSSL
openssl rand -hex 32
```

### 3. 開発サーバー起動

```bash
npm run dev
```

Vite dev server と Electron ウィンドウが自動的に起動します。

- Vite: http://localhost:5173/
- Electron: 1200x800 ウィンドウ

## ビルド

### Windows

```bash
npm run build:win
```

成果物: `dist/` ディレクトリに `.exe` インストーラ

### macOS

```bash
npm run build:mac
```

成果物: `dist/` ディレクトリに `.dmg`

### Linux

```bash
npm run build:linux
```

成果物: `dist/` ディレクトリに `.AppImage` / `.deb`

### 全プラットフォーム

```bash
npm run build
```

## 開発コマンド

| コマンド | 説明 |
|---------|------|
| `npm run dev` | Vite dev server + Electron 起動 |
| `npx tsc --noEmit` | TypeScript 型チェック（0 errors 必須） |
| `npm run build` | 本番ビルド + electron-builder |
| `npm run preview` | ビルド成果物のプレビュー |

## セキュリティ機能

- ✅ 暗号化キーの環境変数化 (`OPENCODE_ENCRYPTION_KEY`)
- ✅ Electron `sandbox: true` 有効化
- ✅ Content Security Policy (CSP) 設定
- ✅ IPC チャネルの型安全化 (`StoreKey` union)
- ✅ electron-store 暗号化 (AES-256)
- ✅ JWT 自動リフレッシュ (401 検出 → 1回リトライ)
- ✅ リクエストタイムアウト (10秒, AbortController)
- ✅ `contextIsolation: true`, `nodeIntegration: false`

## API 仕様 (Response Envelope)

すべての API レスポンスは `ApiResponse<T>` 型でラップ:

```typescript
// 成功
{
  status: "success",
  data: T,
  error: null,
  timestamp?: string
}

// エラー (別形式)
{
  code: number,
  error: string
}
```

詳細は `src/renderer/types/api.ts` を参照。

## ディレクトリ構造

```
opencode-electron/
├── electron/
│   ├── main/
│   │   └── index.ts          # Electron メインプロセス
│   └── preload/
│       └── index.ts          # Preload スクリプト (IPC 橋渡し)
├── src/
│   └── renderer/
│       ├── App.tsx           # ルートコンポーネント
│       ├── main.tsx          # SolidJS エントリ
│       ├── index.html        # HTML テンプレート
│       ├── services/
│       │   └── api.ts        # API クライアント (Response Envelope 対応)
│       ├── store/
│       │   ├── auth.ts       # 認証状態 (SolidJS Store)
│       │   └── ui.ts         # UI 状態
│       └── types/
│           ├── api.ts        # API 型定義 (ApiResponse<T>)
│           └── electron.d.ts # Electron 型定義
├── dist-electron/            # ビルド成果物 (main, preload)
├── dist/                     # レンダラービルド成果物
├── index.html                # Vite エントリ HTML
├── vite.config.ts            # Vite + Electron 設定
├── tsconfig.json             # TypeScript 設定
├── package.json              # 依存関係
├── .env.example              # 環境変数テンプレート
└── README.md                 # このファイル
```

## 開発フェーズ

| フェーズ | 内容 | 状態 |
|---------|------|:----:|
| Phase 0 | セキュリティ修正済み scaffold (Response Envelope 対応) | ✅ |
| Phase 1 | Electron セットアップ + 起動確認 | ✅ |
| Phase 2 | 認証画面・バックエンド接続 | ⏳ |
| Phase 3 | ファイルエクスプローラー・エディタ | ⏳ |
| Phase 4 | メニュー・ショートカット・IPC | ⏳ |
| Phase 5 | テスト・ビルド・デプロイ | ⏳ |

## トラブルシューティング

### `npm run dev` で画面が白い

1. `Ctrl+C` で停止
2. `rm -rf node_modules package-lock.json`
3. `npm install` 再実行
4. `npm run dev` 再実行

### `Error: EADDRINUSE: address already in use :::5173`

ポート 5173 が使用中。`vite.config.ts` の `server.port` を変更:

```typescript
server: {
  port: 5174
}
```

### TypeScript エラー

```bash
npx tsc --noEmit
```

エラー内容に応じて `tsconfig.json` の `strict` 設定を確認。

## ライセンス

MIT

## 関連プロジェクト

- [OpenCode_Rs (バックエンド)](https://github.com/HHKK0127/Opencode_Rs)
  - Rust 実装の API サーバー
  - エンドポイント: `/api/v1/auth/*`, `/api/v1/files/*`

## 貢献

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'feat: add amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成
