use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use tokio::time::{timeout, Duration};

use super::accessibility;
use super::session::{BrowserSession, TabHandle};
use super::types::*;
use super::{BrowserError, BrowserManager};
use crate::api::events::{self, EventBus};

impl BrowserManager {
    // ========================================================================
    // navigate
    // ========================================================================
    pub async fn navigate(
        &self,
        session_name: &str,
        args: NavigateArgs,
        bus: &EventBus,
    ) -> Result<NavigateResponse, BrowserError> {
        let browser = self.browser.as_ref().ok_or(BrowserError::NotRunning)?;

        let new_tab = args.new_tab.unwrap_or(false);

        let page = if new_tab {
            browser
                .new_page(&args.url)
                .await
                .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?
        } else {
            let sessions = self.sessions.read().await;
            match sessions
                .get(session_name)
                .and_then(|s| s.current_tab.as_ref())
            {
                Some(handle) => handle.page.clone(),
                None => {
                    drop(sessions);
                    browser
                        .new_page(&args.url)
                        .await
                        .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?
                }
            }
        };

        let _result = timeout(
            Duration::from_secs(self.config.navigation_timeout_secs),
            page.goto(&args.url),
        )
        .await
        .map_err(|_| BrowserError::NavigationTimeout {
            secs: self.config.navigation_timeout_secs,
            url: args.url.clone(),
        })?
        .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;

        let tab_id = page.target_id().inner().clone();
        let final_url = args.url.clone();

        let title = page.get_title().await.ok().flatten().unwrap_or_default();

        let tab_info = TabInfo {
            tab_id: tab_id.clone(),
            url: final_url.clone(),
            title,
            active: true,
            group_title: args.group_title.clone(),
        };

        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(session_name.to_string()).or_insert_with(|| {
            BrowserSession::new(session_name.to_string(), args.group_title.clone())
        });
        session.tabs.insert(tab_id.clone(), tab_info);
        session.current_tab = Some(TabHandle {
            tab_id: tab_id.clone(),
            page,
        });
        drop(sessions);

        let _ = events::emit_event(
            bus,
            "browser.navigated",
            serde_json::json!({
                "sessionID": session_name,
                "url": final_url,
                "tabId": tab_id,
            }),
        );

        Ok(NavigateResponse {
            success: true,
            url: final_url,
            tab_id,
        })
    }

    // ========================================================================
    // find_tab
    // ========================================================================
    pub async fn find_tab(
        &self,
        session_name: &str,
        args: FindTabArgs,
    ) -> Result<FindTabResponse, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let tab = if args.active.unwrap_or(false) {
            session.tabs.values().find(|t| t.active).cloned()
        } else {
            session.tabs.values().find(|t| t.url == args.url).cloned()
        };

        match tab {
            Some(info) => Ok(FindTabResponse {
                success: true,
                url: info.url,
                tab_id: info.tab_id,
            }),
            None => Err(BrowserError::TabNotFound {
                session: session_name.into(),
            }),
        }
    }

    // ========================================================================
    // snapshot
    // ========================================================================
    pub async fn snapshot(&self, session_name: &str) -> Result<SnapshotResponse, BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;
        let handle = session
            .current_tab
            .as_ref()
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        session.reset_e_refs();

        let url = handle.page.url().await.ok().flatten().unwrap_or_default();
        let title = handle
            .page
            .get_title()
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        let ax_tree_result = handle
            .page
            .execute(
                chromiumoxide::cdp::browser_protocol::accessibility::GetFullAxTreeParams {
                    depth: Some(100),
                    frame_id: None,
                },
            )
            .await
            .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;

        let ax_nodes = &ax_tree_result.nodes;

        let mut children_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        let mut node_map: std::collections::HashMap<
            String,
            &chromiumoxide::cdp::browser_protocol::accessibility::AxNode,
        > = std::collections::HashMap::new();

        for node in ax_nodes {
            node_map.insert(node.node_id.inner().clone(), node);
            if let Some(ref child_ids) = node.child_ids {
                for child_id in child_ids {
                    children_map
                        .entry(node.node_id.inner().clone())
                        .or_default()
                        .push(child_id.inner().clone());
                }
            }
        }

        let root_nodes: Vec<&chromiumoxide::cdp::browser_protocol::accessibility::AxNode> =
            ax_nodes
                .iter()
                .filter(|n| {
                    !ax_nodes.iter().any(|parent| {
                        parent
                            .child_ids
                            .as_ref()
                            .map_or(false, |ids| ids.contains(&n.node_id))
                    })
                })
                .collect();

        let cdp_roots = accessibility::parse_ax_tree(&root_nodes, &children_map, &node_map);
        let tree = accessibility::build_accessibility_tree(&cdp_roots, session, 0);

        Ok(SnapshotResponse { url, title, tree })
    }

    // ========================================================================
    // click
    // ========================================================================
    pub async fn click(
        &self,
        session_name: &str,
        args: ClickArgs,
    ) -> Result<ClickResponse, BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;
        let handle = session
            .current_tab
            .as_ref()
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let selector = &args.selector;
        let js = format!(
            r#"
            (() => {{
                const el = document.querySelector('{}');
                if (!el) return null;
                el.click();
                return {{ tag: el.tagName.toLowerCase(), text: (el.textContent || '').trim().substring(0, 200) }};
            }})()
            "#,
            selector.replace('\'', "\\'")
        );

        let result = handle
            .page
            .evaluate(js.as_str())
            .await
            .map_err(|e| BrowserError::EvaluateError(e.to_string()))?;

        let value = result
            .value()
            .ok_or_else(|| BrowserError::ElementNotFound {
                selector: selector.clone(),
            })?;

        if value.is_null() {
            return Err(BrowserError::ElementNotFound {
                selector: selector.clone(),
            });
        }

        let tag = value
            .get("tag")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let text = value
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(ClickResponse {
            success: true,
            tag,
            text,
        })
    }

    // ========================================================================
    // fill
    // ========================================================================
    pub async fn fill(
        &self,
        session_name: &str,
        args: FillArgs,
    ) -> Result<FillResponse, BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;
        let handle = session
            .current_tab
            .as_ref()
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let selector = args.selector.replace('\'', "\\'");
        let value = args.value.replace('\'', "\\'").replace('\n', "\\n");

        let js = format!(
            r#"
            (() => {{
                const el = document.querySelector('{}');
                if (!el) return null;
                const tag = el.tagName.toLowerCase();
                if (el.isContentEditable || tag === 'textarea' || (tag === 'input' && ['text','search','email','url','password','tel','number'].includes(el.type))) {{
                    if (el.isContentEditable) {{
                        el.textContent = '{}';
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        return {{ tag, mode: 'contenteditable' }};
                    }} else {{
                        el.value = '{}';
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return {{ tag, mode: 'value' }};
                    }}
                }}
                return null;
            }})()
            "#,
            selector, value, value
        );

        let result = handle
            .page
            .evaluate(js.as_str())
            .await
            .map_err(|e| BrowserError::EvaluateError(e.to_string()))?;

        let value = result
            .value()
            .ok_or_else(|| BrowserError::ElementNotFound {
                selector: args.selector.clone(),
            })?;

        if value.is_null() {
            return Err(BrowserError::ElementNotFound {
                selector: args.selector.clone(),
            });
        }

        let tag = value
            .get("tag")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let mode = value
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("value")
            .to_string();

        Ok(FillResponse {
            success: true,
            tag,
            mode,
        })
    }

    // ========================================================================
    // evaluate
    // ========================================================================
    pub async fn evaluate(
        &self,
        session_name: &str,
        args: EvaluateArgs,
    ) -> Result<EvaluateResponse, BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;
        let handle = session
            .current_tab
            .as_ref()
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let wrapped = format!("(async () => {{ {} }})()", args.code);

        let result = handle
            .page
            .evaluate(wrapped.as_str())
            .await
            .map_err(|e| BrowserError::EvaluateError(e.to_string()))?;

        let value = result.value().unwrap_or(&serde_json::Value::Null).clone();

        let result_type = match &value {
            serde_json::Value::String(_) => "string",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Null => "null",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
        .to_string();

        Ok(EvaluateResponse { result_type, value })
    }

    // ========================================================================
    // screenshot
    // ========================================================================
    pub async fn screenshot(
        &self,
        session_name: &str,
        args: ScreenshotArgs,
    ) -> Result<ScreenshotResponse, BrowserError> {
        let path = args.validate_path(&self.config.screenshot_dir)?;

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;
        let handle = session
            .current_tab
            .as_ref()
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let format_str = args.format.as_deref().unwrap_or("png");
        let quality = args.quality.unwrap_or(80);

        let png = format_str != "jpeg";

        let screenshot_params =
            chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams {
                format: Some(if png {
                    chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png
                } else {
                    chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Jpeg
                }),
                quality: Some(quality as i64),
                clip: None,
                from_surface: Some(true),
                capture_beyond_viewport: Some(false),
                optimize_for_speed: None,
            };

        let data = timeout(
            Duration::from_secs(self.config.screenshot_timeout_secs),
            handle.page.screenshot(screenshot_params),
        )
        .await
        .map_err(|_| BrowserError::ScreenshotError("Screenshot timeout".into()))?
        .map_err(|e| BrowserError::ScreenshotError(e.to_string()))?;

        std::fs::write(&path, &data)?;

        let size_bytes = data.len() as u64;
        let max_size = self.config.max_screenshot_size_mb * 1024 * 1024;
        if size_bytes > max_size {
            return Err(BrowserError::ScreenshotError(format!(
                "Screenshot too large: {} bytes (max {} bytes)",
                size_bytes, max_size
            )));
        }

        let mime_type = if png { "image/png" } else { "image/jpeg" }.to_string();

        Ok(ScreenshotResponse {
            format: format_str.to_string(),
            path: path.to_string_lossy().to_string(),
            size_bytes,
            mime_type,
        })
    }

    // ========================================================================
    // save_as_pdf
    // ========================================================================
    pub async fn save_as_pdf(
        &self,
        session_name: &str,
        args: SaveAsPdfArgs,
    ) -> Result<PdfResponse, BrowserError> {
        let path = args.validate_path(&self.config.pdf_dir)?;

        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;
        let handle = session
            .current_tab
            .as_ref()
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let paper_width = 8.5;
        let paper_height = 11.0;

        let landscape = args.landscape.unwrap_or(false);
        let scale = args.scale.unwrap_or(1.0);

        let pdf_params = PrintToPdfParams {
            landscape: Some(landscape),
            display_header_footer: Some(false),
            print_background: Some(args.print_background.unwrap_or(true)),
            scale: Some(scale),
            paper_width: Some(paper_width),
            paper_height: Some(paper_height),
            margin_top: Some(0.4),
            margin_bottom: Some(0.4),
            margin_left: Some(0.4),
            margin_right: Some(0.4),
            page_ranges: None,
            header_template: None,
            footer_template: None,
            prefer_css_page_size: Some(true),
            transfer_mode: None,
            generate_tagged_pdf: None,
            generate_document_outline: None,
        };

        let decoded = timeout(
            Duration::from_secs(self.config.screenshot_timeout_secs * 3),
            handle.page.pdf(pdf_params),
        )
        .await
        .map_err(|_| BrowserError::PdfError("PDF generation timeout".into()))?
        .map_err(|e| BrowserError::PdfError(e.to_string()))?;

        std::fs::write(&path, &decoded)?;

        let size_bytes = decoded.len() as u64;

        let title = handle
            .page
            .get_title()
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        Ok(PdfResponse {
            path: path.to_string_lossy().to_string(),
            size_bytes,
            mime_type: "application/pdf".to_string(),
            page_title: title,
        })
    }

    // ========================================================================
    // list_tabs
    // ========================================================================
    pub async fn list_tabs(&self, session_name: &str) -> Result<TabsResponse, BrowserError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        let tabs: Vec<TabInfo> = session.tabs.values().cloned().collect();

        Ok(TabsResponse {
            success: true,
            tabs,
        })
    }

    // ========================================================================
    // close_tab
    // ========================================================================
    pub async fn close_tab(&self, session_name: &str) -> Result<CloseTabResponse, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_name)
            .ok_or(BrowserError::TabNotFound {
                session: session_name.into(),
            })?;

        if let Some(handle) = session.current_tab.take() {
            let _ = handle
                .page
                .execute(
                    chromiumoxide::cdp::browser_protocol::target::CloseTargetParams {
                        target_id: handle.page.target_id().clone(),
                    },
                )
                .await;
            session.tabs.remove(&handle.tab_id);
            Ok(CloseTabResponse {
                success: true,
                closed: true,
            })
        } else {
            Ok(CloseTabResponse {
                success: true,
                closed: false,
            })
        }
    }

    // ========================================================================
    // close_session
    // ========================================================================
    pub async fn close_session(
        &self,
        session_name: &str,
    ) -> Result<CloseSessionResponse, BrowserError> {
        let mut sessions = self.sessions.write().await;
        let session = match sessions.remove(session_name) {
            Some(s) => s,
            None => {
                return Ok(CloseSessionResponse {
                    success: true,
                    closed: 0,
                })
            }
        };

        let count = session.tabs.len();

        Ok(CloseSessionResponse {
            success: true,
            closed: count,
        })
    }
}
