# Protocol Mapping: OpenGUI ↔ opencode_poc

**自動生成**: 2026-07-03 20:01  
**ステータス**: Phase 1A-3 完了

---

## 概要

OpenGUI Frontend が期待する wire protocol と opencode_poc API を中間層で変換するためのマッピング定義。

**変換方針**:
- OpenGUI Frontend から来たリクエスト → opencode_poc REST API へ変換
- opencode_poc レスポンス → OpenGUI 互換形式に変換
- エラーハンドリング・ステータスコード統一

---

## 1. 認証フロー

### OpenGUI Frontend Request (標準形式)
```json
{
  "method": "POST",
  "path": "/backend/auth/login",
  "headers": {
    "Content-Type": "application/json"
  },
  "body": {
    "username": "user@example.com",
    "password": "password123",
    "provider": "opencode_poc"
  }
}
```

### opencode_poc API 変換
```
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "user@example.com",
  "password": "password123"
}
```

### 逆変換 (Response)
**opencode_poc Response**:
```json
{
  "status": "success",
  "data": {
    "token": "eyJ0eXAi...",
    "expires_in": 86400
  }
}
```

**OpenGUI 互換 Response**:
```json
{
  "status": "success",
  "data": {
    "accessToken": "eyJ0eXAi...",
    "expiresAt": "2026-07-04T20:01:00Z",
    "tokenType": "Bearer"
  }
}
```

---

### OpenGUI → opencode_poc Protocol Mapping Table

| OpenGUI 概念 | OpenGUI Endpoint | opencode_poc Endpoint | 変換タイプ |
|------------|------------------|----------------------|----------|
| **ユーザーログイン** | `POST /backend/auth/login` | `POST /api/v1/auth/login` | Pass-through |
| **ユーザー登録** | `POST /backend/auth/register` | `POST /api/v1/auth/register` | Pass-through |
| **トークン更新** | `POST /backend/auth/refresh` | `POST /api/v1/auth/refresh` | Pass-through |
| **パスワードリセット** | `POST /backend/auth/reset-password` | `POST /api/v1/auth/reset-password` | Pass-through |

---

## 2. ファイル操作フロー

### ファイルアップロード

**OpenGUI Frontend Request**:
```
POST /backend/files/upload
Content-Type: multipart/form-data
Authorization: Bearer <access_token>

file: <binary>
metadata: { name, size, type }
```

**opencode_poc API 変換**:
```
POST /api/v1/files/upload
Content-Type: multipart/form-data
Authorization: Bearer <jwt_token>

file: <binary>
```

**レスポンス変換**:
```json
opencode_poc {
  "status": "success",
  "data": {
    "id": "file-123",
    "filename": "doc.pdf",
    "size": 102400,
    "path": "/uploads/doc.pdf",
    "uploaded_at": "2026-07-03T20:01:00Z"
  }
}

// ↓ 変換

OpenGUI {
  "status": "success",
  "data": {
    "fileId": "file-123",
    "name": "doc.pdf",
    "size": 102400,
    "type": "application/pdf",
    "createdAt": "2026-07-03T20:01:00Z",
    "url": "/api/v1/files/file-123"
  }
}
```

### ファイル一覧取得

**OpenGUI Frontend Request**:
```
GET /backend/files?limit=20&offset=0
Authorization: Bearer <access_token>
```

**opencode_poc API 変換**:
```
GET /api/v1/files?per_page=20&page=1
Authorization: Bearer <jwt_token>
```

---

## 3. セッション/実行管理フロー

### ⚠️ OpenGUI Session API
OpenGUI Frontend が期待する Session 管理エンドポイント（opencode_poc ではまだ未実装）

**OpenGUI Frontend Request** (期待):
```json
{
  "method": "POST",
  "path": "/backend/sessions",
  "body": {
    "projectPath": "/path/to/project",
    "harness": "opencode_poc",
    "model": "claude-3-5-sonnet"
  }
}
```

**対応**:
- Phase 1B では Bridge 層で**シミュレート**
- 実際の Session はサーバー側で管理可能にする（Phase 2B）

---

## 4. プロンプト/実行フロー

### ⚠️ OpenGUI Prompt Queue
**OpenGUI Frontend Request** (期待):
```json
{
  "method": "POST",
  "path": "/backend/prompts",
  "body": {
    "sessionId": "session-123",
    "content": "Fix the TypeScript errors",
    "model": "claude-3-5-sonnet"
  }
}
```

**対応**:
- Phase 1B では bridge layer で**キューイング**
- Phase 2B で完全実装

---

## 5. ストリーミング/イベント

### ⚠️ OpenGUI EventBus (SSE)
```
GET /backend/events?sessionId=session-123
Accept: text/event-stream

Response:
data: {"type": "execution.started", "timestamp": "2026-07-03T20:01:00Z"}
data: {"type": "execution.output", "content": "Building..."}
data: {"type": "execution.completed", "status": "success"}
```

**対応**:
- Phase 1B では未実装（Bridge なし）
- Phase 2B で SSE/WebSocket 実装

---

## 6. 認証ヘッダーマッピング

| OpenGUI | opencode_poc |
|---------|-------------|
| `Authorization: Bearer <access_token>` | `Authorization: Bearer <jwt_token>` |

**注**: 両者とも JWT を使用するため、pass-through で対応

---

## 7. ステータスコード統一

| HTTP Status | 意味 | opencode_poc | OpenGUI |
|------------|------|-------------|---------|
| 200 | OK | ✅ | ✅ |
| 400 | Bad Request | ✅ | ✅ |
| 401 | Unauthorized | ✅ | ✅ |
| 403 | Forbidden | - | 実装予定 |
| 404 | Not Found | ✅ | ✅ |
| 500 | Server Error | ✅ | ✅ |

---

## 8. エラーハンドリング

### opencode_poc Error Format
```json
{
  "status": "error",
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid username or password"
  }
}
```

### OpenGUI Expected Format
```json
{
  "status": "error",
  "error": {
    "type": "INVALID_CREDENTIALS",
    "message": "Invalid username or password",
    "details": {}
  }
}
```

**変換**: Bridge 層で field rename (`code` → `type`)

---

## 9. Minimal Bridge で実装対象 (Phase 1B)

### ✅ 直接 Pass-through (変換不要)
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/register`
- `POST /api/v1/auth/refresh`
- `GET /health`
- `GET /health/db`

### 🟡 軽い変換が必要 (レスポンス形式)
- `POST /api/v1/files/upload`
- `GET /api/v1/files/{id}`
- `GET /api/v1/files` (リスト)
- `DELETE /api/v1/files/{id}`

### ❌ Phase 1B では未実装 (Phase 2B)
- `POST /backend/sessions` (新規 Session 作成)
- `POST /backend/prompts` (実行キュー)
- `GET /backend/events` (SSE streaming)

---

## 10. レスポンス形式統一スクリプト

### TypeScript Bridge での変換例

```typescript
// opencode_poc → OpenGUI 変換
function transformResponse(body: any, endpoint: string): any {
  if (endpoint.includes('/health')) {
    return body; // そのまま
  }
  
  if (body.status === 'success') {
    // ✅ 成功 → そのまま
    return body;
  }
  
  if (body.status === 'error') {
    // ❌ エラー → フィールド rename
    return {
      status: 'error',
      error: {
        type: body.error?.code,
        message: body.error?.message,
        details: {}
      }
    };
  }
  
  return body;
}
```

---

## チェックリスト (Phase 1B)

- [ ] CORS 設定で localhost:5173 許可
- [ ] レスポンス形式を統一（status/data/error）
- [ ] Bridge から opencode_poc への HTTP クライアント実装
- [ ] JWT token pass-through
- [ ] OpenGUI Frontend で接続テスト
- [ ] ログイン → ファイルアップロードの E2E テスト

