# OpenCode Core API 仕様書

**バージョン**: 1.1.0  
**作成日**: 2026-05-27  
**最終更新**: 2026-06-05 (Wave 4 Day 13 キャッシング機能追加)  
**Wave 1-3 + Wave 4 Day 13 完成版**

---

## 概要

OpenCode Core API は、Rust で実装された RESTful API サーバーです。JWT 認証、ファイルアップロード、ユーザー管理を提供します。

### ベース URL
```
http://localhost:8080/api/v1
```

### 認証
全ての保護されたエンドポイントは `Authorization: Bearer <JWT_TOKEN>` ヘッダーが必要です。

---

## 認証エンドポイント

### POST /auth/register
新規ユーザー登録

**リクエスト:**
```json
{
  "username": "string (required, min: 3, max: 32)",
  "password": "string (required, min: 8)"
}
```

**レスポンス (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

**エラーレスポンス:**
- `400 Bad Request` — バリデーションエラー
- `409 Conflict` — ユーザーが既に存在

---

### POST /auth/login
ユーザーログイン

**リクエスト:**
```json
{
  "username": "string (required)",
  "password": "string (required)"
}
```

**レスポンス (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

**エラーレスポンス:**
- `401 Unauthorized` — 認証情報が無効

---

### POST /auth/refresh
トークンをリフレッシュ（有効期限を延長）

**ヘッダー:**
```
Authorization: Bearer <CURRENT_TOKEN>
```

**リクエスト:** 本体不要

**レスポンス (200):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

**エラーレスポンス:**
- `401 Unauthorized` — トークンが無効または期限切れ

---

## ユーザーエンドポイント

### GET /users
全ユーザーのリストを取得（管理者のみ）

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**レスポンス (200):**
```json
{
  "users": [
    {
      "id": "uuid",
      "username": "string",
      "created_at": "2026-05-27T10:30:00Z"
    }
  ]
}
```

---

### GET /users/{id}
特定ユーザー情報を取得

**パラメータ:**
- `id` (path): ユーザーID (UUID)

**レスポンス (200):**
```json
{
  "id": "uuid",
  "username": "string",
  "created_at": "2026-05-27T10:30:00Z"
}
```

**エラーレスポンス:**
- `404 Not Found` — ユーザーが見つからない

---

## ファイルエンドポイント

### POST /files/upload
ファイルアップロード

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
Content-Type: multipart/form-data
```

**リクエスト:**
```
Form Data:
  - file: <binary file> (max 10MB)
```

**レスポンス (200):**
```json
{
  "id": "uuid",
  "filename": "string",
  "size": 1024,
  "uploaded_at": "2026-05-27T10:30:00Z"
}
```

**エラーレスポンス:**
- `400 Bad Request` — ファイルが見つからない
- `413 Payload Too Large` — ファイルサイズが 10MB を超過

**キャッシング** (Wave 4 Day 13):
- Upload 実行時: リスト & 検索キャッシュを無効化

---

### GET /files/{id}
ファイルメタデータ取得 **(キャッシュ付き - 1h TTL)**

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**パラメータ:**
- `id` (path): ファイル ID (UUID)

**レスポンス (200):**
```json
{
  "id": "uuid",
  "filename": "string",
  "size": 1024,
  "mime_type": "application/pdf",
  "created_at": "2026-05-27T10:30:00Z",
  "is_public": false
}
```

**キャッシング** (Wave 4 Day 13):
- **キャッシュキー**: `file:metadata:{id}`
- **TTL**: 1時間
- **パターン**: Cache-Aside (Redis miss → DB query → cache set)
- **メトリクス**: 
  - `redis_cache_hits_total` — キャッシュヒット数
  - `redis_cache_misses_total` — キャッシュミス数
  - `redis_operations_total{operation="api_metadata_cache_hit/miss"}`

---

### GET /files
ファイル一覧取得（ページネーション付き）**(キャッシュ付き - 30m TTL)**

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**クエリパラメータ:**
- `page` (optional): ページ番号 (デフォルト: 1)
- `per_page` (optional): 1ページあたりの件数 (デフォルト: 20, 最大: 100)

**レスポンス (200):**
```json
{
  "files": [
    {
      "id": "uuid",
      "filename": "document.pdf",
      "size": 2048,
      "mime_type": "application/pdf",
      "created_at": "2026-05-27T10:30:00Z",
      "url": "/api/v1/files/{id}/download"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

**キャッシング** (Wave 4 Day 13):
- **キャッシュキー**: `files:list:{page}:{per_page}`
- **TTL**: 30分
- **パターン**: Cache-Aside
- **無効化**: DELETE /files/{id} 実行時、Upload 実行時
- **メトリクス**: `redis_operations_total{operation="api_list_cache_hit/miss"}`

---

### GET /files/search
ファイル検索（フィルタ付き）**(キャッシュ付き - 30m TTL)**

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**クエリパラメータ:**
- `q` (optional): キーワード検索
- `file_type` (optional): ファイルタイプフィルタ
- `created_after` (optional): 作成日付フィルタ
- `page` (optional): ページ番号 (デフォルト: 1)
- `per_page` (optional): 1ページあたりの件数 (デフォルト: 20, 最大: 100)

**レスポンス (200):**
```json
{
  "files": [...],
  "total": 25,
  "page": 1,
  "per_page": 20,
  "total_pages": 2,
  "cached": true
}
```

**キャッシング** (Wave 4 Day 13):
- **キャッシュキー**: `files:search:{query_hash}:{page}:{per_page}`
- **キー生成**: DefaultHasher で全クエリパラメータをハッシュ化
- **TTL**: 30分
- **パターン**: Cache-Aside
- **無効化**: DELETE /files/{id} 実行時、Upload 実行時
- **メトリクス**: `redis_operations_total{operation="search_cache_hit/miss"}`

---

### DELETE /files/{id}
ファイル削除

**ヘッダー:**
```
Authorization: Bearer <JWT_TOKEN>
```

**レスポンス (200):**
```json
{
  "status": "success",
  "message": "File deleted successfully"
}
```

**キャッシング** (Wave 4 Day 13):
- **無効化対象**:
  - `file:metadata:{id}` — メタデータキャッシュ
  - `files:list:*` — 全ページのリストキャッシュ
  - `files:search:*` — 全検索結果キャッシュ
- **メトリクス**: `redis_operations_total{operation="api_invalidate_on_delete"}`

---

## ヘルスチェックエンドポイント

### GET /health
API サーバーのヘルスチェック（認証不要）

**レスポンス (200):**
```json
{
  "status": "healthy",
  "timestamp": "2026-05-27T10:30:00Z"
}
```

---

### GET /health/db
データベース接続チェック（認証不要）

**レスポンス (200):**
```json
{
  "status": "healthy",
  "database": "sqlite",
  "latency_ms": 2
}
```

---

## メトリクスエンドポイント (Wave 2 Day 4+)

### GET /metrics
Prometheusメトリクス取得（認証不要、監視用）

**エンドポイント**: `GET /api/v1/metrics`

**説明**: API パフォーマンスおよびリソース利用状況をPrometheus形式で返却します。Prometheus サーバーのスクレイピング対象エンドポイント。

**レスポンス (200):**
```
Content-Type: text/plain; charset=utf-8

# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{endpoint="auth",method="POST",status="200"} 1024
http_requests_total{endpoint="auth",method="POST",status="401"} 5
http_requests_total{endpoint="files",method="POST",status="200"} 512
...

# HELP http_request_duration_seconds Request latency in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.001",method="POST",endpoint="auth"} 512
http_request_duration_seconds_bucket{le="0.01",method="POST",endpoint="auth"} 920
http_request_duration_seconds_bucket{le="0.05",method="POST",endpoint="auth"} 1018
http_request_duration_seconds_bucket{le="+Inf",method="POST",endpoint="auth"} 1024
...

# HELP active_connections Current number of active connections
# TYPE active_connections gauge
active_connections 42

# HELP file_upload_bytes_total Total bytes uploaded
# TYPE file_upload_bytes_total counter
file_upload_bytes_total 5368709120
```

### メトリクス詳細

#### http_requests_total (Counter)
- **説明**: HTTP リクエスト総数
- **ラベル**: `method` (GET/POST/PUT/DELETE), `endpoint` (auth/files/users/...), `status` (200/401/500/...)
- **用途**: リクエスト数監視、ステータスコード分析
- **Prometheus クエリ例**:
  ```promql
  # 1分間のリクエストレート
  rate(http_requests_total[1m])
  
  # エラー率
  rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])
  ```

#### http_request_duration_seconds (Histogram)
- **説明**: リクエスト処理時間（秒）
- **ラベル**: `method`, `endpoint`
- **バケット**: [0.001, 0.01, 0.05, 0.1, 0.5, 1.0, +Inf]
- **用途**: レイテンシ監視、パーセンタイル計算
- **Prometheus クエリ例**:
  ```promql
  # p95 レイテンシ
  histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
  
  # 平均レイテンシ
  rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])
  ```

#### http_request_size_bytes (Histogram)
- **説明**: リクエストペイロードサイズ（バイト）
- **ラベル**: `method`, `endpoint`
- **用途**: リクエストサイズ分布監視

#### http_response_size_bytes (Histogram)
- **説明**: レスポンスペイロードサイズ（バイト）
- **ラベル**: `method`, `endpoint`
- **用途**: レスポンスサイズ分布監視

#### active_connections (Gauge)
- **説明**: 現在のアクティブな接続数
- **用途**: 接続プール監視、コネクション枯渇検出
- **アラート基準**: `active_connections > 900` で警告

#### file_upload_bytes_total (Counter)
- **説明**: ファイルアップロード合計量（バイト）
- **用途**: トラフィック・容量監視、アップロード量追跡

### Prometheus 統合方法

Prometheus にこのエンドポイントを追加する方法:

**prometheus.yml**:
```yaml
scrape_configs:
  - job_name: 'opencode-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/api/v1/metrics'
    scrape_interval: 15s
```

### 監視・ダッシュボード構築

詳細な監視設定、Grafana ダッシュボード、アラートルール設定については以下を参照:

📋 **[../Operations/MONITORING.md](../Operations/MONITORING.md)** — 完全な監視システム設定ガイド

### Redis キャッシングメトリクス (Wave 4 Day 13+)

#### redis_cache_hits_total (Counter)
- **説明**: Redis キャッシュヒット総数
- **用途**: キャッシュ効率性監視
- **Prometheus クエリ例**:
  ```promql
  rate(redis_cache_hits_total[5m])  # ヒット率の時間変化
  ```

#### redis_cache_misses_total (Counter)
- **説明**: Redis キャッシュミス総数
- **用途**: キャッシュヒット率計算
- **Prometheus クエリ例**:
  ```promql
  # キャッシュヒット率（%）
  (redis_cache_hits_total / (redis_cache_hits_total + redis_cache_misses_total)) * 100
  ```

#### redis_operations_total (Counter)
- **説明**: Redis 操作総数
- **ラベル**: `operation` (api_metadata_cache_hit/miss, api_list_cache_hit/miss, search_cache_hit/miss, など)
- **用途**: キャッシング動作詳細監視
- **Prometheus クエリ例**:
  ```promql
  # メタデータキャッシュのヒット数
  redis_operations_total{operation="api_metadata_cache_hit"}
  ```

#### redis_command_duration_seconds (Histogram)
- **説明**: Redis コマンド実行時間（秒）
- **ラベル**: `command` (GET, SET, DEL, など)
- **用途**: Redis レイテンシ監視
- **Prometheus クエリ例**:
  ```promql
  # SET コマンドのp95レイテンシ
  histogram_quantile(0.95, rate(redis_command_duration_seconds_bucket{command="SET"}[5m]))
  ```

### レスポンス時間

- **通常**: 2-3ms
- **大規模メトリクス収集時**: < 10ms
- **p95**: < 15ms

### エラーレスポンス

エラーが発生した場合（稀）:

```
HTTP/1.1 500 Internal Server Error
Content-Type: text/plain

Failed to generate metrics
```

※ このエンドポイントは監視目的のため、エラーが発生しても自動リトライが行われます

---

## エラーハンドリング

全てのエラーレスポンスは以下の形式です：

```json
{
  "error": "error_code",
  "message": "Detailed error message",
  "timestamp": "2026-05-27T10:30:00Z"
}
```

### HTTP ステータスコード

| コード | 意味 |
|-------|------|
| 200 | OK - リクエスト成功 |
| 400 | Bad Request - バリデーションエラー |
| 401 | Unauthorized - 認証失敗 |
| 404 | Not Found - リソースが見つからない |
| 409 | Conflict - リソースが既に存在 |
| 413 | Payload Too Large - ファイルサイズ超過 |
| 500 | Internal Server Error - サーバーエラー |
| 503 | Service Unavailable - 依存関係エラー |

---

## セキュリティ

### JWT トークン
- **アルゴリズム**: HS256
- **有効期限**: 24時間（86400 秒）
- **署名方式**: 環境変数 `JWT_SECRET` で設定

### パスワードハッシング
- **アルゴリズム**: Argon2id
- **処理時間**: 100-200ms

### ファイルアップロード
- **最大サイズ**: 10MB
- **許可拡張子**: 制限なし（ただし、ファイル名はサニタイズ）
- **保存場所**: `./uploads/` ディレクトリ

### CORS ポリシー
許可されたオリジン:
- `http://localhost:3000`
- `http://localhost:5173`
- `tauri://localhost`

---

## レート制限

現在、レート制限は実装予定です（Wave 2）。

---

## ドキュメント履歴

| バージョン | 日付 | 変更内容 |
|-----------|------|---------|
| 1.1.0 | 2026-06-05 | Wave 4 Day 13 キャッシング機能追加（GET /files/{id}, GET /files, GET /files/search, DELETE /files/{id}キャッシング実装、Redis メトリクス追加） |
| 1.0.0 | 2026-05-27 | Wave 1 仕様書完成 |

---

**Location**: docs/API/API_SPECIFICATION.md  
**Last Updated**: 2026-06-05
