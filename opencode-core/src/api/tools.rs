use actix_web::HttpResponse;

pub async fn tool_ids() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "ids": [
            "read",
            "write",
            "edit",
            "bash",
            "glob",
            "grep",
            "notify",
            "task",
            "browser_navigate",
            "browser_snapshot",
            "browser_click",
            "browser_fill",
            "browser_evaluate",
            "browser_screenshot",
            "browser_save_as_pdf",
            "browser_list_tabs",
            "browser_close_tab",
        ]
    }))
}

pub async fn list_tools() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "tools": [
            {"id": "read", "name": "Read", "description": "Read file contents"},
            {"id": "write", "name": "Write", "description": "Write file contents"},
            {"id": "edit", "name": "Edit", "description": "Edit file"},
            {"id": "bash", "name": "Bash", "description": "Execute bash command"},
            {"id": "glob", "name": "Glob", "description": "Search files by pattern"},
            {"id": "grep", "name": "Grep", "description": "Search file contents"},
            {"id": "notify", "name": "Notify", "description": "Send notification"},
            {"id": "task", "name": "Task", "description": "Manage tasks"},
            {"id": "browser_navigate", "name": "Browser Navigate", "description": "Navigate browser to a URL", "parameters": {"type": "object", "properties": {"url": {"type": "string"}, "new_tab": {"type": "boolean"}, "group_title": {"type": "string"}}, "required": ["url"]}},
            {"id": "browser_snapshot", "name": "Browser Snapshot", "description": "Get accessibility tree of current page"},
            {"id": "browser_click", "name": "Browser Click", "description": "Click an element on the page", "parameters": {"type": "object", "properties": {"selector": {"type": "string"}}, "required": ["selector"]}},
            {"id": "browser_fill", "name": "Browser Fill", "description": "Fill a form field", "parameters": {"type": "object", "properties": {"selector": {"type": "string"}, "value": {"type": "string"}}, "required": ["selector", "value"]}},
            {"id": "browser_evaluate", "name": "Browser Evaluate", "description": "Execute JavaScript in the browser", "parameters": {"type": "object", "properties": {"code": {"type": "string"}}, "required": ["code"]}},
            {"id": "browser_screenshot", "name": "Browser Screenshot", "description": "Take a screenshot of the page", "parameters": {"type": "object", "properties": {"format": {"type": "string"}, "quality": {"type": "integer"}, "path": {"type": "string"}}, "required": ["path"]}},
            {"id": "browser_save_as_pdf", "name": "Browser Save as PDF", "description": "Save the current page as PDF", "parameters": {"type": "object", "properties": {"paper_format": {"type": "string"}, "landscape": {"type": "boolean"}, "scale": {"type": "number"}, "path": {"type": "string"}}, "required": ["path"]}},
            {"id": "browser_list_tabs", "name": "Browser List Tabs", "description": "List all open browser tabs"},
            {"id": "browser_close_tab", "name": "Browser Close Tab", "description": "Close the current browser tab"},
        ]
    }))
}
