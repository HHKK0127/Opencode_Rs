use chromiumoxide::Page;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

use super::types::TabInfo;

pub struct TabHandle {
    pub tab_id: String,
    pub page: Page,
}

impl TabHandle {
    pub async fn is_alive(&self) -> bool {
        self.page
            .execute(
                chromiumoxide::cdp::browser_protocol::target::GetTargetInfoParams {
                    target_id: Some(self.page.target_id().clone()),
                },
            )
            .await
            .is_ok()
    }
}

pub struct BrowserSession {
    pub name: String,
    pub current_tab: Option<TabHandle>,
    pub tabs: HashMap<String, TabInfo>,
    pub group_title: Option<String>,
    pub e_ref_counter: AtomicU64,
}

impl BrowserSession {
    pub fn new(name: String, group_title: Option<String>) -> Self {
        Self {
            name,
            current_tab: None,
            tabs: HashMap::new(),
            group_title,
            e_ref_counter: AtomicU64::new(0),
        }
    }

    pub fn next_e_ref(&self) -> String {
        let id = self.e_ref_counter.fetch_add(1, Ordering::SeqCst);
        format!("@e{}", id)
    }

    pub fn reset_e_refs(&self) {
        self.e_ref_counter.store(0, Ordering::SeqCst);
    }
}

pub type SessionStore = RwLock<HashMap<String, BrowserSession>>;
