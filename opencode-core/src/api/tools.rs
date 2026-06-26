use actix_web::{get, HttpResponse};

#[get("/experimental/tool/ids")]
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
        ]
    }))
}

#[get("/experimental/tool")]
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
        ]
    }))
}
