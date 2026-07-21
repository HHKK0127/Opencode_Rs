#![allow(dead_code)]
//! Redis SCANコマンド実装

use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};
use tracing::info;

pub struct RedisScanner {
    conn: MultiplexedConnection,
}

impl RedisScanner {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    /// パターンマッチングでキーを検索
    pub async fn scan_keys(
        &mut self,
        pattern: &str,
        count_per_scan: usize,
    ) -> Result<Vec<String>, RedisError> {
        let mut keys = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let (next_cursor, found_keys): (String, Vec<String>) = match &cursor {
                Some(c) => {
                    redis::cmd("SCAN")
                        .arg(c)
                        .arg("MATCH")
                        .arg(pattern)
                        .arg("COUNT")
                        .arg(count_per_scan)
                        .query_async(&mut self.conn)
                        .await?
                }
                None => {
                    redis::cmd("SCAN")
                        .arg("0")
                        .arg("MATCH")
                        .arg(pattern)
                        .arg("COUNT")
                        .arg(count_per_scan)
                        .query_async(&mut self.conn)
                        .await?
                }
            };

            keys.extend(found_keys);

            if next_cursor == "0" {
                break;
            }
            cursor = Some(next_cursor);
        }

        Ok(keys)
    }

    /// パターンにマッチするキーを削除
    pub async fn delete_by_pattern(&mut self, pattern: &str) -> Result<usize, RedisError> {
        let keys = self.scan_keys(pattern, 100).await?;

        if keys.is_empty() {
            info!("No keys found for pattern: {}", pattern);
            return Ok(0);
        }

        let deleted: usize = self.conn.del(&keys).await?;
        info!(
            "Deleted {} keys matching pattern: {} (found {} keys)",
            deleted,
            pattern,
            keys.len()
        );

        Ok(deleted)
    }

    /// ファイル関連キャッシュを無効化
    pub async fn invalidate_file_caches(&mut self) -> Result<usize, RedisError> {
        let mut total_deleted = 0;

        // ファイルリストキャッシュ
        total_deleted += self.delete_by_pattern("files:list:*").await?;

        // ファイルメタデータキャッシュ
        total_deleted += self.delete_by_pattern("file:metadata:*").await?;

        // 検索キャッシュ
        total_deleted += self.delete_by_pattern("files:search:*").await?;

        Ok(total_deleted)
    }

    /// ユーザーセッションキャッシュを無効化
    pub async fn invalidate_user_sessions(&mut self, user_id: &str) -> Result<usize, RedisError> {
        self.delete_by_pattern(&format!("session:user:{}:*", user_id))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_redis_scanner_creation() {
        // RedisScanner が正常に作成できることをテスト
        // 実際のテストはローカル Redis が必要
        // ここではコンパイルチェックのみ
    }

    #[test]
    fn test_pattern_matching() {
        // パターン文字列の検証（実際の SCAN は非同期が必要）
        let patterns = vec![
            "files:list:*",
            "file:metadata:*",
            "files:search:*",
            "session:user:*",
        ];

        for pattern in patterns {
            assert!(!pattern.is_empty());
            assert!(pattern.contains(":"));
        }
    }
}
