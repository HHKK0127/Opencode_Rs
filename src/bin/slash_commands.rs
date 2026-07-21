#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_assignments,
    clippy::all
)]

//! Slash command system for the TUI.
//!
//! Parses `/command` prefixes from user input and dispatches them to
//! registered handlers before the message is sent to the LLM.
//!
//! RPC-style dispatch: each command produces a typed `SlashAction` that the
//! application handles, instead of relying on magic string prefixes.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SlashAction — typed command dispatch (RPC)
// ---------------------------------------------------------------------------

/// Typed actions produced by slash commands. The application matches on these
/// instead of parsing magic strings.
#[derive(Debug, Clone)]
pub enum SlashAction {
    /// Show help text.
    Help,
    /// Clear chat history.
    Clear,
    /// Exit the application.
    Exit,
    /// Change the active model.
    SetModel { model: String },
    /// Open the configuration screen.
    OpenConfig,
    /// List available tools.
    ListTools,
    /// Save session to file.
    SaveSession { filename: String },
    /// Load session from file.
    LoadSession { filename: String },
    /// Show permission policies.
    ShowPermissions,
    /// Show session status.
    ShowStatus,
    /// Show session files.
    ShowFiles,
    /// Compact session (clear old messages).
    Compact,
    /// Unknown command.
    Unknown { name: String },
}

// ---------------------------------------------------------------------------
// SlashCommand — command metadata
// ---------------------------------------------------------------------------

/// A registered slash command.
#[derive(Clone)]
pub struct SlashCommand {
    /// Command name (without `/`).
    pub name: &'static str,
    /// Short description for `/help`.
    pub description: &'static str,
    /// Usage hint.
    pub usage: &'static str,
}

/// Built-in slash commands.
pub const BUILTIN_COMMANDS: &[SlashCommand] = &[
    SlashCommand {
        name: "help",
        description: "Show this help message",
        usage: "/help",
    },
    SlashCommand {
        name: "clear",
        description: "Clear chat history",
        usage: "/clear",
    },
    SlashCommand {
        name: "model",
        description: "Change the active model",
        usage: "/model <name>",
    },
    SlashCommand {
        name: "config",
        description: "Open configuration screen",
        usage: "/config",
    },
    SlashCommand {
        name: "tools",
        description: "List available tools",
        usage: "/tools",
    },
    SlashCommand {
        name: "save",
        description: "Save session to file",
        usage: "/save [filename]",
    },
    SlashCommand {
        name: "load",
        description: "Load session from file",
        usage: "/load <filename>",
    },
    SlashCommand {
        name: "permissions",
        description: "Show permission policies",
        usage: "/permissions",
    },
    SlashCommand {
        name: "status",
        description: "Show session status",
        usage: "/status",
    },
    SlashCommand {
        name: "files",
        description: "Show session files",
        usage: "/files",
    },
    SlashCommand {
        name: "compact",
        description: "Compact session (clear old messages)",
        usage: "/compact",
    },
    SlashCommand {
        name: "exit",
        description: "Exit the application",
        usage: "/exit",
    },
];

// ---------------------------------------------------------------------------
// SlashCommandResult — backward-compatible result
// ---------------------------------------------------------------------------

/// Result of parsing a slash command.
#[derive(Debug)]
pub enum SlashCommandResult {
    /// The input was a slash command; handle it (no LLM call).
    Handled {
        /// Response text to show the user.
        response: String,
    },
    /// The input was not a slash command; send to LLM.
    Passthrough {
        /// The original (or cleaned) message text.
        text: String,
    },
    /// Exit the application.
    Exit,
}

// ---------------------------------------------------------------------------
// SlashCommandDispatcher — RPC dispatcher
// ---------------------------------------------------------------------------

/// Dispatches slash commands to typed actions.
pub struct SlashCommandDispatcher {
    /// Registry of command handlers (name → handler).
    handlers: HashMap<String, fn(&str) -> SlashAction>,
}

impl SlashCommandDispatcher {
    /// Create a new dispatcher with all built-in commands.
    pub fn new() -> Self {
        let mut dispatcher = Self {
            handlers: HashMap::new(),
        };
        dispatcher.register_builtins();
        dispatcher
    }

    /// Register all built-in slash commands.
    fn register_builtins(&mut self) {
        self.register("help", |_| SlashAction::Help);
        self.register("clear", |_| SlashAction::Clear);
        self.register("exit", |_| SlashAction::Exit);
        self.register("quit", |_| SlashAction::Exit);
        self.register("model", |args| {
            if args.is_empty() {
                SlashAction::Help
            } else {
                SlashAction::SetModel {
                    model: args.to_string(),
                }
            }
        });
        self.register("config", |_| SlashAction::OpenConfig);
        self.register("tools", |_| SlashAction::ListTools);
        self.register("save", |args| {
            let filename = if args.is_empty() {
                "session.jsonl".to_string()
            } else {
                args.to_string()
            };
            SlashAction::SaveSession { filename }
        });
        self.register("load", |args| {
            if args.is_empty() {
                SlashAction::Unknown {
                    name: "load requires a filename".to_string(),
                }
            } else {
                SlashAction::LoadSession {
                    filename: args.to_string(),
                }
            }
        });
        self.register("permissions", |_| SlashAction::ShowPermissions);
        self.register("status", |_| SlashAction::ShowStatus);
        self.register("files", |_| SlashAction::ShowFiles);
        self.register("compact", |_| SlashAction::Compact);
    }

    /// Register a custom slash command.
    pub fn register(&mut self, name: &str, handler: fn(&str) -> SlashAction) {
        self.handlers.insert(name.to_string(), handler);
    }

    /// Dispatch a user input string.
    /// Returns `None` if the input is not a slash command.
    pub fn dispatch(&self, input: &str) -> Option<SlashAction> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let rest = &trimmed[1..];
        let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
        let cmd_name = parts[0].to_lowercase();
        let args = parts.get(1).copied().unwrap_or("").trim();

        if let Some(handler) = self.handlers.get(&cmd_name) {
            Some(handler(args))
        } else {
            Some(SlashAction::Unknown { name: cmd_name })
        }
    }

    /// Dispatch and produce a backward-compatible `SlashCommandResult`.
    pub fn dispatch_to_result(&self, input: &str) -> SlashCommandResult {
        match self.dispatch(input) {
            Some(action) => match action {
                SlashAction::Help => {
                    let mut help = String::from("Available commands:\n");
                    for cmd in BUILTIN_COMMANDS {
                        help.push_str(&format!("  {:<20} {}\n", cmd.usage, cmd.description));
                    }
                    SlashCommandResult::Handled { response: help }
                }
                SlashAction::Clear => SlashCommandResult::Handled {
                    response: "__CLEAR_HISTORY__".to_string(),
                },
                SlashAction::Exit => SlashCommandResult::Exit,
                SlashAction::SetModel { model } => SlashCommandResult::Handled {
                    response: format!("__SET_MODEL__:{model}"),
                },
                SlashAction::OpenConfig => SlashCommandResult::Handled {
                    response: "__OPEN_CONFIG__".to_string(),
                },
                SlashAction::ListTools => SlashCommandResult::Handled {
                    response: "__LIST_TOOLS__".to_string(),
                },
                SlashAction::SaveSession { filename } => SlashCommandResult::Handled {
                    response: format!("__SAVE_SESSION__:{filename}"),
                },
                SlashAction::LoadSession { filename } => SlashCommandResult::Handled {
                    response: format!("__LOAD_SESSION__:{filename}"),
                },
                SlashAction::ShowPermissions => SlashCommandResult::Handled {
                    response: "__SHOW_PERMISSIONS__".to_string(),
                },
                SlashAction::ShowStatus => SlashCommandResult::Handled {
                    response: "__SHOW_STATUS__".to_string(),
                },
                SlashAction::ShowFiles => SlashCommandResult::Handled {
                    response: "__SHOW_FILES__".to_string(),
                },
                SlashAction::Compact => SlashCommandResult::Handled {
                    response: "__COMPACT_SESSION__".to_string(),
                },
                SlashAction::Unknown { name } => SlashCommandResult::Handled {
                    response: format!(
                        "Unknown command: `/{name}`\nType `/help` for available commands."
                    ),
                },
            },
            None => SlashCommandResult::Passthrough {
                text: input.trim().to_string(),
            },
        }
    }
}

impl Default for SlashCommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Legacy API (backward compatible)
// ---------------------------------------------------------------------------

/// Parse a user input line and determine if it's a slash command.
/// Uses the dispatcher internally for consistency.
pub fn parse_slash_command(input: &str) -> SlashCommandResult {
    let dispatcher = SlashCommandDispatcher::new();
    dispatcher.dispatch_to_result(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_normal_input() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("hello world");
        match result {
            SlashCommandResult::Passthrough { text } => assert_eq!(text, "hello world"),
            _ => panic!("expected passthrough"),
        }
    }

    #[test]
    fn help_command() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/help");
        match result {
            SlashCommandResult::Handled { response } => {
                assert!(response.contains("Available commands"));
                assert!(response.contains("/help"));
            }
            _ => panic!("expected handled"),
        }
    }

    #[test]
    fn clear_command() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/clear");
        match result {
            SlashCommandResult::Handled { response } => {
                assert_eq!(response, "__CLEAR_HISTORY__");
            }
            _ => panic!("expected handled"),
        }
    }

    #[test]
    fn model_command() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/model claude-4");
        match result {
            SlashCommandResult::Handled { response } => {
                assert_eq!(response, "__SET_MODEL__:claude-4");
            }
            _ => panic!("expected handled"),
        }
    }

    #[test]
    fn model_command_no_args() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/model");
        match result {
            SlashCommandResult::Handled { response } => {
                assert!(response.contains("Available commands"));
            }
            _ => panic!("expected handled"),
        }
    }

    #[test]
    fn exit_command() {
        let dispatcher = SlashCommandDispatcher::new();
        match dispatcher.dispatch_to_result("/exit") {
            SlashCommandResult::Exit => {}
            _ => panic!("expected exit"),
        }
        match dispatcher.dispatch_to_result("/quit") {
            SlashCommandResult::Exit => {}
            _ => panic!("expected exit"),
        }
    }

    #[test]
    fn unknown_command() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/foobar");
        match result {
            SlashCommandResult::Handled { response } => {
                assert!(response.contains("Unknown command"));
            }
            _ => panic!("expected handled"),
        }
    }

    #[test]
    fn tool_command() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/tools");
        match result {
            SlashCommandResult::Handled { response } => {
                assert_eq!(response, "__LIST_TOOLS__");
            }
            _ => panic!("expected handled"),
        }
    }

    #[test]
    fn status_command() {
        let dispatcher = SlashCommandDispatcher::new();
        let result = dispatcher.dispatch_to_result("/status");
        match result {
            SlashCommandResult::Handled { response } => {
                assert_eq!(response, "__SHOW_STATUS__");
            }
            _ => panic!("expected handled"),
        }
    }

    // --- RPC dispatcher tests ---

    #[test]
    fn rpc_dispatch_clear() {
        let dispatcher = SlashCommandDispatcher::new();
        let action = dispatcher.dispatch("/clear");
        assert!(matches!(action, Some(SlashAction::Clear)));
    }

    #[test]
    fn rpc_dispatch_set_model() {
        let dispatcher = SlashCommandDispatcher::new();
        let action = dispatcher.dispatch("/model gpt-4o");
        match action {
            Some(SlashAction::SetModel { model }) => assert_eq!(model, "gpt-4o"),
            _ => panic!("expected SetModel"),
        }
    }

    #[test]
    fn rpc_dispatch_exit() {
        let dispatcher = SlashCommandDispatcher::new();
        assert!(matches!(
            dispatcher.dispatch("/exit"),
            Some(SlashAction::Exit)
        ));
        assert!(matches!(
            dispatcher.dispatch("/quit"),
            Some(SlashAction::Exit)
        ));
    }

    #[test]
    fn rpc_dispatch_none_for_non_command() {
        let dispatcher = SlashCommandDispatcher::new();
        assert!(dispatcher.dispatch("hello").is_none());
    }

    #[test]
    fn rpc_dispatch_unknown() {
        let dispatcher = SlashCommandDispatcher::new();
        match dispatcher.dispatch("/foobar") {
            Some(SlashAction::Unknown { name }) => assert_eq!(name, "foobar"),
            _ => panic!("expected Unknown"),
        }
    }

    #[test]
    fn rpc_dispatch_save_with_default() {
        let dispatcher = SlashCommandDispatcher::new();
        match dispatcher.dispatch("/save") {
            Some(SlashAction::SaveSession { filename }) => assert_eq!(filename, "session.jsonl"),
            _ => panic!("expected SaveSession with default"),
        }
    }

    #[test]
    fn rpc_dispatch_save_with_filename() {
        let dispatcher = SlashCommandDispatcher::new();
        match dispatcher.dispatch("/save my-session.jsonl") {
            Some(SlashAction::SaveSession { filename }) => assert_eq!(filename, "my-session.jsonl"),
            _ => panic!("expected SaveSession with custom filename"),
        }
    }

    #[test]
    fn rpc_dispatch_compact() {
        let dispatcher = SlashCommandDispatcher::new();
        assert!(matches!(
            dispatcher.dispatch("/compact"),
            Some(SlashAction::Compact)
        ));
    }

    #[test]
    fn custom_command_registration() {
        let mut dispatcher = SlashCommandDispatcher::new();
        dispatcher.register("test", |args| SlashAction::Unknown {
            name: format!("test:{}", args),
        });
        match dispatcher.dispatch("/test hello") {
            Some(SlashAction::Unknown { name }) => assert_eq!(name, "test:hello"),
            _ => panic!("expected custom command"),
        }
    }
}
