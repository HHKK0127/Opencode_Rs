//! Git context — gather repository metadata for the LLM runtime.
//!
//! Wraps the `git` CLI to provide structured access to branch info,
//! recent commits, working-tree status, and diffs. No extra Rust git
//! library is required; the CLI is invoked via `tokio::process`.
//!
//! Inspired by `claw-code`'s git integration.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::error::{LlmError, LlmResult};

/// A single commit entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Commit {
    /// Full commit hash.
    pub hash: String,
    /// Short hash (first 7 chars).
    pub short_hash: String,
    /// Author name.
    pub author: String,
    /// Author email.
    pub email: String,
    /// Commit date in ISO 8601.
    pub date: String,
    /// Commit subject (first line of message).
    pub subject: String,
    /// Full commit message (may be multiline).
    #[serde(default)]
    pub body: String,
}

/// A single changed file in the working tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileChange {
    /// Path relative to the repo root.
    pub path: String,
    /// Change status: `M`/`A`/`D`/`R`/`C`/`?`/`U`/etc.
    pub status: String,
    /// Original path (for renames).
    #[serde(default)]
    pub old_path: Option<String>,
    /// Number of inserted lines.
    #[serde(default)]
    pub insertions: usize,
    /// Number of deleted lines.
    #[serde(default)]
    pub deletions: usize,
}

/// Overall repository status summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    /// Current branch.
    pub branch: Option<String>,
    /// Upstream tracking branch (e.g. `origin/main`).
    pub upstream: Option<String>,
    /// Commits ahead of upstream.
    pub ahead: usize,
    /// Commits behind upstream.
    pub behind: usize,
    /// All file changes in the working tree (and untracked).
    pub files: Vec<FileChange>,
    /// Whether the working tree is clean.
    pub clean: bool,
    /// Optional commit hash of the current HEAD.
    pub head: Option<String>,
    /// Path of the repository root.
    pub repo_root: Option<PathBuf>,
}

impl Default for RepoStatus {
    fn default() -> Self {
        Self {
            branch: None,
            upstream: None,
            ahead: 0,
            behind: 0,
            files: Vec::new(),
            clean: true,
            head: None,
            repo_root: None,
        }
    }
}

/// Diff output between two refs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Diff {
    /// Ref A (older).
    pub from: String,
    /// Ref B (newer).
    pub to: String,
    /// Combined diff text.
    pub text: String,
    /// Number of files changed.
    pub files_changed: usize,
    /// Number of insertions.
    pub insertions: usize,
    /// Number of deletions.
    pub deletions: usize,
}

/// Git context for a single repository.
#[derive(Debug, Clone)]
pub struct GitContext {
    /// Path to the working directory (used to invoke `git`).
    cwd: PathBuf,
}

impl GitContext {
    /// Create a new git context rooted at `cwd`.
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: cwd.into() }
    }

    /// The current working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Whether the given directory is inside a git repository.
    pub async fn is_repo(&self) -> bool {
        self.run(&["rev-parse", "--git-dir"]).await.is_ok()
    }

    /// Return the absolute path of the repository root.
    pub async fn repo_root(&self) -> LlmResult<PathBuf> {
        let out = self.run(&["rev-parse", "--show-toplevel"]).await?;
        Ok(PathBuf::from(out.trim()))
    }

    /// Return the current branch name.
    pub async fn current_branch(&self) -> LlmResult<String> {
        let out = self.run(&["rev-parse", "--abbrev-ref", "HEAD"]).await?;
        Ok(out.trim().to_string())
    }

    /// Return the current HEAD commit hash.
    pub async fn head(&self) -> LlmResult<String> {
        let out = self.run(&["rev-parse", "HEAD"]).await?;
        Ok(out.trim().to_string())
    }

    /// Return the upstream tracking branch, if any.
    pub async fn upstream(&self) -> Option<String> {
        let out = self
            .run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .await
            .ok()?;
        let trimmed = out.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Get the count of commits ahead/behind upstream.
    pub async fn ahead_behind(&self) -> LlmResult<(usize, usize)> {
        let Some(upstream) = self.upstream().await else {
            return Ok((0, 0));
        };
        let out = self
            .run(&[
                "rev-list",
                "--left-right",
                "--count",
                &format!("{upstream}...HEAD"),
            ])
            .await?;
        let mut parts = out.split_whitespace();
        let behind: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let ahead: usize = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        Ok((ahead, behind))
    }

    /// List the most recent `n` commits.
    pub async fn recent_commits(&self, n: usize) -> LlmResult<Vec<Commit>> {
        let n_str = n.to_string();
        let sep = "---END-OF-COMMIT---";
        let format = format!("%H%x00%h%x00%an%x00%ae%x00%aI%x00%s%x00%b{sep}");
        let out = self
            .run_with_format(&[
                "log",
                &format!("-n{n_str}"),
                &format!("--pretty=format:{format}"),
            ])
            .await?;
        let mut commits = Vec::new();
        for chunk in out.split(sep).filter(|s| !s.trim().is_empty()) {
            let trimmed = chunk.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.split('\0').collect();
            if parts.len() < 6 {
                continue;
            }
            commits.push(Commit {
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                author: parts[2].to_string(),
                email: parts[3].to_string(),
                date: parts[4].to_string(),
                subject: parts[5].to_string(),
                body: if parts.len() > 6 {
                    parts[6..].join("\0").trim().to_string()
                } else {
                    String::new()
                },
            });
        }
        Ok(commits)
    }

    /// Get the current repository status (branch, ahead/behind, file changes).
    pub async fn status(&self) -> LlmResult<RepoStatus> {
        let mut s = RepoStatus::default();

        if !self.is_repo().await {
            return Ok(s);
        }

        s.branch = self.current_branch().await.ok();
        s.head = self.head().await.ok();
        s.upstream = self.upstream().await;
        s.repo_root = self.repo_root().await.ok();

        if let Ok((ahead, behind)) = self.ahead_behind().await {
            s.ahead = ahead;
            s.behind = behind;
        }

        // Get porcelain status
        let porcelain = self
            .run(&["status", "--porcelain", "--untracked-files=all"])
            .await
            .unwrap_or_default();

        for line in porcelain.lines() {
            if line.len() < 4 {
                continue;
            }
            let code = &line[..2];
            let path_part = &line[3..];
            let (old, new) = if let Some(idx) = path_part.find(" -> ") {
                (
                    Some(path_part[..idx].to_string()),
                    path_part[idx + 4..].to_string(),
                )
            } else {
                (None, path_part.to_string())
            };
            s.files.push(FileChange {
                path: new,
                status: code.to_string(),
                old_path: old,
                insertions: 0,
                deletions: 0,
            });
        }

        s.clean = s.files.is_empty();
        Ok(s)
    }

    /// Get a `numstat` summary for changed files in the working tree.
    pub async fn numstat(&self) -> LlmResult<Vec<(String, usize, usize)>> {
        let out = self
            .run(&["diff", "--numstat", "--no-renames"])
            .await
            .unwrap_or_default();
        let mut result = Vec::new();
        for line in out.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 3 {
                continue;
            }
            let ins: usize = parts[0].parse().unwrap_or(0);
            let del: usize = parts[1].parse().unwrap_or(0);
            result.push((parts[2].to_string(), ins, del));
        }
        Ok(result)
    }

    /// Get a diff between two refs.
    pub async fn diff(&self, from: &str, to: &str) -> LlmResult<Diff> {
        let text = self
            .run(&["diff", "--no-color", from, to])
            .await
            .unwrap_or_default();
        let numstat = self
            .run(&["diff", "--numstat", "--no-renames", from, to])
            .await
            .unwrap_or_default();
        let mut files_changed = 0usize;
        let mut insertions = 0usize;
        let mut deletions = 0usize;
        for line in numstat.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 3 {
                continue;
            }
            files_changed += 1;
            insertions += parts[0].parse::<usize>().unwrap_or(0);
            deletions += parts[1].parse::<usize>().unwrap_or(0);
        }
        Ok(Diff {
            from: from.into(),
            to: to.into(),
            text,
            files_changed,
            insertions,
            deletions,
        })
    }

    /// Get the diff for a single file.
    pub async fn diff_file(&self, from: &str, to: &str, file: &str) -> LlmResult<String> {
        self.run(&["diff", "--no-color", from, to, "--", file])
            .await
    }

    /// Show the content of a file at a given ref.
    pub async fn show_file(&self, ref_name: &str, file: &str) -> LlmResult<String> {
        self.run(&["show", &format!("{ref_name}:{file}")]).await
    }

    /// List file paths tracked by git.
    pub async fn list_files(&self) -> LlmResult<Vec<String>> {
        let out = self.run(&["ls-files"]).await?;
        Ok(out.lines().map(|s| s.to_string()).collect())
    }

    /// Detect merge / rebase / cherry-pick in progress.
    pub async fn is_in_progress(&self) -> bool {
        self.run(&["rev-parse", "--is-inside-work-tree"])
            .await
            .is_ok()
            && (self.path_exists(&self.cwd.join(".git/MERGE_HEAD")).await
                || self.path_exists(&self.cwd.join(".git/REBASE_HEAD")).await
                || self
                    .path_exists(&self.cwd.join(".git/CHERRY_PICK_HEAD"))
                    .await)
    }

    async fn path_exists(&self, p: &Path) -> bool {
        tokio::fs::try_exists(p).await.unwrap_or(false)
    }

    /// Format a concise summary string for LLM prompt context.
    pub async fn summary(&self) -> String {
        if !self.is_repo().await {
            return String::from("Not a git repository.");
        }
        let Ok(status) = self.status().await else {
            return String::from("Unable to read git status.");
        };
        let mut out = String::new();
        if let Some(branch) = &status.branch {
            out.push_str(&format!("Branch: {branch}\n"));
        }
        if let Some(head) = &status.head {
            out.push_str(&format!("HEAD: {}\n", &head[..head.len().min(7)]));
        }
        if let Some(up) = &status.upstream {
            out.push_str(&format!(
                "Tracking: {up} (ahead {}, behind {})\n",
                status.ahead, status.behind
            ));
        }
        if status.clean {
            out.push_str("Working tree: clean\n");
        } else {
            out.push_str(&format!(
                "Working tree: {} file(s) changed\n",
                status.files.len()
            ));
        }
        out
    }

    async fn run(&self, args: &[&str]) -> LlmResult<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LlmError::Internal(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn run_with_format(&self, args: &[&str]) -> LlmResult<String> {
        // The format string contains % which is fine to pass as a single arg.
        self.run(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn git_context_creation() {
        let ctx = GitContext::new(".");
        assert_eq!(ctx.cwd(), Path::new("."));
    }

    #[tokio::test]
    async fn non_git_dir_is_not_a_repo() {
        // Use a temp directory to ensure it's not a git repo.
        let tmp = std::env::temp_dir().join(format!("opencode-llm-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let ctx = GitContext::new(&tmp);
        assert!(!ctx.is_repo().await);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn commit_struct_construction() {
        let c = Commit {
            hash: "abcdef1234567890".into(),
            short_hash: "abcdef1".into(),
            author: "Alice".into(),
            email: "alice@example.com".into(),
            date: "2026-07-15T10:00:00Z".into(),
            subject: "Test commit".into(),
            body: "Body line".into(),
        };
        assert_eq!(c.short_hash.len(), 7);
        assert_eq!(c.subject, "Test commit");
    }

    #[tokio::test]
    async fn file_change_status_parse() {
        let f = FileChange {
            path: "src/lib.rs".into(),
            status: "M ".into(),
            old_path: None,
            insertions: 5,
            deletions: 3,
        };
        assert_eq!(f.status, "M ");
        assert_eq!(f.path, "src/lib.rs");
    }

    #[tokio::test]
    async fn repo_status_defaults() {
        let s = RepoStatus::default();
        assert!(s.branch.is_none());
        assert!(s.clean);
        assert!(s.files.is_empty());
        assert_eq!(s.ahead, 0);
        assert_eq!(s.behind, 0);
    }

    #[tokio::test]
    async fn diff_struct_construction() {
        let d = Diff {
            from: "main".into(),
            to: "HEAD".into(),
            text: "diff --git ...".into(),
            files_changed: 1,
            insertions: 5,
            deletions: 2,
        };
        assert_eq!(d.from, "main");
        assert_eq!(d.insertions, 5);
    }

    #[tokio::test]
    async fn file_change_rename_parsing() {
        // Simulating a rename line.
        let line = "R  old/path.rs -> new/path.rs";
        let path_part = &line[3..];
        let (old, new) = if let Some(idx) = path_part.find(" -> ") {
            (
                Some(path_part[..idx].to_string()),
                path_part[idx + 4..].to_string(),
            )
        } else {
            (None, path_part.to_string())
        };
        assert_eq!(old, Some("old/path.rs".to_string()));
        assert_eq!(new, "new/path.rs");
    }

    #[tokio::test]
    async fn summary_returns_string_for_non_git() {
        let tmp = std::env::temp_dir().join(format!("opencode-llm-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let ctx = GitContext::new(&tmp);
        let summary = ctx.summary().await;
        assert!(summary.contains("Not a git repository"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn upstream_none_for_fresh_repo() {
        // Make a fresh git repo with no commits.
        let tmp = std::env::temp_dir().join(format!("opencode-llm-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let _ = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&tmp)
            .output()
            .await;
        let ctx = GitContext::new(&tmp);
        // No upstream configured.
        assert!(ctx.upstream().await.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn list_files_in_empty_repo() {
        let tmp = std::env::temp_dir().join(format!("opencode-llm-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let _ = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&tmp)
            .output()
            .await;
        let ctx = GitContext::new(&tmp);
        let files = ctx.list_files().await.unwrap();
        assert!(files.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn ahead_behind_returns_zero_for_fresh_repo() {
        let tmp = std::env::temp_dir().join(format!("opencode-llm-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        let _ = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&tmp)
            .output()
            .await;
        let ctx = GitContext::new(&tmp);
        let (ahead, behind) = ctx.ahead_behind().await.unwrap();
        assert_eq!(ahead, 0);
        assert_eq!(behind, 0);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
