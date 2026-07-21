//! Permission enforcer — controls which tools and operations are allowed.
//!
//! The [`PermissionEnforcer`] evaluates requests against a set of
//! [`Policy`] rules and returns a [`PermissionLevel`] decision.
//! Interactive prompts ("ask user") can be resolved via the TUI callback.

/// What level of permission a policy grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    /// Always allowed without confirmation.
    AlwaysAllow,
    /// Ask the user for confirmation.
    AskUser,
    /// Always denied.
    AlwaysDeny,
}

/// A policy rule that matches tool or resource names by prefix or exact name.
#[derive(Debug, Clone)]
pub struct Policy {
    /// Human-readable label.
    pub label: String,
    /// Glob/prefix pattern (e.g. `bash`, `mcp__fs__*`, `read_*`).
    pub pattern: String,
    /// The permission level.
    pub level: PermissionLevel,
}

/// Decision returned by the enforcer.
#[derive(Debug, Clone)]
pub enum PermissionDecision {
    /// Allowed (possibly with a reason).
    Allowed {
        /// The matching policy label.
        policy: String,
    },
    /// Needs user confirmation.
    NeedsConfirmation {
        /// Tool or resource name.
        target: String,
        /// Reason or description.
        description: String,
    },
    /// Denied with a reason.
    Denied {
        /// Reason for denial.
        reason: String,
    },
}

/// Callback type for interactive permission prompts.
/// Returns `true` if the user approved, `false` otherwise.
pub type ConfirmationCallback = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// Permission enforcer.
pub struct PermissionEnforcer {
    policies: Vec<Policy>,
    callback: Option<ConfirmationCallback>,
}

impl Default for PermissionEnforcer {
    fn default() -> Self {
        Self {
            policies: vec![
                Policy {
                    label: "Read tools".to_string(),
                    pattern: "read_*".to_string(),
                    level: PermissionLevel::AlwaysAllow,
                },
                Policy {
                    label: "Glob/Grep search".to_string(),
                    pattern: "glob_search".to_string(),
                    level: PermissionLevel::AlwaysAllow,
                },
                Policy {
                    label: "Grep search".to_string(),
                    pattern: "grep_search".to_string(),
                    level: PermissionLevel::AlwaysAllow,
                },
                Policy {
                    label: "Web fetch".to_string(),
                    pattern: "web_fetch".to_string(),
                    level: PermissionLevel::AskUser,
                },
                Policy {
                    label: "Web search".to_string(),
                    pattern: "web_search".to_string(),
                    level: PermissionLevel::AskUser,
                },
                Policy {
                    label: "Write/Edit tools".to_string(),
                    pattern: "write_*".to_string(),
                    level: PermissionLevel::AskUser,
                },
                Policy {
                    label: "Edit file".to_string(),
                    pattern: "edit_file".to_string(),
                    level: PermissionLevel::AskUser,
                },
                Policy {
                    label: "Shell execution".to_string(),
                    pattern: "bash".to_string(),
                    level: PermissionLevel::AskUser,
                },
                Policy {
                    label: "MCP tools (default)".to_string(),
                    pattern: "mcp__*".to_string(),
                    level: PermissionLevel::AskUser,
                },
            ],
            callback: None,
        }
    }
}

impl PermissionEnforcer {
    /// Create a new enforcer with the given policies.
    pub fn new(policies: Vec<Policy>) -> Self {
        Self {
            policies,
            callback: None,
        }
    }

    /// Set a confirmation callback for interactive prompts.
    pub fn with_callback(mut self, cb: ConfirmationCallback) -> Self {
        self.callback = Some(cb);
        self
    }

    /// Evaluate a permission request for the given tool/resource name.
    pub fn evaluate(&self, target: &str, description: &str) -> PermissionDecision {
        // Find the most specific matching policy.
        let mut matched: Option<&Policy> = None;
        for policy in &self.policies {
            if wildcard_match(target, &policy.pattern) {
                matched = Some(policy);
            }
        }

        match matched {
            None => {
                // No matching policy → deny by default.
                PermissionDecision::Denied {
                    reason: format!("no policy matches `{target}`"),
                }
            }
            Some(policy) => match policy.level {
                PermissionLevel::AlwaysAllow => PermissionDecision::Allowed {
                    policy: policy.label.clone(),
                },
                PermissionLevel::AlwaysDeny => PermissionDecision::Denied {
                    reason: format!("denied by policy `{}`", policy.label),
                },
                PermissionLevel::AskUser => {
                    if let Some(ref cb) = self.callback {
                        if cb(target, description) {
                            PermissionDecision::Allowed {
                                policy: policy.label.clone(),
                            }
                        } else {
                            PermissionDecision::Denied {
                                reason: "denied by user".into(),
                            }
                        }
                    } else {
                        PermissionDecision::NeedsConfirmation {
                            target: target.to_string(),
                            description: description.to_string(),
                        }
                    }
                }
            },
        }
    }

    /// Add a policy.
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Remove all policies matching a pattern.
    pub fn remove_policies(&mut self, pattern: &str) {
        self.policies.retain(|p| p.pattern != pattern);
    }

    /// List all policies.
    pub fn policies(&self) -> &[Policy] {
        &self.policies
    }

    /// Clear all policies.
    pub fn clear(&mut self) {
        self.policies.clear();
    }

    /// Reset to default policies.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Simple wildcard match (`*` matches any sequence of characters).
fn wildcard_match(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        name.starts_with(prefix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else {
        name == pattern
    }
}

/// Configure the enforcer to always allow a set of tools.
pub fn allow_list(tools: &[&str]) -> Vec<Policy> {
    tools
        .iter()
        .map(|t| Policy {
            label: format!("Allow {t}"),
            pattern: t.to_string(),
            level: PermissionLevel::AlwaysAllow,
        })
        .collect()
}

/// Configure the enforcer to always deny a set of tools.
pub fn deny_list(tools: &[&str]) -> Vec<Policy> {
    tools
        .iter()
        .map(|t| Policy {
            label: format!("Deny {t}"),
            pattern: t.to_string(),
            level: PermissionLevel::AlwaysDeny,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_allows_read_tools() {
        let enf = PermissionEnforcer::default();
        let d = enf.evaluate("read_file", "read a file");
        assert!(matches!(d, PermissionDecision::Allowed { .. }));
    }

    #[test]
    fn default_allows_glob_search() {
        let enf = PermissionEnforcer::default();
        let d = enf.evaluate("glob_search", "search files");
        assert!(matches!(d, PermissionDecision::Allowed { .. }));
    }

    #[test]
    fn default_asks_for_bash() {
        let enf = PermissionEnforcer::default();
        let d = enf.evaluate("bash", "run a command");
        assert!(matches!(d, PermissionDecision::NeedsConfirmation { .. }));
    }

    #[test]
    fn deny_policy_blocks() {
        let enf = PermissionEnforcer::new(vec![Policy {
            label: "block all".to_string(),
            pattern: "*".to_string(),
            level: PermissionLevel::AlwaysDeny,
        }]);
        let d = enf.evaluate("anything", "");
        assert!(matches!(d, PermissionDecision::Denied { .. }));
    }

    #[test]
    fn callback_resolves_ask_user() {
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        let enf = PermissionEnforcer::default().with_callback(Box::new(move |_, _| {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            true
        }));
        let d = enf.evaluate("bash", "run");
        assert!(matches!(d, PermissionDecision::Allowed { .. }));
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn wildcard_prefix_match() {
        assert!(wildcard_match("read_file", "read_*"));
        assert!(wildcard_match("write_file", "write_*"));
        assert!(!wildcard_match("bash", "read_*"));
    }

    #[test]
    fn wildcard_suffix_match() {
        assert!(wildcard_match("mcp__fs__read", "mcp__*"));
        assert!(wildcard_match("mcp__fs__write", "mcp__*"));
        assert!(!wildcard_match("bash", "mcp__*"));
    }

    #[test]
    fn exact_match() {
        assert!(wildcard_match("bash", "bash"));
        assert!(!wildcard_match("bash_history", "bash"));
    }

    #[test]
    fn wildcard_star_matches_all() {
        assert!(wildcard_match("anything", "*"));
    }

    #[test]
    fn no_match_defaults_to_denied() {
        let enf = PermissionEnforcer::new(vec![]);
        let d = enf.evaluate("unknown_tool", "");
        assert!(matches!(d, PermissionDecision::Denied { .. }));
    }

    #[test]
    fn allow_list_creates_policies() {
        let policies = allow_list(&["bash", "read_file"]);
        assert_eq!(policies.len(), 2);
        let enf = PermissionEnforcer::new(policies);
        assert!(matches!(
            enf.evaluate("bash", ""),
            PermissionDecision::Allowed { .. }
        ));
        assert!(matches!(
            enf.evaluate("write_file", ""),
            PermissionDecision::Denied { .. }
        ));
    }

    #[test]
    fn deny_list_blocks() {
        let policies = deny_list(&["bash"]);
        let enf = PermissionEnforcer::new(policies);
        assert!(matches!(
            enf.evaluate("bash", ""),
            PermissionDecision::Denied { .. }
        ));
    }

    #[test]
    fn add_and_remove_policy() {
        let mut enf = PermissionEnforcer::default();
        enf.add_policy(Policy {
            label: "custom".to_string(),
            pattern: "custom_tool".to_string(),
            level: PermissionLevel::AlwaysAllow,
        });
        assert!(matches!(
            enf.evaluate("custom_tool", ""),
            PermissionDecision::Allowed { .. }
        ));
        enf.remove_policies("custom_tool");
        assert!(matches!(
            enf.evaluate("custom_tool", ""),
            PermissionDecision::Denied { .. }
        ));
    }

    #[test]
    fn reset_restores_defaults() {
        let mut enf = PermissionEnforcer::new(vec![]);
        enf.reset();
        assert!(matches!(
            enf.evaluate("read_file", ""),
            PermissionDecision::Allowed { .. }
        ));
    }
}
