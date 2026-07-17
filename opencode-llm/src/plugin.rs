//! Plugin system for opencode-llm.
//!
//! Allows external plugins to register tools, lifecycle hooks, and
//! configuration. Inspired by `claw-code`'s plugin architecture.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │  PluginManager                              │  ← load / unload / query
//! │  - load_plugin / unload_plugin              │
//! │  - all_tools / all_executors / all_hooks    │
//! └──────┬──────────────────────┬───────────────┘
//!  ┌─────▼──────┐       ┌──────▼───────┐
//!  │ NativePlugin│       │ WasmPlugin   │
//!  │ (Rust)     │       │ (WASM)       │
//!  └────────────┘       └──────────────┘
//! ```

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::LlmError;
use crate::tools::{ToolError, ToolExecutor, ToolSpec};

/// Result alias for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin-specific error type.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Plugin failed to load.
    #[error("plugin load failed: {0}")]
    LoadFailed(String),

    /// Plugin failed to initialize.
    #[error("plugin init failed: {0}")]
    InitFailed(String),

    /// Plugin with the same name is already loaded.
    #[error("plugin already loaded: {0}")]
    AlreadyLoaded(String),

    /// Plugin not found by name.
    #[error("plugin not found: {0}")]
    NotFound(String),

    /// Plugin execution error.
    #[error("plugin error: {0}")]
    Execution(String),

    /// I/O error during plugin loading.
    #[error("plugin io error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("plugin config error: {0}")]
    Config(String),
}

impl From<PluginError> for LlmError {
    fn from(e: PluginError) -> Self {
        LlmError::Internal(e.to_string())
    }
}

impl From<PluginError> for ToolError {
    fn from(e: PluginError) -> Self {
        ToolError::Other(e.to_string())
    }
}

/// Lifecycle events a plugin can hook into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginHook {
    /// Called when the runtime starts a new conversation.
    ConversationStart,
    /// Called when the runtime finishes a conversation.
    ConversationEnd,
    /// Called after every tool execution.
    AfterToolCall,
    /// Called before the model is invoked.
    BeforeModelInvoke,
    /// Called after the model responds.
    AfterModelResponse,
}

/// A plugin descriptor — metadata only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    /// Unique plugin name (e.g. `"my-plugin"`).
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Author information.
    pub author: Option<String>,
    /// Hooks this plugin subscribes to.
    pub hooks: Vec<PluginHook>,
    /// Whether the plugin is enabled by default.
    pub enabled_by_default: bool,
    /// Whether the plugin is required (cannot be disabled).
    pub required: bool,
}

/// Plugin configuration stored in the plugin's directory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// Whether the plugin is enabled.
    pub enabled: bool,
    /// Arbitrary key-value settings for the plugin.
    #[serde(default)]
    pub settings: BTreeMap<String, String>,
    /// Tool-specific configuration.
    #[serde(default)]
    pub tool_config: BTreeMap<String, serde_json::Value>,
}

/// The trait every plugin must implement.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Return the plugin's descriptor.
    fn descriptor(&self) -> PluginDescriptor;

    /// Initialize the plugin. Called once after loading.
    async fn initialize(&self, config: &PluginConfig) -> PluginResult<()>;

    /// Shut down the plugin. Called before unloading.
    async fn shutdown(&self) -> PluginResult<()> {
        Ok(())
    }

    /// Return the tools this plugin provides (spec + executor pairs).
    fn tools(&self) -> Vec<(ToolSpec, ToolExecutor)> {
        Vec::new()
    }

    /// Lifecycle hook: conversation started.
    async fn on_conversation_start(&self) -> PluginResult<()> {
        Ok(())
    }

    /// Lifecycle hook: conversation ended.
    async fn on_conversation_end(&self) -> PluginResult<()> {
        Ok(())
    }

    /// Lifecycle hook: after a tool call completes.
    async fn on_after_tool_call(&self, _tool_name: &str, _result: &str) -> PluginResult<()> {
        Ok(())
    }

    /// Lifecycle hook: before the model is invoked.
    async fn on_before_model_invoke(&self) -> PluginResult<()> {
        Ok(())
    }

    /// Lifecycle hook: after the model responds.
    async fn on_after_model_response(&self, _response: &str) -> PluginResult<()> {
        Ok(())
    }
}

/// A loaded plugin with its active configuration.
struct LoadedPlugin {
    plugin: Arc<dyn Plugin>,
    config: PluginConfig,
    descriptor: PluginDescriptor,
    initialized: bool,
}

/// Manages the lifecycle and registry of all plugins.
pub struct PluginManager {
    plugins: BTreeMap<String, LoadedPlugin>,
    hooks: HashMap<PluginHook, Vec<String>>, // hook → list of plugin names
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Create a new empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
            hooks: HashMap::new(),
        }
    }

    /// Register a plugin. The plugin's `initialize` is called immediately.
    pub async fn register(
        &mut self,
        plugin: Arc<dyn Plugin>,
        config: Option<PluginConfig>,
    ) -> PluginResult<()> {
        let desc = plugin.descriptor();
        let name = desc.name.clone();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name));
        }

        let config = config.unwrap_or_default();
        plugin.initialize(&config).await?;

        // Register hooks.
                for hook in &desc.hooks {
                    self.hooks
                        .entry(*hook)
                        .or_insert_with(Vec::new)
                        .push(name.clone());
                }

        self.plugins.insert(
            name,
            LoadedPlugin {
                plugin,
                config,
                descriptor: desc,
                initialized: true,
            },
        );

        Ok(())
    }

    /// Unload a plugin by name. Calls `shutdown` before removal.
    pub async fn unload(&mut self, name: &str) -> PluginResult<()> {
        let loaded = self
            .plugins
            .remove(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        loaded.plugin.shutdown().await?;

        // Remove hooks.
        for hook in &loaded.descriptor.hooks {
            if let Some(entry) = self.hooks.get_mut(hook) {
                entry.retain(|n| n != name);
            }
        }

        Ok(())
    }

    /// Check if a plugin is loaded.
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// List all loaded plugin descriptors.
    pub fn list_plugins(&self) -> Vec<PluginDescriptor> {
        self.plugins
            .values()
            .map(|p| p.descriptor.clone())
            .collect()
    }

    /// Get a plugin's config by name.
    pub fn plugin_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins.get(name).map(|p| &p.config)
    }

    /// Update a plugin's config.
    pub async fn update_config(
        &mut self,
        name: &str,
        config: PluginConfig,
    ) -> PluginResult<()> {
        let loaded = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        // Re-initialize with new config.
        loaded.plugin.initialize(&config).await?;
        loaded.config = config;
        Ok(())
    }

    /// Collect all tool specs from all loaded plugins.
    pub fn all_tool_specs(&self) -> Vec<ToolSpec> {
        self.plugins
            .values()
            .flat_map(|p| p.plugin.tools().into_iter().map(|(spec, _)| spec))
            .collect()
    }

    /// Collect all tool executors from all loaded plugins.
    pub fn all_tool_executors(&self) -> BTreeMap<String, ToolExecutor> {
        let mut map = BTreeMap::new();
        for loaded in self.plugins.values() {
            for (spec, executor) in loaded.plugin.tools() {
                map.insert(spec.name, executor);
            }
        }
        map
    }

    /// Fire a lifecycle hook for all plugins that subscribe to it.
    pub async fn fire_hook(&self, hook: PluginHook) {
        let Some(plugin_names) = self.hooks.get(&hook) else {
            return;
        };
        for name in plugin_names {
            if let Some(loaded) = self.plugins.get(name) {
                let result = match hook {
                    PluginHook::ConversationStart => loaded.plugin.on_conversation_start().await,
                    PluginHook::ConversationEnd => loaded.plugin.on_conversation_end().await,
                    _ => Ok(()),
                };
                if let Err(e) = result {
                    tracing::warn!("plugin hook failed for `{name}`: {e}");
                }
            }
        }
    }

    /// Fire the after-tool-call hook for all plugins that subscribe to it.
    pub async fn fire_after_tool_call(&self, tool_name: &str, result: &str) {
        let Some(plugin_names) = self.hooks.get(&PluginHook::AfterToolCall) else {
            return;
        };
        for name in plugin_names {
            if let Some(loaded) = self.plugins.get(name) {
                if let Err(e) = loaded.plugin.on_after_tool_call(tool_name, result).await {
                    tracing::warn!("plugin after-tool-call hook failed for `{name}`: {e}");
                }
            }
        }
    }

    /// Fire the before-model-invoke hook for all plugins that subscribe to it.
    pub async fn fire_before_model_invoke(&self) {
        let Some(plugin_names) = self.hooks.get(&PluginHook::BeforeModelInvoke) else {
            return;
        };
        for name in plugin_names {
            if let Some(loaded) = self.plugins.get(name) {
                if let Err(e) = loaded.plugin.on_before_model_invoke().await {
                    tracing::warn!("plugin before-model-invoke hook failed for `{name}`: {e}");
                }
            }
        }
    }

    /// Fire the after-model-response hook for all plugins that subscribe to it.
    pub async fn fire_after_model_response(&self, response: &str) {
        let Some(plugin_names) = self.hooks.get(&PluginHook::AfterModelResponse) else {
            return;
        };
        for name in plugin_names {
            if let Some(loaded) = self.plugins.get(name) {
                if let Err(e) = loaded.plugin.on_after_model_response(response).await {
                    tracing::warn!("plugin after-model-response hook failed for `{name}`: {e}");
                }
            }
        }
    }

    /// Number of loaded plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Whether the manager has no plugins loaded.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Built-in plugins
// ---------------------------------------------------------------------------

/// A no-op plugin that does nothing. Useful as a base for testing.
pub struct NoopPlugin {
    name: String,
    version: String,
    description: String,
}

impl NoopPlugin {
    /// Create a new no-op plugin.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

#[async_trait]
impl Plugin for NoopPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: None,
            hooks: Vec::new(),
            enabled_by_default: true,
            required: false,
        }
    }

    async fn initialize(&self, _config: &PluginConfig) -> PluginResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_list_plugins() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(NoopPlugin::new("test-plugin", "1.0.0"));
        mgr.register(plugin, None).await.unwrap();
        assert_eq!(mgr.len(), 1);
        let list = mgr.list_plugins();
        assert_eq!(list[0].name, "test-plugin");
        assert_eq!(list[0].version, "1.0.0");
    }

    #[tokio::test]
    async fn duplicate_plugin_name_fails() {
        let mut mgr = PluginManager::new();
        let p1 = Arc::new(NoopPlugin::new("dup", "1.0.0"));
        let p2 = Arc::new(NoopPlugin::new("dup", "2.0.0"));
        mgr.register(p1, None).await.unwrap();
        let err = mgr.register(p2, None).await.unwrap_err();
        assert!(matches!(err, PluginError::AlreadyLoaded(_)));
    }

    #[tokio::test]
    async fn unload_plugin_removes_it() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(NoopPlugin::new("to-unload", "1.0.0"));
        mgr.register(plugin, None).await.unwrap();
        assert!(mgr.is_loaded("to-unload"));
        mgr.unload("to-unload").await.unwrap();
        assert!(!mgr.is_loaded("to-unload"));
    }

    #[tokio::test]
    async fn unload_nonexistent_plugin_fails() {
        let mut mgr = PluginManager::new();
        let err = mgr.unload("nonexistent").await.unwrap_err();
        assert!(matches!(err, PluginError::NotFound(_)));
    }

    #[tokio::test]
    async fn plugin_tools_are_collected() {
        struct TestPlugin;
        #[async_trait]
        impl Plugin for TestPlugin {
            fn descriptor(&self) -> PluginDescriptor {
                PluginDescriptor {
                    name: "test-tool-plugin".into(),
                    version: "1.0.0".into(),
                    description: "test".into(),
                    author: None,
                    hooks: Vec::new(),
                    enabled_by_default: true,
                    required: false,
                }
            }
            async fn initialize(&self, _config: &PluginConfig) -> PluginResult<()> {
                Ok(())
            }
            fn tools(&self) -> Vec<(ToolSpec, ToolExecutor)> {
                let spec = ToolSpec::new("test_tool", "A test tool", serde_json::json!({
                    "type": "object",
                    "properties": { "input": { "type": "string" } }
                }));
                let executor: ToolExecutor = Arc::new(crate::tools::bash::BashTool);
                vec![(spec, executor)]
            }
        }

        let mut mgr = PluginManager::new();
        let plugin: Arc<dyn Plugin> = Arc::new(TestPlugin);
        mgr.register(plugin, None).await.unwrap();
        let specs = mgr.all_tool_specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "test_tool");
        let executors = mgr.all_tool_executors();
        assert!(executors.contains_key("test_tool"));
    }

    #[tokio::test]
    async fn lifecycle_hooks_are_fired() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct HookPlugin {
            start_called: AtomicBool,
            end_called: AtomicBool,
        }
        #[async_trait]
        impl Plugin for HookPlugin {
            fn descriptor(&self) -> PluginDescriptor {
                PluginDescriptor {
                    name: "hook-plugin".into(),
                    version: "1.0.0".into(),
                    description: "hooks".into(),
                    author: None,
                    hooks: vec![PluginHook::ConversationStart, PluginHook::ConversationEnd],
                    enabled_by_default: true,
                    required: false,
                }
            }
            async fn initialize(&self, _config: &PluginConfig) -> PluginResult<()> {
                Ok(())
            }
            async fn on_conversation_start(&self) -> PluginResult<()> {
                self.start_called.store(true, Ordering::SeqCst);
                Ok(())
            }
            async fn on_conversation_end(&self) -> PluginResult<()> {
                self.end_called.store(true, Ordering::SeqCst);
                Ok(())
            }
        }

        let plugin = Arc::new(HookPlugin {
            start_called: AtomicBool::new(false),
            end_called: AtomicBool::new(false),
        });

        let mut mgr = PluginManager::new();
        mgr.register(plugin.clone(), None).await.unwrap();

        mgr.fire_hook(PluginHook::ConversationStart).await;
        assert!(plugin.start_called.load(Ordering::SeqCst));

        mgr.fire_hook(PluginHook::ConversationEnd).await;
        assert!(plugin.end_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn update_config_reinitializes_plugin() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct ConfigPlugin {
            init_count: AtomicUsize,
        }
        #[async_trait]
        impl Plugin for ConfigPlugin {
            fn descriptor(&self) -> PluginDescriptor {
                PluginDescriptor {
                    name: "config-plugin".into(),
                    version: "1.0.0".into(),
                    description: "config".into(),
                    author: None,
                    hooks: Vec::new(),
                    enabled_by_default: true,
                    required: false,
                }
            }
            async fn initialize(&self, _config: &PluginConfig) -> PluginResult<()> {
                self.init_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let plugin = Arc::new(ConfigPlugin {
            init_count: AtomicUsize::new(0),
        });
        let mut mgr = PluginManager::new();
        mgr.register(plugin.clone(), None).await.unwrap();
        assert_eq!(plugin.init_count.load(Ordering::SeqCst), 1);

        mgr.update_config(
            "config-plugin",
            PluginConfig {
                enabled: true,
                settings: [("key".into(), "value".into())].into(),
                tool_config: BTreeMap::new(),
            },
        )
        .await
        .unwrap();
        assert_eq!(plugin.init_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn multiple_plugins_independent() {
        let mut mgr = PluginManager::new();
        let p1 = Arc::new(NoopPlugin::new("plugin-a", "1.0.0"));
        let p2 = Arc::new(NoopPlugin::new("plugin-b", "1.0.0"));
        mgr.register(p1, None).await.unwrap();
        mgr.register(p2, None).await.unwrap();
        assert_eq!(mgr.len(), 2);
        mgr.unload("plugin-a").await.unwrap();
        assert_eq!(mgr.len(), 1);
        assert!(!mgr.is_loaded("plugin-a"));
        assert!(mgr.is_loaded("plugin-b"));
    }

    #[tokio::test]
    async fn empty_manager_returns_empty_lists() {
        let mgr = PluginManager::new();
        assert!(mgr.is_empty());
        assert!(mgr.list_plugins().is_empty());
        assert!(mgr.all_tool_specs().is_empty());
        assert!(mgr.all_tool_executors().is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[tokio::test]
    async fn plugin_config_access() {
        let mut mgr = PluginManager::new();
        let plugin = Arc::new(NoopPlugin::new("cfg-plugin", "1.0.0"));
        let cfg = PluginConfig {
            enabled: true,
            settings: [("key".into(), "val".into())].into(),
            tool_config: BTreeMap::new(),
        };
        mgr.register(plugin, Some(cfg.clone())).await.unwrap();
        let stored = mgr.plugin_config("cfg-plugin").unwrap();
        assert_eq!(stored.settings.get("key").unwrap(), "val");
    }

    #[tokio::test]
    async fn after_tool_call_hook_fires() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct AfterToolPlugin {
            called: AtomicBool,
        }
        #[async_trait]
        impl Plugin for AfterToolPlugin {
            fn descriptor(&self) -> PluginDescriptor {
                PluginDescriptor {
                    name: "after-tool".into(),
                    version: "1.0.0".into(),
                    description: "test".into(),
                    author: None,
                    hooks: vec![PluginHook::AfterToolCall],
                    enabled_by_default: true,
                    required: false,
                }
            }
            async fn initialize(&self, _config: &PluginConfig) -> PluginResult<()> {
                Ok(())
            }
            async fn on_after_tool_call(&self, _name: &str, _result: &str) -> PluginResult<()> {
                self.called.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        }

        let plugin = Arc::new(AfterToolPlugin {
            called: AtomicBool::new(false),
        });
        let mut mgr = PluginManager::new();
        mgr.register(plugin.clone(), None).await.unwrap();
        mgr.fire_after_tool_call("bash", "ok").await;
        assert!(plugin.called.load(std::sync::atomic::Ordering::SeqCst));
    }
}