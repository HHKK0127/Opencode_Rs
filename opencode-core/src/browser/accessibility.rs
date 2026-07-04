use super::types::AccessibilityNode;
use super::session::BrowserSession;

const MAX_TREE_DEPTH: usize = 100;

pub fn build_accessibility_tree(
    nodes: &[CdpAXNode],
    session: &BrowserSession,
    depth: usize,
) -> Vec<AccessibilityNode> {
    if depth > MAX_TREE_DEPTH {
        tracing::warn!("Accessibility tree depth limit ({MAX_TREE_DEPTH}) reached");
        return vec![];
    }

    nodes
        .iter()
        .filter_map(|node| {
            let e_ref = if is_interactive(&node.role) {
                session.next_e_ref()
            } else {
                String::new()
            };

            let children = build_accessibility_tree(&node.children, session, depth + 1);

            Some(AccessibilityNode {
                e_ref,
                role: node.role.clone(),
                name: node.name.clone().unwrap_or_default(),
                value: node.value.clone(),
                children,
            })
        })
        .collect()
}

fn is_interactive(role: &str) -> bool {
    matches!(
        role,
        "button"
            | "link"
            | "textbox"
            | "checkbox"
            | "radio"
            | "combobox"
            | "menuitem"
            | "tab"
            | "switch"
            | "slider"
            | "searchbox"
            | "spinbutton"
            | "option"
    )
}

/// Simplified CDP AX node representation for parsing.
#[derive(Debug, Clone)]
pub struct CdpAXNode {
    pub role: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub children: Vec<CdpAXNode>,
}

pub fn parse_ax_tree(
    ax_nodes: &[&chromiumoxide::cdp::browser_protocol::accessibility::AxNode],
    children_map: &std::collections::HashMap<String, Vec<String>>,
    node_map: &std::collections::HashMap<String, &chromiumoxide::cdp::browser_protocol::accessibility::AxNode>,
) -> Vec<CdpAXNode> {
    ax_nodes
        .iter()
        .filter_map(|node| {
            let child_ids = children_map.get(node.node_id.as_ref()).cloned().unwrap_or_default();
            let children: Vec<CdpAXNode> = child_ids
                .iter()
                .filter_map(|child_id| {
                    node_map.get(child_id).map(|child| {
                        let grandchild_ids = children_map.get(child_id).cloned().unwrap_or_default();
                        let grandchild_nodes: Vec<CdpAXNode> = grandchild_ids
                            .iter()
                            .filter_map(|gc_id| {
                                node_map.get(gc_id).map(|gc| CdpAXNode {
                                    role: gc.role.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    name: gc.name.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).map(String::from),
                                    value: gc.value.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).map(String::from),
                                    children: vec![],
                                })
                            })
                            .collect();

                        CdpAXNode {
                            role: child.role.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            name: child.name.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).map(String::from),
                            value: child.value.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).map(String::from),
                            children: grandchild_nodes,
                        }
                    })
                })
                .collect();

            Some(CdpAXNode {
                role: node.role.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                name: node.name.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).map(String::from),
                value: node.value.as_ref().and_then(|v| v.value.as_ref()).and_then(|v| v.as_str()).map(String::from),
                children,
            })
        })
        .collect()
}
