# Hermes 統合 - 技術仕様書

**対象**: Wave 4 Day 16-17 での統合  
**スコープ**: スケジュール自動化 + マルチプラットフォーム通知

---

## 1️⃣ スケジュール自動化システム

### 目標
```
Cron ベースの自動化タスク実行
├─ Cache entry TTL cleanup
├─ Daily metrics report
├─ Weekly file retention cleanup
└─ Monthly storage optimization
```

### 技術スタック

```toml
[dependencies]
tokio-cron = "0.1"        # Cron expression parser
tokio-schedule = "0.1"    # Task scheduling
chrono = "0.4"            # DateTime handling
anyhow = "1.0"            # Error handling
```

### 実装設計

```rust
// src/scheduler/mod.rs
pub struct ScheduledTask {
    id: String,
    name: String,
    cron_expression: String,  // "0 2 * * *" (毎日2am)
    handler: Arc<dyn TaskHandler>,
    last_run: Option<DateTime<Utc>>,
    next_run: Option<DateTime<Utc>>,
}

pub trait TaskHandler: Send + Sync {
    async fn execute(&self, context: &ScheduleContext) -> Result<TaskResult>;
}

pub struct ScheduleContext {
    redis: Arc<RedisPool>,
    db: Arc<DbPool>,
    metrics: Arc<MetricsRegistry>,
    logger: Arc<tracing::Subscriber>,
}

pub struct TaskResult {
    task_id: String,
    status: TaskStatus,
    duration_ms: u64,
    message: String,
    error: Option<String>,
}

#[derive(Debug)]
pub enum TaskStatus {
    Success,
    Failed,
    Skipped,
}
```

### 実装タスク

#### Task 1: Cron Scheduler 基盤
```rust
pub struct CronScheduler {
    tasks: Arc<RwLock<Vec<ScheduledTask>>>,
    runtime: tokio::runtime::Handle,
    shutdown: tokio::sync::broadcast::Sender<()>,
}

impl CronScheduler {
    pub async fn new() -> Result<Self>;
    pub async fn register_task(&self, task: ScheduledTask) -> Result<()>;
    pub async fn start(&self) -> Result<()>;
    pub async fn shutdown(&self) -> Result<()>;
    pub async fn get_task_status(&self, task_id: &str) -> Result<TaskStatus>;
    pub async fn trigger_task_now(&self, task_id: &str) -> Result<TaskResult>;
}
```

#### Task 2: Built-in Tasks 実装
```rust
// Cache cleanup
pub struct CacheCleanupTask;

impl TaskHandler for CacheCleanupTask {
    async fn execute(&self, context: &ScheduleContext) -> Result<TaskResult> {
        // Redis: TTL 切れキャッシュを削除
        // Metrics: cleanup_count, cleanup_size_bytes を記録
        // Log: 削除対象数とサイズ
    }
}

// Metrics export
pub struct MetricsExportTask;

impl TaskHandler for MetricsExportTask {
    async fn execute(&self, context: &ScheduleContext) -> Result<TaskResult> {
        // Prometheus メトリクスをファイル or S3 へ export
        // Daily summary 生成
        // 通知システムで Slack/Email に送信
    }
}

// Storage cleanup
pub struct StorageRetentionTask;

impl TaskHandler for StorageRetentionTask {
    async fn execute(&self, context: &ScheduleContext) -> Result<TaskResult> {
        // 1年以上前のファイルを削除
        // Storage backend から実削除
        // 監査ログ記録
    }
}
```

#### Task 3: Web API for Scheduler
```rust
// GET /api/v1/scheduler/tasks
pub async fn list_scheduled_tasks(
    scheduler: web::Data<CronScheduler>,
) -> Result<HttpResponse> {
    // すべての登録済みタスク一覧を返却
    // status, next_run, last_run 含む
}

// POST /api/v1/scheduler/tasks/{id}/trigger
pub async fn trigger_task_now(
    scheduler: web::Data<CronScheduler>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // 指定タスクを即座に実行
    // 実行結果を返却
}

// GET /api/v1/scheduler/tasks/{id}/history
pub async fn get_task_history(
    scheduler: web::Data<CronScheduler>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // タスク実行履歴を返却
    // status, duration, error を含む
}
```

#### Task 4: 初期設定スクリプト
```bash
# scripts/setup-scheduler.sh
#!/bin/bash

# 1. Cron tasks を config に登録
cat >> config/production.toml <<EOF
[scheduler]
enabled = true

[[scheduler.tasks]]
id = "cache-cleanup"
name = "Cache Cleanup"
cron = "0 2 * * *"  # Daily 2am
enabled = true

[[scheduler.tasks]]
id = "metrics-export"
name = "Daily Metrics Export"
cron = "0 3 * * *"  # Daily 3am
enabled = true
export_to = "slack"  # or "email", "s3"

[[scheduler.tasks]]
id = "storage-retention"
name = "Weekly Storage Cleanup"
cron = "0 4 * * 0"  # Weekly Sunday 4am
enabled = true
retention_days = 365
EOF

# 2. テストタスク実行
cargo run -- scheduler test-task cache-cleanup
```

### テスト戦略

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_scheduler_registration() {
        // タスク登録テスト
    }

    #[tokio::test]
    async fn test_cron_expression_parsing() {
        // "0 2 * * *" を正しく解析
        // 次実行時刻を計算
    }

    #[tokio::test]
    async fn test_task_execution() {
        // タスク実行テスト
        // 成功 / 失敗 / タイムアウト
    }

    #[tokio::test]
    async fn test_scheduler_persistence() {
        // Redis: 実行履歴を永続化
        // DB: タスク設定を保存
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        // 複数タスク同時実行
        // リソース競合なし
    }
}
```

---

## 2️⃣ マルチプラットフォーム通知システム

### 目標
```
スケジュール実行結果、エラー、アラート を複数チャネルに送信
├─ Slack
├─ Discord  
├─ Email
└─ Telegram (オプション)
```

### 技術スタック

```toml
[dependencies]
slack-morphism = "0.42"   # Slack API
serenity = "0.12"         # Discord (optional)
lettre = "0.11"           # Email
reqwest = "0.12"          # HTTP client
serde_json = "1.0"        # JSON serialization
```

### 実装設計

```rust
// src/notification/mod.rs
pub trait NotificationChannel: Send + Sync {
    async fn send(&self, message: &NotificationMessage) -> Result<String>;
}

pub struct NotificationMessage {
    pub channel_type: ChannelType,
    pub title: String,
    pub body: String,
    pub severity: Severity,
    pub context: serde_json::Value,  // Metadata: task_id, duration, etc.
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

pub enum ChannelType {
    Slack,
    Discord,
    Email,
    Telegram,
}
```

#### Slack 実装
```rust
pub struct SlackChannel {
    webhook_url: String,
    default_channel: String,  // #notifications, #errors, etc.
    client: reqwest::Client,
}

impl SlackChannel {
    pub fn new(webhook_url: String, default_channel: String) -> Self;
}

impl NotificationChannel for SlackChannel {
    async fn send(&self, message: &NotificationMessage) -> Result<String> {
        // Slack Block Kit で formatted message を構築
        let blocks = match message.severity {
            Severity::Error | Severity::Critical => {
                // Red header, error details, logs
                create_error_blocks(message)
            }
            Severity::Warning => {
                // Yellow header, warnings
                create_warning_blocks(message)
            }
            _ => {
                // Green header, success details
                create_success_blocks(message)
            }
        };

        // Webhook 経由で送信
        let response = self.client.post(&self.webhook_url)
            .json(&SlackMessage { blocks })
            .send()
            .await?;

        Ok(response.status().to_string())
    }
}

// Slack Block Kit 構造
#[derive(Serialize)]
struct SlackMessage {
    blocks: Vec<serde_json::Value>,
}

fn create_error_blocks(msg: &NotificationMessage) -> Vec<serde_json::Value> {
    vec![
        json!({
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": format!("🔴 {}", msg.title),
                "emoji": true
            }
        }),
        json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": msg.body
            }
        }),
        json!({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": format!("Task: {}", msg.context["task_id"])
                }
            ]
        })
    ]
}
```

#### Email 実装
```rust
pub struct EmailChannel {
    smtp_host: String,
    smtp_port: u16,
    from_address: String,
    to_address: String,
    username: Option<String>,
    password: Option<String>,
}

impl EmailChannel {
    pub async fn send_html_email(
        &self,
        title: &str,
        body: &str,
        context: &serde_json::Value,
    ) -> Result<()> {
        // HTML email を構築
        let html_body = format!(
            r#"
            <html>
                <body style="font-family: Arial;">
                    <h1>{}</h1>
                    <p>{}</p>
                    <hr>
                    <small>{}</small>
                </body>
            </html>
            "#,
            title, body, serde_json::to_string_pretty(context)?
        );

        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(self.to_address.parse()?)
            .subject(title)
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(body.to_string()))
                    .singlepart(SinglePart::html(html_body))
            )?;

        // SMTP で送信
        let transport = SmtpTransport::relay(&self.smtp_host)?
            .port(self.smtp_port)
            .credentials(Credentials::new(...))
            .build();

        transport.send(&email)?;
        Ok(())
    }
}

impl NotificationChannel for EmailChannel {
    async fn send(&self, message: &NotificationMessage) -> Result<String> {
        self.send_html_email(&message.title, &message.body, &message.context)
            .await?;
        Ok("Email sent".to_string())
    }
}
```

#### NotificationRouter
```rust
pub struct NotificationRouter {
    channels: RwLock<HashMap<ChannelType, Arc<dyn NotificationChannel>>>,
    config: NotificationConfig,
}

pub struct NotificationConfig {
    pub slack_enabled: bool,
    pub slack_webhook_url: Option<String>,
    pub email_enabled: bool,
    pub email_to: Option<String>,
    pub discord_enabled: bool,
    pub discord_webhook_url: Option<String>,
}

impl NotificationRouter {
    pub async fn send_to_all(&self, message: &NotificationMessage) -> Result<Vec<String>> {
        let mut results = Vec::new();
        
        // 複数チャネルに並列で送信
        let futures = self.channels.read().await
            .iter()
            .map(|(_, channel)| channel.send(message));

        for result in futures::future::join_all(futures).await {
            match result {
                Ok(msg_id) => results.push(msg_id),
                Err(e) => tracing::warn!("Notification send error: {:?}", e),
            }
        }

        Ok(results)
    }

    pub async fn send(&self, message: &NotificationMessage) -> Result<Vec<String>> {
        // Severity に応じてチャネルを選択
        match message.severity {
            Severity::Critical => self.send_to_all(message).await,
            Severity::Error => {
                // Slack + Email
                self.send_to_channels(message, &[ChannelType::Slack, ChannelType::Email]).await
            }
            Severity::Warning => {
                // Slack only
                self.send_to_channels(message, &[ChannelType::Slack]).await
            }
            _ => Ok(vec![]),
        }
    }
}
```

#### Scheduler 統合
```rust
// Scheduler task 実行完了時に通知を送信
pub async fn execute_and_notify(
    task: &ScheduledTask,
    scheduler_context: &ScheduleContext,
    notifier: &NotificationRouter,
) -> Result<TaskResult> {
    let start = Instant::now();
    
    let task_result = task.handler.execute(scheduler_context).await;
    
    let duration = start.elapsed();
    
    // 結果に応じて通知
    let notification = match &task_result {
        Ok(result) if result.status == TaskStatus::Success => {
            NotificationMessage {
                channel_type: ChannelType::Slack,
                title: format!("✅ Task Completed: {}", task.name),
                body: format!("Duration: {:.2}s\nMessage: {}", 
                    duration.as_secs_f64(), result.message),
                severity: Severity::Info,
                context: json!({
                    "task_id": task.id,
                    "duration_ms": duration.as_millis(),
                }),
            }
        }
        Err(e) => {
            NotificationMessage {
                channel_type: ChannelType::Slack,
                title: format!("❌ Task Failed: {}", task.name),
                body: format!("Error: {}\nDuration: {:.2}s", e, duration.as_secs_f64()),
                severity: Severity::Error,
                context: json!({
                    "task_id": task.id,
                    "error": e.to_string(),
                }),
            }
        }
        _ => return task_result,
    };

    // 通知送信（エラーは log のみ）
    let _ = notifier.send(&notification).await;
    
    task_result
}
```

### テスト戦略

```rust
#[cfg(test)]
mod tests {
    use crate::notification::*;

    #[tokio::test]
    async fn test_slack_notification() {
        // Mock Slack webhook を使用
        // message format 確認
    }

    #[tokio::test]
    async fn test_email_notification() {
        // Mock SMTP server を使用
        // HTML format 確認
    }

    #[tokio::test]
    async fn test_notification_routing() {
        // Severity に応じて正しいチャネルに送信
    }

    #[tokio::test]
    async fn test_notification_retry() {
        // 送信失敗時のリトライロジック
    }
}
```

---

## 3️⃣ 統合タイムライン

### Day 16 (3-4 hours)
```
1. Cron Scheduler 基盤実装
2. Built-in tasks (cleanup, metrics export)
3. Unit tests (5)
```

### Day 17 (3-4 hours)
```
1. Notification system 実装
2. Slack/Email integration
3. Scheduler ← Notifier 統合
4. Integration tests (5)
5. API endpoints (GET /scheduler/tasks)
```

### テスト計画
```
Scheduler tests (Day 16): 5
Notification tests (Day 17): 5
Integration tests (Day 17): 5
─────────────────────────
Total: 15 tests

既存 Wave 4 tests: 27
新規 Hermes 統合: 15
────────────────────
Wave 4 Total: 42 tests
```

---

## 4️⃣ 設定例

### config/production.toml
```toml
[scheduler]
enabled = true
max_concurrent_tasks = 3
task_timeout_seconds = 3600

[[scheduler.tasks]]
id = "cache-cleanup"
name = "Cache Cleanup"
cron = "0 2 * * *"
handler = "cache_cleanup"
enabled = true

[[scheduler.tasks]]
id = "metrics-export"
name = "Daily Metrics Export"
cron = "0 3 * * *"
handler = "metrics_export"
enabled = true
export_to = ["slack", "email"]

[[scheduler.tasks]]
id = "storage-retention"
name = "Weekly Storage Cleanup"
cron = "0 4 * * 0"
handler = "storage_retention"
enabled = true
retention_days = 365

[notification]
slack_enabled = true
slack_webhook_url = "${SLACK_WEBHOOK_URL}"

email_enabled = true
email_smtp_host = "${EMAIL_SMTP_HOST}"
email_smtp_port = 587
email_from = "opencode@example.com"
email_to = "ops@example.com"
```

### 環境変数
```bash
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/WEBHOOK/URL
EMAIL_SMTP_HOST=smtp.gmail.com
EMAIL_SMTP_PASSWORD=your_app_password
```

---

## 5️⃣ 成果物チェックリスト

- [ ] `src/scheduler/mod.rs` - Cron scheduler
- [ ] `src/scheduler/tasks.rs` - Built-in tasks
- [ ] `src/notification/mod.rs` - Notification system
- [ ] `src/notification/slack.rs` - Slack integration
- [ ] `src/notification/email.rs` - Email integration
- [ ] `src/api/scheduler.rs` - Scheduler API endpoints
- [ ] `tests/scheduler/` - Scheduler tests (5)
- [ ] `tests/notification/` - Notification tests (5)
- [ ] `tests/integration/` - Integration tests (5)
- [ ] `docs/SCHEDULER_GUIDE.md` - User documentation
- [ ] `config/development.toml` - Updated config
- [ ] `config/production.toml` - Updated config

---

**Recommendation**: Wave 4 Week 4 (Days 11-15) 完了後、Day 16-17 で Hermes 統合を実施

**Effort**: 6-8 hours (開発 + テスト)  
**Test Coverage**: 15 新規テスト
