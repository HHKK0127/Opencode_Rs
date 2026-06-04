# セットアップ手順

OpenCode Core API の開発環境・本番環境でのセットアップ方法です。

---

## 必要要件

### 開発環境
- **Rust**: 1.75 以上
- **Cargo**: Rust に同梱
- **SQLite**: 3.0 以上
- **Docker**（オプション、本番環境推奨）

### インストール確認
```bash
# Rust & Cargo
rustc --version
cargo --version

# SQLite
sqlite3 --version
```

---

## クイックスタート（開発環境）

### 1. リポジトリクローン
```bash
cd C:\Drive\Cargo
git clone https://github.com/opencode/core.git
cd RsCode
```

### 2. 環境設定
```bash
# .env ファイル作成（オプション）
cp .env.example .env

# 設定値確認
cat config/development.toml
```

### 3. ビルド
```bash
# デバッグビルド（開発用・高速）
cargo build

# リリースビルド（本番用・最適化）
cargo build --release
```

### 4. データベース初期化
```bash
# サーバー起動時に自動初期化
# 初期ユーザー: testuser / testpassword
cargo run
```

サーバーが起動します：
```
Server listening on 127.0.0.1:8080
```

### 5. 動作確認
```bash
# ヘルスチェック
curl http://localhost:8080/health

# ログイン
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpassword"}'
```

---

## テスト実行

### ユニット・統合テスト
```bash
# 全テスト実行
cargo test

# 出力表示
cargo test -- --nocapture

# 特定テストのみ
cargo test auth_flow
```

### E2E テスト
```bash
# 前提条件: サーバー起動済み
cargo test --test e2e
```

### ロードテスト（k6）
```bash
# 前提条件: サーバー起動済み、k6 インストール済み

# k6 インストール (macOS)
brew install k6

# k6 インストール (Linux/Windows)
# https://k6.io/docs/getting-started/installation/

# ロードテスト実行
# PowerShell (Windows)
.\tests\load\run-load-tests.ps1

# Bash (Linux/macOS)
chmod +x tests/load/run-load-tests.sh
./tests/load/run-load-tests.sh
```

---

## Docker での実行

### ビルド
```bash
# Docker イメージビルド
docker build -t opencode-api:latest .

# ビルド確認
docker images | grep opencode
```

### 実行
```bash
# Docker Compose を使用（推奨）
docker-compose up -d

# ログ確認
docker-compose logs -f opencode-api

# 停止
docker-compose down
```

### ヘルスチェック
```bash
# 待機（起動完了まで）
sleep 3

# ヘルスチェック
curl http://localhost:8080/health
```

---

## 設定ファイル

### development.toml
開発環境用設定（ローカル開発に適切）

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 4

[database]
path = "./poc_test.db"
max_connections = 5

[logging]
level = "debug"

[upload]
max_size_mb = 10
```

### production.toml
本番環境用設定（Docker コンテナ内で使用）

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8

[database]
path = "/data/prod.db"
max_connections = 20

[logging]
level = "info"

[upload]
max_size_mb = 50
```

### 環境変数でのオーバーライド
```bash
# 環境変数設定（OPENCODE__ プリフィックス使用）
export OPENCODE__SERVER__PORT=9000
export OPENCODE__LOGGING__LEVEL=debug

cargo run
```

---

## トラブルシューティング

### ポート 8080 が既に使用中
```bash
# ポート変更
OPENCODE__SERVER__PORT=8081 cargo run

# または、既存プロセスを終了
# Windows
netstat -ano | findstr :8080
taskkill /PID <PID> /F

# Linux/macOS
lsof -i :8080
kill -9 <PID>
```

### データベースエラー
```bash
# データベースをリセット
rm poc_test.db
cargo run  # 再初期化
```

### JWT エラー
```bash
# JWT_SECRET が設定されているか確認
echo $JWT_SECRET

# 設定されていない場合
export JWT_SECRET=your-secret-key-here-min-32-chars
cargo run
```

### CORS エラー
許可されたオリジンを確認（middleware_cors.rs）：
- `http://localhost:3000`
- `http://localhost:5173`
- `tauri://localhost`

開発サーバーがこれらのオリジンで実行されていることを確認してください。

---

## パフォーマンス測定

### ビルド時間
```bash
time cargo build --release
```

### バイナリサイズ
```bash
# Linux/macOS
ls -lh target/release/opencode-server

# Windows
dir target\release\opencode-server.exe
```

### サーバー起動時間
```bash
time ./target/release/opencode-server
```

---

## デプロイメント

詳細は `DEPLOYMENT.md` を参照してください。

### 本番環境チェックリスト
- [ ] JWT_SECRET を安全に設定
- [ ] HTTPS を有効化
- [ ] CORS オリジンを本番ドメインに限定
- [ ] ロギングレベルを "info" に設定
- [ ] データベースバックアップを構成
- [ ] 監視・アラート設定完了
- [ ] セキュリティスキャン完了
- [ ] パフォーマンステスト完了

---

## よくある質問

**Q: 初期ユーザーのパスワードを変更したい**
```bash
# ユーザーテーブルを確認
sqlite3 poc_test.db "SELECT * FROM users;"

# 直接変更（開発環境のみ）
sqlite3 poc_test.db "UPDATE users SET password_hash='...' WHERE username='testuser';"
```

**Q: ファイルアップロード先を変更したい**
- `config/development.toml` または `config/production.toml` の `upload.path` を編集

**Q: PostgreSQL に変更したい**
- Wave 2 での実装予定
- 現在は SQLite のみ対応

---

## サポート・問題報告

問題が発生した場合：
1. `RUST_BACKTRACE=1 cargo run` でバックトレース表示
2. ログを確認（`RUST_LOG=debug`）
3. GitHub Issues で報告

---

## 次のステップ

- API 仕様書: [API_SPECIFICATION.md](API_SPECIFICATION.md)
- デプロイメント: [DEPLOYMENT.md](DEPLOYMENT.md)
- トラブルシューティング: [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
