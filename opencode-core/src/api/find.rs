use actix_web::{get, web::Query, HttpResponse};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct FindFileParams {
    pub pattern: Option<String>,
    pub path: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct FindSymbolParams {
    pub query: Option<String>,
    pub path: Option<String>,
}

#[get("/find/file")]
pub async fn find_file(params: Query<FindFileParams>) -> HttpResponse {
    let dir = params.path.as_deref().unwrap_or(".");
    let pattern = params.pattern.as_deref().unwrap_or("*");
    let limit = params.limit.unwrap_or(100) as usize;

    let mut results = Vec::new();
    find_files_recursive(Path::new(dir), &pattern, 0, limit, &mut results);

    HttpResponse::Ok().json(serde_json::json!({"results": results}))
}

#[get("/find/symbol")]
pub async fn find_symbol() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"results": []}))
}

fn find_files_recursive(
    dir: &Path,
    pattern: &str,
    depth: usize,
    limit: usize,
    results: &mut Vec<serde_json::Value>,
) {
    if depth > 5 || results.len() >= limit {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if results.len() >= limit {
            break;
        }
        let path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if matches_simple_glob(name, pattern) {
                results.push(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "filename": name,
                    "is_directory": is_dir,
                    "size": entry.metadata().ok().map(|m| m.len() as i64),
                }));
            }
        }

        if is_dir {
            find_files_recursive(&path, pattern, depth + 1, limit, results);
        }
    }
}

fn matches_simple_glob(name: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "*.*" {
        return true;
    }
    if !pattern.contains('*') && !pattern.contains('?') {
        return name == pattern;
    }

    let glob_chars: Vec<char> = pattern.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();
    simple_glob_match(&glob_chars, &name_chars, 0, 0)
}

fn simple_glob_match(pat: &[char], name: &[char], pi: usize, ni: usize) -> bool {
    if pi == pat.len() {
        return ni == name.len();
    }

    match pat[pi] {
        '*' => {
            if pi + 1 == pat.len() {
                return true;
            }
            for j in ni..=name.len() {
                if simple_glob_match(pat, name, pi + 1, j) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if ni < name.len() {
                simple_glob_match(pat, name, pi + 1, ni + 1)
            } else {
                false
            }
        }
        c => {
            if ni < name.len() && name[ni] == c {
                simple_glob_match(pat, name, pi + 1, ni + 1)
            } else {
                false
            }
        }
    }
}
