# ファイル処理 API 仕様書（Wave 2）

**バージョン**: 2.0.0  
**作成日**: 2026-05-28  
**Wave 2 実装版**

---

## 概要

OpenCode Core API のファイル処理モジュール。アップロード、ダウンロード、削除、検索機能を提供します。

### ベース URL
```
http://localhost:8080/api/v1/files
```

---

## エンドポイント一覧

### POST /api/v1/files/upload
マルチパート形式でファイルをアップロード

**ヘッダー:**
```
Content-Type: multipart/form-data
Authorization: Bearer <JWT_TOKEN>
```

**フォームデータ:**
| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| file | binary | ✅ | アップロードするファイル（最大100MB）|
| description | string | ❌ | ファイルの説明 |
| tags | string | ❌ | タグ（カンマ区切り） |

**レスポンス (200):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "filename": "report-2026-05-28.pdf",
  "original_name": "report.pdf",
  "size": 1048576,
  "mime_type": "application/pdf",
  "path": "/uploads/2026/05/28/550e8400-report.pdf",
  "url": "/api/v1/files/550e8400-e29b-41d4-a716-446655440000/download",
  "checksum": "sha256:abcdef123456...",
  "created_at": "2026-05-28T10:00:00Z",
  "expires_at": null
}
```

**エラーレスポンス:**
- `400 Bad Request` — ファイルが見つからない
- `413 Payload Too Large` — ファイルサイズ超過（100MB）
- `415 Unsupported Media Type` — サポート外 MIME タイプ

---

### GET /api/v1/files/{id}
ファイルのメタデータを取得

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**パラメータ:**
- `id` (path): ファイル ID (UUID)

**レスポンス (200):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "filename": "report-2026-05-28.pdf",
  "original_name": "report.pdf",
  "size": 1048576,
  "mime_type": "application/pdf",
  "description": "Monthly report",
  "tags": ["report", "monthly"],
  "created_at": "2026-05-28T10:00:00Z",
  "updated_at": "2026-05-28T10:00:00Z",
  "is_public": false,
  "expires_at": null
}
```

**エラーレスポンス:**
- `404 Not Found` — ファイルが見つからない
- `403 Forbidden` — アクセス権限がない

---

### GET /api/v1/files/{id}/download
ファイルをダウンロード（ストリーミング対応）

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
Range: bytes=0-1023  (オプション)
```

**レスポンス (200):**
```
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="report.pdf"
Accept-Ranges: bytes
Content-Length: 1048576
```

**レスポンス (206 Partial Content):**
Range リクエスト時のレスポンス

```
Content-Range: bytes 0-1023/1048576
Content-Length: 1024
```

**エラーレスポンス:**
- `404 Not Found` — ファイルが見つからない
- `403 Forbidden` — ダウンロード権限がない
- `416 Range Not Satisfiable` — Range 値が無効

---

### DELETE /api/v1/files/{id}
ファイルを削除（ソフト削除）

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**パラメータ:**
- `id` (path): ファイル ID (UUID)

**レスポンス (200):**
```json
{
  "status": "success",
  "message": "File deleted successfully"
}
```

**エラーレスポンス:**
- `404 Not Found` — ファイルが見つからない
- `403 Forbidden` — 削除権限がない

---

### GET /api/v1/files
ファイル一覧を取得（ページネーション対応）

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**クエリパラメータ:**
| パラメータ | 型 | デフォルト | 説明 |
|-----------|-----|-----------|------|
| page | integer | 1 | ページ番号 |
| per_page | integer | 20 | 1ページあたりの件数（最大: 100） |
| sort | string | created_at | ソート対象（created_at \| size \| filename） |
| order | string | desc | ソート順序（asc \| desc） |
| tags | string | - | タグ検索（カンマ区切り） |
| mime_type | string | - | MIME タイプでフィルタ |

**レスポンス (200):**
```json
{
  "files": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "filename": "report-2026-05-28.pdf",
      "size": 1048576,
      "mime_type": "application/pdf",
      "created_at": "2026-05-28T10:00:00Z",
      "url": "/api/v1/files/550e8400-e29b-41d4-a716-446655440000/download"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 150,
    "total_pages": 8
  }
}
```

---

## ストリーミング対応

### 大容量ファイルのダウンロード

HTTP Range リクエストに対応：

```bash
# 1MB ずつダウンロード
curl -H "Range: bytes=0-1048575" http://localhost:8080/api/v1/files/{id}/download
curl -H "Range: bytes=1048576-2097151" http://localhost:8080/api/v1/files/{id}/download
```

### 大容量ファイルのアップロード

Wave 3 で チャンク アップロード実装予定

---

## エラーハンドリング

全てのエラーレスポンスは以下の形式：

```json
{
  "error": "error_code",
  "message": "Detailed error message",
  "timestamp": "2026-05-28T10:00:00Z"
}
```

### HTTP ステータスコード

| コード | 意味 | 例 |
|--------|------|-----|
| 200 | OK | ファイル取得成功 |
| 206 | Partial Content | Range リクエスト成功 |
| 400 | Bad Request | ファイルなし |
| 401 | Unauthorized | 認証失敗 |
| 403 | Forbidden | 権限不足 |
| 404 | Not Found | ファイル未存在 |
| 413 | Payload Too Large | サイズ超過 |
| 415 | Unsupported Media Type | MIME タイプ未対応 |
| 416 | Range Not Satisfiable | Range 値が無効 |
| 500 | Internal Server Error | サーバーエラー |

---

## セキュリティ

### ファイルアップロード
- **最大サイズ**: 100MB
- **許可 MIME タイプ**: 制限なし（スキャン対応は Wave 3）
- **ファイル名**: サニタイズ（英数字、ハイフン、アンダースコア、ドット）

### アクセス制御
- アップロードユーザーのみ削除・ダウンロード可能
- 公開ファイル（is_public=true）は認証不要

### チェックサム検証
全ファイルの SHA-256 ハッシュを計算・保存

```json
"checksum": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
```

---

## レート制限

現在、レート制限は Wave 1 のままです。Wave 3 で強化予定。

---

## ドキュメント履歴

| バージョン | 日付 | 変更内容 |
|-----------|------|---------|
| 1.0.0 | 2026-05-27 | Wave 1 仕様書完成 |
| 2.0.0 | 2026-05-28 | Wave 2 ファイル処理実装 |
