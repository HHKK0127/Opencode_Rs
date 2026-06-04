# ストレージ戦略（Wave 2）

**バージョン**: 1.0.0  
**作成日**: 2026-05-28  
**スコープ**: Wave 2 実装フェーズ

---

## 戦略概要

### Wave 2 選択: ローカルファイルシステム

```
┌─────────────────────────────────────────┐
│   API Server (Rust/Actix-web)           │
├─────────────────────────────────────────┤
│   ┌──────────────────────────────────┐  │
│   │  File Handler Layer              │  │
│   ├──────────────────────────────────┤  │
│   │  Storage Abstraction             │  │
│   └──────────────────────────────────┘  │
└────────────┬────────────────────────────┘
             │
             ├─→ Local Filesystem (Wave 2)
             │   ./uploads/{YYYY/MM/DD}/
             │
             ├─→ S3-Compatible (Wave 3)
             │   MinIO / AWS S3
             │
             └─→ Distributed (Wave 4)
                 NFS / Ceph
```

---

## Wave 2: ローカルファイルシステム

### 選択理由

| 観点 | スコア | 理由 |
|------|--------|------|
| **シンプル性** | ⭐⭐⭐ | 追加インフラ不要・即座に実装 |
| **パフォーマンス** | ⭐⭐⭐ | ローカル I/O は最速 |
| **コスト** | ⭐⭐⭐ | 追加コストなし |
| **スケーラビリティ** | ⭐ | 単一サーバー限定 |
| **信頼性** | ⭐⭐ | バックアップ戦略が重要 |

### ディレクトリ構造

```
./uploads/
├── 2026/
│   ├── 05/
│   │   ├── 28/
│   │   │   ├── 550e8400-e29b-41d4-a716-446655440000-report.pdf
│   │   │   ├── 550e8401-e29b-41d4-a716-446655440001-image.jpg
│   │   │   └── 550e8402-e29b-41d4-a716-446655440002-data.csv
│   │   ├── 29/
│   │   │   └── 550e8403-...
│   │   └── 30/
│   └── 06/
│       └── 01/
├── temp/
│   └── {session-id}-chunk-*.tmp
└── archive/
    └── deleted-files-archive.tar.gz
```

### ファイル命名規則

```
{uuid}-{sanitized_original_filename}

例:
  550e8400-e29b-41d4-a716-446655440000-report-2026-05.pdf
  550e8401-e29b-41d4-a716-446655440001-profile-photo.jpg
  550e8402-e29b-41d4-a716-446655440002-data.csv
```

### ファイル名サニタイズ

許可文字: `[a-zA-Z0-9._-]`

```rust
// サニタイズルール
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => c,
                _ => '-'
            }
        })
        .collect()
}

// 例
"My Report (2026).pdf" → "My-Report-2026-.pdf"
"データ.xlsx" → "--------.xlsx"
```

---

## ディスク容量管理

### 容量制限

| 項目 | 容量 | 説明 |
|------|------|------|
| **1ユーザーあたりの上限** | 5GB | 個人ユーザー用 |
| **1ファイルあたりの上限** | 100MB | Wave 2 制限 |
| **全体ストレージ上限** | 1TB | サーバー容量の 80% |
| **Tier A ユーザー上限** | 50GB | エンタープライズ用（Wave 3） |

### ディスク満杯対応

```
容量使用率 80%: ⚠️  アラート
容量使用率 95%: 🚨 緊急アラート
容量使用率 99%: 🛑 アップロード停止
```

**対応手順**:
1. 古いファイル削除（7日以上の削除済みファイル）
2. ユーザーに通知
3. ディスク拡張実施

---

## バックアップ戦略

### バックアップ時期

| 頻度 | 方式 | 保持期間 |
|------|------|---------|
| **日次** | `rsync` で増分バックアップ | 7日 |
| **週次** | スナップショット作成 | 4週 |
| **月次** | アーカイブ化して保存 | 12ヶ月 |

### バックアップコマンド

```bash
# 日次バックアップ（23:00実行）
rsync -avz --delete ./uploads/ backup-server:/backups/opencode-uploads-$(date +%Y%m%d)/

# 週次スナップショット（日曜 23:30）
lvcreate -L10G -s -n "uploads-snapshot-$(date +%Y%m%d)" /dev/vg0/uploads

# 月次アーカイブ（1日 00:00）
tar -czf /archive/opencode-uploads-$(date +%Y%m).tar.gz ./uploads/
aws s3 cp /archive/opencode-uploads-$(date +%Y%m).tar.gz s3://backup-bucket/
```

### リストア手順

```bash
# 日次バックアップから復元（例：5月27日のファイル）
rsync -avz backup-server:/backups/opencode-uploads-20260527/ ./uploads/

# スナップショットから復元
lvconvert -m1 /dev/vg0/uploads-snapshot-20260527
mount /dev/vg0/uploads-snapshot-20260527 /mnt/recover
```

---

## セキュリティ考慮事項

### ファイルアクセス制御

```rust
// ファイルの owner チェック
fn verify_file_access(user_id: &str, file: &File) -> bool {
    file.user_id == user_id || file.is_public
}
```

### ウイルススキャン

Wave 3 で実装予定:
- ClamAV 統合
- オンデマンドスキャン

### ファイル整合性検証

```
SHA-256 チェックサム: すべてのファイルで計算・保存
ダウンロード時: チェックサム検証
```

---

## 監視・メトリクス

### 監視対象

```
- ディスク使用率（%)
- ディスク I/O (IOPS)
- アップロード速度 (MB/s)
- ダウンロード速度 (MB/s)
- ファイル数
- 削除待ちファイル数
```

### アラート閾値

```
ディスク使用率 > 80%: WARNING
ディスク使用率 > 95%: CRITICAL
I/O 待機時間 > 100ms: WARNING
```

---

## Wave 3 への移行計画

### S3 互換ストレージへの移行

**優位性**:
- ✅ スケーラビリティ（無制限容量）
- ✅ 高信頼性（複数レプリケーション）
- ✅ グローバル CDN 対応
- ❌ コスト増加
- ❌ 複雑性増加

**移行手順**:

```
1. ストレージ抽象化レイヤー実装
   FileStorage trait:
     - upload()
     - download()
     - delete()

2. S3 実装追加
   S3Storage impl FileStorage

3. 切り替え設定
   STORAGE_BACKEND = "s3" | "local"

4. 段階的マイグレーション
   - 新規ファイル → S3
   - 既存ファイル → 段階的に移行
```

### 推定コスト

```
AWS S3:
  - ストレージ: $0.023/GB/月
  - アップロード: $0/リクエスト
  - ダウンロード: $0.09/GB

100GB使用時:
  月額: $0.023 × 100 + ダウンロード = $2.30 + $9/GB
       = 約 $12-20/月
```

---

## トラブルシューティング

### ディスク容量不足

```bash
# 使用率確認
df -h ./uploads

# 古いファイル削除
find ./uploads -type f -mtime +30 -delete

# ジャーナルクリーン
rm -rf ./uploads/temp/*
```

### ファイルが見つからない

```bash
# ファイル検索
find ./uploads -name "*{uuid}*"

# DBと実ファイルの一貫性確認
sqlite3 poc_test.db "SELECT id, path FROM files WHERE id = '{uuid}'"
```

### パーミッションエラー

```bash
# パーミッション修正
chmod -R 755 ./uploads
chown -R www-data:www-data ./uploads
```

---

## ベストプラクティス

1. **定期的なバックアップ**: 日次必須
2. **容量監視**: 85% で警告
3. **古いファイル削除**: 90日以上のソフト削除ファイル
4. **ディスク拡張計画**: 容量の 70% 使用時に検討
5. **アクセスログ**: 全ダウンロードを記録

---

## 参考資料

- [SQLite best practices](https://www.sqlite.org/bestpractice.html)
- [Linux ext4 storage optimization](https://wiki.archlinux.org/title/Improving_performance)
- [Backup and recovery strategies](https://en.wikipedia.org/wiki/Backup)

---

**作成者**: Wave 2 チーム  
**最終更新**: 2026-05-28
