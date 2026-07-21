//! Session persistence — JSONL-based storage for conversation history.
//!
//! Each session is stored as a JSONL file (one JSON object per line).
//! The format is compatible with the TUI's save/load feature.

use std::path::{Path, PathBuf};

use crate::error::{LlmError, LlmResult};
use serde::{Deserialize, Serialize};

/// A single entry in the session log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    /// Message role: "user", "assistant", "system", "error".
    pub role: String,
    /// Message text content.
    pub text: String,
    /// Optional timestamp (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl SessionEntry {
    /// Create a new session entry.
    pub fn new(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            text: text.into(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            metadata: None,
        }
    }

    /// Create a user message entry.
    pub fn user(text: impl Into<String>) -> Self {
        Self::new("user", text)
    }

    /// Create an assistant message entry.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new("assistant", text)
    }

    /// Create a system message entry.
    pub fn system(text: impl Into<String>) -> Self {
        Self::new("system", text)
    }
}

/// Session storage manager.
pub struct SessionStorage {
    /// Directory where session files are stored.
    base_dir: PathBuf,
    /// Current session file name (without extension).
    session_name: String,
}

impl SessionStorage {
    /// Create a new session storage in the given directory.
    ///
    /// The directory is created if it doesn't exist.
    pub fn new(base_dir: impl Into<PathBuf>) -> LlmResult<Self> {
        let base_dir = base_dir.into();
        std::fs::create_dir_all(&base_dir).map_err(LlmError::Io)?;
        Ok(Self {
            base_dir,
            session_name: "session".to_string(),
        })
    }

    /// Set the session name (used as the filename stem).
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.session_name = name.into();
        self
    }

    /// Get the full path to the session file.
    fn session_path(&self) -> PathBuf {
        self.base_dir.join(format!("{}.jsonl", self.session_name))
    }

    /// Append a single entry to the session file.
    pub fn append(&self, entry: &SessionEntry) -> LlmResult<()> {
        let path = self.session_path();
        let line = serde_json::to_string(entry).map_err(LlmError::Json)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(LlmError::Io)?;
        use std::io::Write;
        writeln!(file, "{line}").map_err(LlmError::Io)?;
        Ok(())
    }

    /// Read all entries from the session file.
    ///
    /// Returns an empty vec if the file doesn't exist.
    pub fn read_all(&self) -> LlmResult<Vec<SessionEntry>> {
        let path = self.session_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path).map_err(LlmError::Io)?;
        let mut entries = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: SessionEntry = serde_json::from_str(trimmed).map_err(LlmError::Json)?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Overwrite the session file with the given entries.
    pub fn write_all(&self, entries: &[SessionEntry]) -> LlmResult<()> {
        let path = self.session_path();
        let mut file = std::fs::File::create(&path).map_err(LlmError::Io)?;
        use std::io::Write;
        for entry in entries {
            let line = serde_json::to_string(entry).map_err(LlmError::Json)?;
            writeln!(file, "{line}").map_err(LlmError::Io)?;
        }
        Ok(())
    }

    /// Delete the session file.
    pub fn delete(&self) -> LlmResult<()> {
        let path = self.session_path();
        if path.exists() {
            std::fs::remove_file(&path).map_err(LlmError::Io)?;
        }
        Ok(())
    }

    /// List all session files in the storage directory.
    pub fn list_sessions(&self) -> LlmResult<Vec<String>> {
        let mut sessions = Vec::new();
        for entry in std::fs::read_dir(&self.base_dir).map_err(LlmError::Io)? {
            let entry = entry.map_err(LlmError::Io)?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                if let Some(stem) = path.file_stem() {
                    sessions.push(stem.to_string_lossy().to_string());
                }
            }
        }
        sessions.sort();
        Ok(sessions)
    }

    /// Get the current session name.
    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    /// Get the base directory.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Default session directory name.
pub const DEFAULT_SESSION_DIR: &str = ".opencode_sessions";

/// Create a default session storage in the user's home or current directory.
pub fn default_session_storage() -> LlmResult<SessionStorage> {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode")
        .join("sessions");
    SessionStorage::new(base)
}

/// Load a session from a JSONL file path.
pub fn load_session_file(path: impl AsRef<Path>) -> LlmResult<Vec<SessionEntry>> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(LlmError::Io)?;
    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: SessionEntry = serde_json::from_str(trimmed).map_err(LlmError::Json)?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Save entries to a JSONL file path (overwrites).
pub fn save_session_file(path: impl AsRef<Path>, entries: &[SessionEntry]) -> LlmResult<()> {
    use std::io::Write;
    let path = path.as_ref();
    let mut file = std::fs::File::create(path).map_err(LlmError::Io)?;
    for entry in entries {
        let line = serde_json::to_string(entry).map_err(LlmError::Json)?;
        writeln!(file, "{line}").map_err(LlmError::Io)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_read_session() {
        let dir = std::env::temp_dir().join(format!("opencode_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let storage = SessionStorage::new(&dir).unwrap();
        storage.append(&SessionEntry::user("Hello")).unwrap();
        storage
            .append(&SessionEntry::assistant("Hi there!"))
            .unwrap();
        storage
            .append(&SessionEntry::system("System message"))
            .unwrap();

        let entries = storage.read_all().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].role, "user");
        assert_eq!(entries[0].text, "Hello");
        assert_eq!(entries[1].role, "assistant");
        assert_eq!(entries[1].text, "Hi there!");
        assert_eq!(entries[2].role, "system");
        assert_eq!(entries[2].text, "System message");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_all_overwrites() {
        let dir = std::env::temp_dir().join(format!("opencode_test2_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let storage = SessionStorage::new(&dir).unwrap();
        storage.append(&SessionEntry::user("Old")).unwrap();
        storage.write_all(&[SessionEntry::user("New")]).unwrap();

        let entries = storage.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "New");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_sessions_returns_names() {
        let dir = std::env::temp_dir().join(format!("opencode_test3_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let s1 = SessionStorage::new(&dir).unwrap().with_name("alpha");
        let s2 = SessionStorage::new(&dir).unwrap().with_name("beta");
        s1.append(&SessionEntry::user("a")).unwrap();
        s2.append(&SessionEntry::user("b")).unwrap();

        let sessions = s1.list_sessions().unwrap();
        assert!(sessions.contains(&"alpha".to_string()));
        assert!(sessions.contains(&"beta".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_nonexistent_returns_empty() {
        let dir = std::env::temp_dir().join(format!("opencode_test4_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let storage = SessionStorage::new(&dir).unwrap();
        let entries = storage.read_all().unwrap();
        assert!(entries.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_removes_file() {
        let dir = std::env::temp_dir().join(format!("opencode_test5_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let storage = SessionStorage::new(&dir).unwrap();
        storage.append(&SessionEntry::user("test")).unwrap();
        assert!(storage.session_path().exists());

        storage.delete().unwrap();
        assert!(!storage.session_path().exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_save_file_functions() {
        let dir = std::env::temp_dir().join(format!("opencode_test6_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.jsonl");
        let entries = vec![
            SessionEntry::user("Hello"),
            SessionEntry::assistant("World"),
        ];
        save_session_file(&path, &entries).unwrap();
        let loaded = load_session_file(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].text, "Hello");
        assert_eq!(loaded[1].text, "World");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
