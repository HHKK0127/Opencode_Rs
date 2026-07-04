# OpenCode_poc API Endpoints

**自動生成**: 2026-07-03 20:01  
**ステータス**: Phase 1A-2 完了

---

## 認証エンドポイント

### POST /api/v1/auth/login
**認証なし**

**Request**:
```json
{
  "username": "testuser",
  "password": "testpassword"
}
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "expires_in": 86400
  }
}
```

**Response** (エラー):
```json
{
  "status": "error",
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid username or password"
  }
}
```

---

### POST /api/v1/auth/register
**認証なし**

**Request**:
```json
{
  "username": "newuser",
  "password": "securepassword123"
}
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "newuser"
  }
}
```

---

### POST /api/v1/auth/refresh
**認証なし**

**Request**:
```json
{
  "token": "existing_jwt_token"
}
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "token": "new_jwt_token",
    "expires_in": 86400
  }
}
```

---

### POST /api/v1/auth/reset-password
**認証なし**

**Request**:
```json
{
  "username": "testuser",
  "new_password": "newpassword123"
}
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "message": "Password reset successfully"
  }
}
```

---

## ファイルエンドポイント

### POST /api/v1/files/upload
**認証**: JWT Required (Authorization: Bearer <token>)

**Request**: multipart/form-data
```
Content-Type: multipart/form-data
Authorization: Bearer <jwt_token>

file: <binary_data>
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "id": "file-uuid-123",
    "filename": "document.pdf",
    "size": 102400,
    "path": "/uploads/document.pdf",
    "uploaded_at": "2026-07-03T20:01:00Z"
  }
}
```

---

### GET /api/v1/files/{id}
**認証**: JWT Required

**Request**: `GET /api/v1/files/file-uuid-123`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "id": "file-uuid-123",
    "filename": "document.pdf",
    "size": 102400,
    "path": "/uploads/document.pdf",
    "uploaded_at": "2026-07-03T20:01:00Z"
  }
}
```

---

### GET /api/v1/files/{id}/download
**認証**: JWT Required

**Request**: `GET /api/v1/files/file-uuid-123/download`

**Response**: Binary file content with headers:
```
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="document.pdf"
```

---

### DELETE /api/v1/files/{id}
**認証**: JWT Required

**Request**: `DELETE /api/v1/files/file-uuid-123`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "message": "File deleted successfully"
  }
}
```

---

### GET /api/v1/files (リスト取得)
**認証**: JWT Required

**Request**: `GET /api/v1/files?page=1&per_page=20`

**Query Params**:
- `page` (optional): ページ番号（デフォルト: 1）
- `per_page` (optional): ページあたりアイテム数（デフォルト: 20）

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "items": [
      {
        "id": "file-uuid-1",
        "filename": "doc1.pdf",
        "size": 102400,
        "uploaded_at": "2026-07-03T20:01:00Z"
      }
    ],
    "total": 5,
    "page": 1,
    "per_page": 20
  }
}
```

---

### POST /api/v1/files/upload/init
**認証**: JWT Required  
**用途**: チャンク分割アップロードの初期化

**Request**:
```json
{
  "filename": "largefile.zip",
  "size": 1048576000,
  "chunk_count": 100
}
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "session_id": "upload-session-uuid",
    "chunk_count": 100,
    "chunk_size": 10485760
  }
}
```

---

### POST /api/v1/files/upload/chunk
**認証**: JWT Required

**Request**: multipart/form-data
```
Content-Type: multipart/form-data
Authorization: Bearer <jwt_token>

session_id: upload-session-uuid
chunk_index: 0
chunk_data: <binary_data>
```

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "chunk_index": 0,
    "uploaded": true
  }
}
```

---

### POST /api/v1/files/upload/complete/{session_id}
**認証**: JWT Required

**Request**: `POST /api/v1/files/upload/complete/upload-session-uuid`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "file_id": "file-uuid-123",
    "filename": "largefile.zip",
    "size": 1048576000
  }
}
```

---

### GET /api/v1/files/upload/progress/{session_id}
**認証**: JWT Required

**Request**: `GET /api/v1/files/upload/progress/upload-session-uuid`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "uploaded_chunks": 25,
    "total_chunks": 100,
    "progress_percent": 25
  }
}
```

---

### GET /api/v1/files/search
**認証**: JWT Required

**Request**: `GET /api/v1/files/search?q=document&mime_type=application/pdf&size_min=0&size_max=10485760`

**Query Params**:
- `q` (optional): キーワード検索
- `mime_type` (optional): MIME タイプでフィルタ
- `size_min` (optional): ファイルサイズ最小値（バイト）
- `size_max` (optional): ファイルサイズ最大値（バイト）
- `created_after` (optional): ISO8601 日時
- `sort` (optional): ソート対象 (name, size, created_at)
- `order` (optional): asc | desc
- `page` (optional): ページ番号
- `per_page` (optional): ページあたりアイテム数

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "items": [...],
    "total": 42,
    "page": 1,
    "per_page": 20
  }
}
```

---

### GET /api/v1/files/stats
**認証**: JWT Required

**Request**: `GET /api/v1/files/stats`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "total_files": 150,
    "total_size_bytes": 5368709120,
    "average_file_size": 35791394,
    "file_count_by_type": {
      "pdf": 45,
      "docx": 32,
      "xlsx": 28,
      "other": 45
    }
  }
}
```

---

## ユーザーエンドポイント

### GET /api/v1/users
**認証**: JWT Required

**Request**: `GET /api/v1/users?page=1&per_page=20`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "items": [
      {
        "id": "user-uuid-1",
        "username": "testuser",
        "created_at": "2026-06-25T10:00:00Z"
      }
    ],
    "total": 5,
    "page": 1,
    "per_page": 20
  }
}
```

---

### GET /api/v1/users/{id}
**認証**: JWT Required

**Request**: `GET /api/v1/users/user-uuid-1`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "id": "user-uuid-1",
    "username": "testuser",
    "created_at": "2026-06-25T10:00:00Z"
  }
}
```

---

## プロジェクトエンドポイント

### GET /api/v1/projects
**認証**: JWT Required

**Request**: `GET /api/v1/projects`

**Response** (成功):
```json
{
  "status": "success",
  "data": {
    "projects": [
      {
        "id": "project-1",
        "name": "OpenCode",
        "description": "AI coding assistant",
        "created_at": "2026-06-01T00:00:00Z"
      }
    ]
  }
}
```

---

## ヘルスチェックエンドポイント

### GET /health
**認証**: なし

**Request**: `GET /health`

**Response** (成功):
```json
{
  "status": "ok",
  "database": "connected",
  "timestamp": "2026-07-03T20:01:00Z"
}
```

---

### GET /health/db
**認証**: なし

**Request**: `GET /health/db`

**Response** (成功):
```json
{
  "status": "connected",
  "latency_ms": 5.234
}
```

---

## レスポンス統一形式

**成功レスポンス**:
```json
{
  "status": "success",
  "data": { /* 実際のデータ */ }
}
```

**エラーレスポンス**:
```json
{
  "status": "error",
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message"
  }
}
```

**HTTP ステータスコード**:
- `200 OK` - 成功
- `400 Bad Request` - リクエスト不正
- `401 Unauthorized` - 認証失敗
- `403 Forbidden` - 権限不足
- `404 Not Found` - リソース不存在
- `500 Internal Server Error` - サーバーエラー

