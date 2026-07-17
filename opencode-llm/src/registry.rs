//! Registry system for tasks, teams, and cron jobs.
//!
//! Provides three registries:
//! - [`TaskRegistry`] — manage async task definitions with status tracking
//! - [`TeamRegistry`] — manage AI agent team compositions
//! - [`CronRegistry`] — schedule recurring jobs with cron expressions
//!
//! Inspired by `claw-code`'s registry architecture.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::error::{LlmError, LlmResult};

// ---------------------------------------------------------------------------
// TaskRegistry
// ---------------------------------------------------------------------------

/// Status of a registered task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is ready to be picked up.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
    /// Task is blocked on a dependency.
    Blocked,
}

impl TaskStatus {
    /// Whether the task is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
}

/// A registered task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// Current status.
    pub status: TaskStatus,
    /// Priority (higher = more urgent).
    pub priority: i32,
    /// IDs of tasks this task depends on.
    pub depends_on: Vec<String>,
    /// IDs of tasks that depend on this task.
    #[serde(default)]
    pub dependents: Vec<String>,
    /// When the task was created.
    pub created_at: DateTime<Utc>,
    /// When the task was last updated.
    pub updated_at: DateTime<Utc>,
    /// Optional assigned agent/team name.
    pub assignee: Option<String>,
    /// Arbitrary tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional result/error message.
    pub result: Option<String>,
    /// Maximum attempts before marking as failed.
    pub max_retries: u32,
    /// Current retry count.
    #[serde(default)]
    pub retry_count: u32,
}

impl Task {
    /// Create a new pending task.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            status: TaskStatus::Pending,
            priority: 0,
            depends_on: Vec::new(),
            dependents: Vec::new(),
            created_at: now,
            updated_at: now,
            assignee: None,
            tags: Vec::new(),
            result: None,
            max_retries: 3,
            retry_count: 0,
        }
    }

    /// Set the task description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the task priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a dependency.
    pub fn with_dependency(mut self, dep_id: impl Into<String>) -> Self {
        self.depends_on.push(dep_id.into());
        self
    }
}

/// Registry for async task management.
pub struct TaskRegistry {
    tasks: Mutex<BTreeMap<String, Task>>,
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRegistry {
    /// Create a new empty task registry.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(BTreeMap::new()),
        }
    }

    /// Register a new task.
    pub async fn register(&self, task: Task) -> LlmResult<()> {
        let mut tasks = self.tasks.lock().await;
        if tasks.contains_key(&task.id) {
            return Err(LlmError::Internal(format!(
                "task `{}` already registered",
                task.id
            )));
        }
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    /// Get a task by ID.
    pub async fn get(&self, id: &str) -> Option<Task> {
        self.tasks.lock().await.get(id).cloned()
    }

    /// Update a task's status.
    pub async fn update_status(&self, id: &str, status: TaskStatus) -> LlmResult<()> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks
            .get_mut(id)
            .ok_or_else(|| LlmError::Internal(format!("task `{id}` not found")))?;
        task.status = status;
        task.updated_at = Utc::now();
        Ok(())
    }

    /// Update a task's result.
    pub async fn update_result(&self, id: &str, result: String) -> LlmResult<()> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks
            .get_mut(id)
            .ok_or_else(|| LlmError::Internal(format!("task `{id}` not found")))?;
        task.result = Some(result);
        task.updated_at = Utc::now();
        Ok(())
    }

    /// Increment retry count for a task.
    pub async fn increment_retry(&self, id: &str) -> LlmResult<u32> {
        let mut tasks = self.tasks.lock().await;
        let task = tasks
            .get_mut(id)
            .ok_or_else(|| LlmError::Internal(format!("task `{id}` not found")))?;
        task.retry_count += 1;
        task.updated_at = Utc::now();
        Ok(task.retry_count)
    }

    /// List tasks by status.
    pub async fn list_by_status(&self, status: TaskStatus) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        tasks
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    /// List all tasks.
    pub async fn list_all(&self) -> Vec<Task> {
        self.tasks.lock().await.values().cloned().collect()
    }

    /// Delete a task.
    pub async fn delete(&self, id: &str) -> LlmResult<()> {
        let mut tasks = self.tasks.lock().await;
        tasks
            .remove(id)
            .ok_or_else(|| LlmError::Internal(format!("task `{id}` not found")))?;
        Ok(())
    }

    /// Find tasks that are ready to execute (pending + no uncompleted dependencies).
    pub async fn ready_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        tasks
            .values()
            .filter(|t| {
                if t.status != TaskStatus::Pending {
                    return false;
                }
                t.depends_on.iter().all(|dep_id| {
                    tasks
                        .get(dep_id)
                        .map(|dep| dep.status == TaskStatus::Completed)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .collect()
    }

    /// Clear all tasks.
    pub async fn clear(&self) {
        self.tasks.lock().await.clear();
    }

    /// Number of registered tasks.
    pub async fn len(&self) -> usize {
        self.tasks.lock().await.len()
    }

    /// Whether the registry is empty.
    pub async fn is_empty(&self) -> bool {
        self.tasks.lock().await.is_empty()
    }
}

// ---------------------------------------------------------------------------
// TeamRegistry
// ---------------------------------------------------------------------------

/// Capabilities of a team member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberCapability {
    /// Tool names this member can execute.
    pub tools: Vec<String>,
    /// Whether this member can write files.
    pub can_write: bool,
    /// Whether this member can execute shell commands.
    pub can_exec_shell: bool,
    /// Maximum timeout in seconds for this member.
    pub max_timeout_secs: u64,
}

impl Default for TeamMemberCapability {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            can_write: false,
            can_exec_shell: false,
            max_timeout_secs: 60,
        }
    }
}

/// A team member (agent or human) registered in the team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// Unique member name/ID.
    pub name: String,
    /// Human-readable role description.
    pub role: String,
    /// Model this member uses (if applicable).
    pub model: Option<String>,
    /// Whether this member is enabled.
    pub enabled: bool,
    /// Capabilities.
    pub capabilities: TeamMemberCapability,
}

/// A registered team of agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Unique team name.
    pub name: String,
    /// Team description.
    pub description: String,
    /// Team members.
    pub members: Vec<TeamMember>,
    /// Whether the team is enabled.
    pub enabled: bool,
    /// Maximum number of conversation rounds for this team.
    pub max_rounds: usize,
}

impl Team {
    /// Create a new team.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            members: Vec::new(),
            enabled: true,
            max_rounds: 64,
        }
    }

    /// Add a member to this team.
    pub fn add_member(&mut self, member: TeamMember) {
        self.members.push(member);
    }
}

/// Registry for AI agent team compositions.
pub struct TeamRegistry {
    teams: Mutex<BTreeMap<String, Team>>,
}

impl Default for TeamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamRegistry {
    /// Create a new empty team registry.
    pub fn new() -> Self {
        Self {
            teams: Mutex::new(BTreeMap::new()),
        }
    }

    /// Register a team.
    pub async fn register(&self, team: Team) -> LlmResult<()> {
        let mut teams = self.teams.lock().await;
        if teams.contains_key(&team.name) {
            return Err(LlmError::Internal(format!(
                "team `{}` already registered",
                team.name
            )));
        }
        teams.insert(team.name.clone(), team);
        Ok(())
    }

    /// Get a team by name.
    pub async fn get(&self, name: &str) -> Option<Team> {
        self.teams.lock().await.get(name).cloned()
    }

    /// List all teams.
    pub async fn list_all(&self) -> Vec<Team> {
        self.teams.lock().await.values().cloned().collect()
    }

    /// List enabled teams.
    pub async fn list_enabled(&self) -> Vec<Team> {
        let teams = self.teams.lock().await;
        teams
            .values()
            .filter(|t| t.enabled)
            .cloned()
            .collect()
    }

    /// Delete a team.
    pub async fn delete(&self, name: &str) -> LlmResult<()> {
        let mut teams = self.teams.lock().await;
        teams
            .remove(name)
            .ok_or_else(|| LlmError::Internal(format!("team `{name}` not found")))?;
        Ok(())
    }

    /// Enable or disable a team.
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> LlmResult<()> {
        let mut teams = self.teams.lock().await;
        let team = teams
            .get_mut(name)
            .ok_or_else(|| LlmError::Internal(format!("team `{name}` not found")))?;
        team.enabled = enabled;
        Ok(())
    }

    /// Add a member to an existing team.
    pub async fn add_member(&self, team_name: &str, member: TeamMember) -> LlmResult<()> {
        let mut teams = self.teams.lock().await;
        let team = teams
            .get_mut(team_name)
            .ok_or_else(|| LlmError::Internal(format!("team `{team_name}` not found")))?;
        team.members.push(member);
        Ok(())
    }

    /// Remove a member from a team.
    pub async fn remove_member(&self, team_name: &str, member_name: &str) -> LlmResult<()> {
        let mut teams = self.teams.lock().await;
        let team = teams
            .get_mut(team_name)
            .ok_or_else(|| LlmError::Internal(format!("team `{team_name}` not found")))?;
        let before = team.members.len();
        team.members.retain(|m| m.name != member_name);
        if team.members.len() == before {
            return Err(LlmError::Internal(format!(
                "member `{member_name}` not found in team `{team_name}`"
            )));
        }
        Ok(())
    }

    /// Clear all teams.
    pub async fn clear(&self) {
        self.teams.lock().await.clear();
    }
}

// ---------------------------------------------------------------------------
// CronRegistry
// ---------------------------------------------------------------------------

/// A parsed cron expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    /// Raw cron expression string (e.g. `"0 */6 * * *"`).
    pub expression: String,
    /// Human-readable description of the schedule.
    pub description: String,
}

impl CronSchedule {
    /// Create a new cron schedule.
    pub fn new(expression: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            description: description.into(),
        }
    }

    /// Common: every hour.
    pub fn hourly() -> Self {
        Self::new("0 * * * *", "Every hour")
    }

    /// Common: every 6 hours.
    pub fn every_6_hours() -> Self {
        Self::new("0 */6 * * *", "Every 6 hours")
    }

    /// Common: daily at midnight.
    pub fn daily() -> Self {
        Self::new("0 0 * * *", "Daily at midnight")
    }

    /// Common: weekly on Monday at midnight.
    pub fn weekly() -> Self {
        Self::new("0 0 * * 1", "Weekly on Monday")
    }
}

/// Job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is active and will run on schedule.
    Active,
    /// Job is paused.
    Paused,
    /// Job has been retired.
    Retired,
    /// Job failed on last execution.
    Failed,
}

/// A registered cron job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    /// Unique job name/ID.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Cron schedule.
    pub schedule: CronSchedule,
    /// Current status.
    pub status: JobStatus,
    /// Last run time.
    pub last_run_at: Option<DateTime<Utc>>,
    /// Last run result.
    pub last_result: Option<String>,
    /// Number of consecutive failures.
    pub consecutive_failures: u32,
    /// Maximum consecutive failures before auto-pausing.
    pub max_failures_before_pause: u32,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CronJob {
    /// Create a new cron job.
    pub fn new(name: impl Into<String>, description: impl Into<String>, schedule: CronSchedule) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schedule,
            status: JobStatus::Active,
            last_run_at: None,
            last_result: None,
            consecutive_failures: 0,
            max_failures_before_pause: 5,
            created_at: Utc::now(),
            tags: Vec::new(),
        }
    }
}

/// Trait for cron job execution logic.
#[async_trait]
pub trait CronHandler: Send + Sync {
    /// Execute the job's task. Return a result string on success.
    async fn execute(&self) -> LlmResult<String>;
}

/// Registry for scheduled cron jobs.
pub struct CronRegistry {
    jobs: Mutex<BTreeMap<String, CronJob>>,
    handlers: Mutex<BTreeMap<String, Arc<dyn CronHandler>>>,
}

impl Default for CronRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CronRegistry {
    /// Create a new empty cron registry.
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(BTreeMap::new()),
            handlers: Mutex::new(BTreeMap::new()),
        }
    }

    /// Register a cron job.
    pub async fn register(&self, job: CronJob, handler: Arc<dyn CronHandler>) -> LlmResult<()> {
        let mut jobs = self.jobs.lock().await;
        if jobs.contains_key(&job.name) {
            return Err(LlmError::Internal(format!(
                "cron job `{}` already registered",
                job.name
            )));
        }
        let name = job.name.clone();
        jobs.insert(name.clone(), job);
        self.handlers.lock().await.insert(name, handler);
        Ok(())
    }

    /// Get a cron job by name.
    pub async fn get(&self, name: &str) -> Option<CronJob> {
        self.jobs.lock().await.get(name).cloned()
    }

    /// List all cron jobs.
    pub async fn list_all(&self) -> Vec<CronJob> {
        self.jobs.lock().await.values().cloned().collect()
    }

    /// List active cron jobs (that should be running on schedule).
    pub async fn list_active(&self) -> Vec<CronJob> {
        let jobs = self.jobs.lock().await;
        jobs
            .values()
            .filter(|j| j.status == JobStatus::Active)
            .cloned()
            .collect()
    }

    /// Execute a cron job by name.
    pub async fn execute(&self, name: &str) -> LlmResult<String> {
        let handler = {
            let handlers = self.handlers.lock().await;
            handlers
                .get(name)
                .cloned()
                .ok_or_else(|| LlmError::Internal(format!("cron job `{name}` has no handler")))?
        };

        let result = match handler.execute().await {
            Ok(output) => {
                let mut jobs = self.jobs.lock().await;
                if let Some(job) = jobs.get_mut(name) {
                    job.last_run_at = Some(Utc::now());
                    job.last_result = Some(output.clone());
                    job.consecutive_failures = 0;
                }
                info!(cron_job = name, "cron job executed successfully");
                output
            }
            Err(e) => {
                let mut jobs = self.jobs.lock().await;
                if let Some(job) = jobs.get_mut(name) {
                    job.last_run_at = Some(Utc::now());
                    job.last_result = Some(format!("error: {e}"));
                    job.consecutive_failures += 1;
                    if job.consecutive_failures >= job.max_failures_before_pause {
                        job.status = JobStatus::Failed;
                        warn!(
                            cron_job = name,
                            failures = job.consecutive_failures,
                            "auto-pausing cron job after too many failures"
                        );
                    }
                }
                return Err(e);
            }
        };

        Ok(result)
    }

    /// Pause a cron job.
    pub async fn pause(&self, name: &str) -> LlmResult<()> {
        let mut jobs = self.jobs.lock().await;
        let job = jobs
            .get_mut(name)
            .ok_or_else(|| LlmError::Internal(format!("cron job `{name}` not found")))?;
        job.status = JobStatus::Paused;
        Ok(())
    }

    /// Resume a paused or failed cron job.
    pub async fn resume(&self, name: &str) -> LlmResult<()> {
        let mut jobs = self.jobs.lock().await;
        let job = jobs
            .get_mut(name)
            .ok_or_else(|| LlmError::Internal(format!("cron job `{name}` not found")))?;
        job.status = JobStatus::Active;
        job.consecutive_failures = 0;
        Ok(())
    }

    /// Delete a cron job.
    pub async fn delete(&self, name: &str) -> LlmResult<()> {
        let mut jobs = self.jobs.lock().await;
        jobs.remove(name)
            .ok_or_else(|| LlmError::Internal(format!("cron job `{name}` not found")))?;
        self.handlers.lock().await.remove(name);
        Ok(())
    }

    /// Number of registered cron jobs.
    pub async fn len(&self) -> usize {
        self.jobs.lock().await.len()
    }

    /// Whether the registry is empty.
    pub async fn is_empty(&self) -> bool {
        self.jobs.lock().await.is_empty()
    }

    /// Clear all cron jobs.
    pub async fn clear(&self) {
        self.jobs.lock().await.clear();
        self.handlers.lock().await.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- TaskRegistry tests ----

    #[tokio::test]
    async fn task_register_and_get() {
        let reg = TaskRegistry::new();
        let task = Task::new("task-1", "Test task").with_description("A test");
        reg.register(task).await.unwrap();
        let retrieved = reg.get("task-1").await.unwrap();
        assert_eq!(retrieved.title, "Test task");
        assert_eq!(retrieved.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn task_duplicate_fails() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("dup", "First")).await.unwrap();
        let err = reg.register(Task::new("dup", "Second")).await.unwrap_err();
        assert!(matches!(err, LlmError::Internal(_)));
    }

    #[tokio::test]
    async fn task_update_status() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("t1", "Task 1")).await.unwrap();
        reg.update_status("t1", TaskStatus::Running).await.unwrap();
        let task = reg.get("t1").await.unwrap();
        assert_eq!(task.status, TaskStatus::Running);
    }

    #[tokio::test]
    async fn task_update_result() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("t1", "Task 1")).await.unwrap();
        reg.update_result("t1", "done".into()).await.unwrap();
        let task = reg.get("t1").await.unwrap();
        assert_eq!(task.result, Some("done".into()));
    }

    #[tokio::test]
    async fn task_increment_retry() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("t1", "Task 1")).await.unwrap();
        assert_eq!(reg.increment_retry("t1").await.unwrap(), 1);
        assert_eq!(reg.increment_retry("t1").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn task_list_by_status() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("t1", "Task 1")).await.unwrap();
        reg.update_status("t1", TaskStatus::Completed).await.unwrap();
        reg.register(Task::new("t2", "Task 2")).await.unwrap();
        let pending = reg.list_by_status(TaskStatus::Pending).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "t2");
    }

    #[tokio::test]
    async fn task_ready_tasks_respects_dependencies() {
        let reg = TaskRegistry::new();
        reg.register(
            Task::new("t1", "Parent")
                .with_priority(1),
        )
        .await
        .unwrap();
        reg.register(
            Task::new("t2", "Child")
                .with_dependency("t1")
                .with_priority(2),
        )
        .await
        .unwrap();
        // Only t1 should be ready (t2 depends on t1).
        let ready = reg.ready_tasks().await;
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t1");

        // Complete t1 → t2 becomes ready.
        reg.update_status("t1", TaskStatus::Completed).await.unwrap();
        let ready = reg.ready_tasks().await;
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t2");
    }

    #[tokio::test]
    async fn task_delete() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("t1", "Task 1")).await.unwrap();
        reg.delete("t1").await.unwrap();
        assert!(reg.get("t1").await.is_none());
    }

    #[tokio::test]
    async fn task_clear() {
        let reg = TaskRegistry::new();
        reg.register(Task::new("t1", "Task 1")).await.unwrap();
        reg.register(Task::new("t2", "Task 2")).await.unwrap();
        reg.clear().await;
        assert_eq!(reg.len().await, 0);
    }

    #[tokio::test]
    async fn task_is_terminal() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
        assert!(!TaskStatus::Blocked.is_terminal());
    }

    // ---- TeamRegistry tests ----

    #[tokio::test]
    async fn team_register_and_get() {
        let reg = TeamRegistry::new();
        let team = Team::new("dev-team", "Development team");
        reg.register(team).await.unwrap();
        let retrieved = reg.get("dev-team").await.unwrap();
        assert_eq!(retrieved.name, "dev-team");
    }

    #[tokio::test]
    async fn team_duplicate_fails() {
        let reg = TeamRegistry::new();
        reg.register(Team::new("same", "First")).await.unwrap();
        let err = reg.register(Team::new("same", "Second")).await.unwrap_err();
        assert!(matches!(err, LlmError::Internal(_)));
    }

    #[tokio::test]
    async fn team_add_and_remove_member() {
        let reg = TeamRegistry::new();
        reg.register(Team::new("team", "Test")).await.unwrap();
        let member = TeamMember {
            name: "agent-a".into(),
            role: "coder".into(),
            model: Some("claude-sonnet-4".into()),
            enabled: true,
            capabilities: TeamMemberCapability {
                tools: vec!["bash".into()],
                can_write: true,
                can_exec_shell: true,
                max_timeout_secs: 120,
            },
        };
        reg.add_member("team", member).await.unwrap();
        let team = reg.get("team").await.unwrap();
        assert_eq!(team.members.len(), 1);
        reg.remove_member("team", "agent-a").await.unwrap();
        let team = reg.get("team").await.unwrap();
        assert!(team.members.is_empty());
    }

    #[tokio::test]
    async fn team_list_enabled() {
        let reg = TeamRegistry::new();
        reg.register(Team::new("enabled-team", "Active")).await.unwrap();
        reg.register(Team::new("disabled-team", "Inactive")).await.unwrap();
        reg.set_enabled("disabled-team", false).await.unwrap();
        let enabled = reg.list_enabled().await;
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "enabled-team");
    }

    #[tokio::test]
    async fn team_delete() {
        let reg = TeamRegistry::new();
        reg.register(Team::new("delete-me", "Gone")).await.unwrap();
        reg.delete("delete-me").await.unwrap();
        assert!(reg.get("delete-me").await.is_none());
    }

    #[tokio::test]
    async fn team_remove_nonexistent_member_fails() {
        let reg = TeamRegistry::new();
        reg.register(Team::new("team", "Test")).await.unwrap();
        let err = reg.remove_member("team", "ghost").await.unwrap_err();
        assert!(matches!(err, LlmError::Internal(_)));
    }

    // ---- CronRegistry tests ----

    #[tokio::test]
    async fn cron_register_and_get() {
        struct TestHandler;
        #[async_trait]
        impl CronHandler for TestHandler {
            async fn execute(&self) -> LlmResult<String> {
                Ok("ok".into())
            }
        }

        let reg = CronRegistry::new();
        let job = CronJob::new("hourly-cleanup", "Hourly cleanup", CronSchedule::hourly());
        reg.register(job, Arc::new(TestHandler)).await.unwrap();
        let retrieved = reg.get("hourly-cleanup").await.unwrap();
        assert_eq!(retrieved.name, "hourly-cleanup");
        assert_eq!(retrieved.status, JobStatus::Active);
    }

    #[tokio::test]
    async fn cron_duplicate_fails() {
        let reg = CronRegistry::new();
        struct H;
        #[async_trait]
        impl CronHandler for H {
            async fn execute(&self) -> LlmResult<String> {
                Ok("ok".into())
            }
        }
        reg.register(
            CronJob::new("dup", "First", CronSchedule::hourly()),
            Arc::new(H),
        )
        .await
        .unwrap();
        let err = reg
            .register(
                CronJob::new("dup", "Second", CronSchedule::hourly()),
                Arc::new(H),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, LlmError::Internal(_)));
    }

    #[tokio::test]
    async fn cron_execute_success() {
        struct OkHandler;
        #[async_trait]
        impl CronHandler for OkHandler {
            async fn execute(&self) -> LlmResult<String> {
                Ok("success output".into())
            }
        }

        let reg = CronRegistry::new();
        reg.register(
            CronJob::new("ok-job", "Ok job", CronSchedule::hourly()),
            Arc::new(OkHandler),
        )
        .await
        .unwrap();

        let result = reg.execute("ok-job").await.unwrap();
        assert_eq!(result, "success output");

        let job = reg.get("ok-job").await.unwrap();
        assert!(job.last_run_at.is_some());
        assert_eq!(job.last_result, Some("success output".into()));
        assert_eq!(job.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn cron_execute_failure_auto_pauses() {
        struct FailHandler;
        #[async_trait]
        impl CronHandler for FailHandler {
            async fn execute(&self) -> LlmResult<String> {
                Err(LlmError::Internal("something went wrong".into()))
            }
        }

        let reg = CronRegistry::new();
        let mut job = CronJob::new("fail-job", "Fails", CronSchedule::hourly());
        job.max_failures_before_pause = 2; // Lower threshold for test
        reg.register(job, Arc::new(FailHandler)).await.unwrap();

        // First failure → still active
        assert!(reg.execute("fail-job").await.is_err());
        let job = reg.get("fail-job").await.unwrap();
        assert_eq!(job.status, JobStatus::Active);
        assert_eq!(job.consecutive_failures, 1);

        // Second failure → auto-paused
        assert!(reg.execute("fail-job").await.is_err());
        let job = reg.get("fail-job").await.unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.consecutive_failures, 2);
    }

    #[tokio::test]
    async fn cron_pause_and_resume() {
        struct H;
        #[async_trait]
        impl CronHandler for H {
            async fn execute(&self) -> LlmResult<String> {
                Ok("ok".into())
            }
        }
        let reg = CronRegistry::new();
        reg.register(
            CronJob::new("job", "Test", CronSchedule::hourly()),
            Arc::new(H),
        )
        .await
        .unwrap();
        reg.pause("job").await.unwrap();
        assert_eq!(reg.get("job").await.unwrap().status, JobStatus::Paused);
        reg.resume("job").await.unwrap();
        assert_eq!(reg.get("job").await.unwrap().status, JobStatus::Active);
    }

    #[tokio::test]
    async fn cron_delete() {
        struct H;
        #[async_trait]
        impl CronHandler for H {
            async fn execute(&self) -> LlmResult<String> {
                Ok("ok".into())
            }
        }
        let reg = CronRegistry::new();
        reg.register(
            CronJob::new("del", "Delete me", CronSchedule::hourly()),
            Arc::new(H),
        )
        .await
        .unwrap();
        reg.delete("del").await.unwrap();
        assert!(reg.get("del").await.is_none());
    }

    #[tokio::test]
    async fn cron_list_active() {
        struct H;
        #[async_trait]
        impl CronHandler for H {
            async fn execute(&self) -> LlmResult<String> {
                Ok("ok".into())
            }
        }
        let reg = CronRegistry::new();
        reg.register(
            CronJob::new("active-job", "Active", CronSchedule::hourly()),
            Arc::new(H),
        )
        .await
        .unwrap();
        reg.register(
            CronJob::new("paused-job", "Paused", CronSchedule::daily()),
            Arc::new(H),
        )
        .await
        .unwrap();
        reg.pause("paused-job").await.unwrap();
        let active = reg.list_active().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "active-job");
    }

    #[tokio::test]
    async fn cron_schedule_descriptions() {
        assert!(CronSchedule::hourly().expression.contains("0 *"));
        assert!(CronSchedule::daily().expression.contains("0 0 * * *"));
        assert!(CronSchedule::weekly().expression.contains("* 1"));
        assert!(CronSchedule::every_6_hours().expression.contains("*/6"));
    }
}