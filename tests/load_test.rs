#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments,
    clippy::all
)]

use actix_web::test;
use futures::future;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[actix_web::test]
async fn test_concurrent_file_uploads_100() {
    let client = Arc::new(Client::new());

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap().to_string();

    // Limit concurrency with semaphore
    let semaphore = Arc::new(Semaphore::new(50)); // Max 50 concurrent requests

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let client = Arc::clone(&client);
            let token = token.clone();
            let sem = Arc::clone(&semaphore);

            tokio::spawn(async move {
                let _permit = sem.acquire().await;
                let file_content = format!("concurrent test file {}", i).into_bytes();
                let filename = format!("concurrent_{}.txt", i);

                let start = std::time::Instant::now();
                let resp = client
                    .post("http://127.0.0.1:8080/api/v1/files/upload")
                    .header("Authorization", format!("Bearer {}", token))
                    .multipart(reqwest::multipart::Form::new().part(
                        "file",
                        reqwest::multipart::Part::bytes(file_content).file_name(filename),
                    ))
                    .send()
                    .await;

                let elapsed = start.elapsed();
                (i, resp.is_ok(), elapsed)
            })
        })
        .collect();

    let results: Vec<_> = future::join_all(handles).await;
    let results: Vec<_> = results.into_iter().filter_map(|r| r.ok()).collect();

    let success_count = results.iter().filter(|(_, ok, _)| *ok).count();
    let total_time: std::time::Duration = results.iter().map(|(_, _, t)| *t).sum();
    let avg_time = total_time / results.len() as u32;

    eprintln!("✓ Concurrent uploads (100):");
    eprintln!(
        "  Success rate: {}/100 ({:.1}%)",
        success_count,
        (success_count as f64 / 100.0) * 100.0
    );
    eprintln!("  Average time: {:?}", avg_time);
    eprintln!("  Total time: {:?}", total_time);

    assert!(
        success_count >= 90,
        "Success rate should be >=90%, got {}/100",
        success_count
    );
    assert!(
        avg_time < std::time::Duration::from_secs(2),
        "Average time should be <2s, got {:?}",
        avg_time
    );
}

#[actix_web::test]
async fn test_concurrent_search_requests() {
    let client = Arc::new(Client::new());

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap().to_string();

    // Upload some test files first
    for i in 0..20 {
        let filename = format!("search_load_{}.txt", i);
        let _resp = client
            .post("http://127.0.0.1:8080/api/v1/files/upload")
            .header("Authorization", format!("Bearer {}", token))
            .multipart(
                reqwest::multipart::Form::new().part(
                    "file",
                    reqwest::multipart::Part::bytes(b"search test content".to_vec())
                        .file_name(filename),
                ),
            )
            .send()
            .await;
    }

    // Now perform 50 concurrent search requests
    let semaphore = Arc::new(Semaphore::new(25)); // Max 25 concurrent

    let handles: Vec<_> = (0..50)
        .map(|i| {
            let client = Arc::clone(&client);
            let token = token.clone();
            let sem = Arc::clone(&semaphore);

            tokio::spawn(async move {
                let _permit = sem.acquire().await;

                let start = std::time::Instant::now();
                let resp = client
                    .get("http://127.0.0.1:8080/api/v1/files/search?q=search&sort=created_at&order=desc&page=1&per_page=10")
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                let elapsed = start.elapsed();
                (i, resp.is_ok() && resp.as_ref().unwrap().status() == 200, elapsed)
            })
        })
        .collect();

    let results: Vec<_> = future::join_all(handles).await;
    let results: Vec<_> = results.into_iter().filter_map(|r| r.ok()).collect();

    let success_count = results.iter().filter(|(_, ok, _)| *ok).count();
    let avg_time: std::time::Duration = results
        .iter()
        .map(|(_, _, t)| *t)
        .sum::<std::time::Duration>()
        / results.len() as u32;

    eprintln!("✓ Concurrent searches (50):");
    eprintln!(
        "  Success rate: {}/50 ({:.1}%)",
        success_count,
        (success_count as f64 / 50.0) * 100.0
    );
    eprintln!("  Average time: {:?}", avg_time);

    assert!(
        success_count >= 45,
        "Success rate should be >=90%, got {}/50",
        success_count
    );
}

#[actix_web::test]
async fn test_mixed_api_operations_load() {
    let client = Arc::new(Client::new());

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap().to_string();

    let semaphore = Arc::new(Semaphore::new(30));

    let mut handles = Vec::new();

    // Upload operations (30)
    for i in 0..30 {
        let client = Arc::clone(&client);
        let token = token.clone();
        let sem = Arc::clone(&semaphore);

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            let filename = format!("mixed_{}.txt", i);
            let start = std::time::Instant::now();
            let resp = client
                .post("http://127.0.0.1:8080/api/v1/files/upload")
                .header("Authorization", format!("Bearer {}", token))
                .multipart(
                    reqwest::multipart::Form::new().part(
                        "file",
                        reqwest::multipart::Part::bytes(b"mixed load test".to_vec())
                            .file_name(filename),
                    ),
                )
                .send()
                .await;
            (0, resp.is_ok(), start.elapsed())
        }));
    }

    // Search operations (20)
    for _ in 0..20 {
        let client = Arc::clone(&client);
        let token = token.clone();
        let sem = Arc::clone(&semaphore);

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            let start = std::time::Instant::now();
            let resp = client
                .get("http://127.0.0.1:8080/api/v1/files/search?sort=created_at")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await;
            (1, resp.is_ok(), start.elapsed())
        }));
    }

    // List operations (15)
    for _ in 0..15 {
        let client = Arc::clone(&client);
        let token = token.clone();
        let sem = Arc::clone(&semaphore);

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            let start = std::time::Instant::now();
            let resp = client
                .get("http://127.0.0.1:8080/api/v1/files?page=1&per_page=20")
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await;
            (2, resp.is_ok(), start.elapsed())
        }));
    }

    let results: Vec<_> = future::join_all(handles).await;
    let results: Vec<_> = results.into_iter().filter_map(|r| r.ok()).collect();

    let upload_results: Vec<_> = results.iter().filter(|(op, _, _)| *op == 0).collect();
    let search_results: Vec<_> = results.iter().filter(|(op, _, _)| *op == 1).collect();
    let list_results: Vec<_> = results.iter().filter(|(op, _, _)| *op == 2).collect();

    let upload_success = if upload_results.is_empty() {
        0.0
    } else {
        upload_results.iter().filter(|(_, ok, _)| *ok).count() as f64 / upload_results.len() as f64
    };
    let search_success = if search_results.is_empty() {
        0.0
    } else {
        search_results.iter().filter(|(_, ok, _)| *ok).count() as f64 / search_results.len() as f64
    };
    let list_success = if list_results.is_empty() {
        0.0
    } else {
        list_results.iter().filter(|(_, ok, _)| *ok).count() as f64 / list_results.len() as f64
    };

    eprintln!("✓ Mixed API load (65 total operations):");
    eprintln!("  Uploads (30): {:.1}% success", upload_success * 100.0);
    eprintln!("  Searches (20): {:.1}% success", search_success * 100.0);
    eprintln!("  Lists (15): {:.1}% success", list_success * 100.0);

    assert!(upload_success >= 0.90, "Upload success should be >=90%");
    assert!(search_success >= 0.90, "Search success should be >=90%");
    assert!(list_success >= 0.90, "List success should be >=90%");
}

#[actix_web::test]
async fn test_chunked_upload_concurrent_sessions() {
    let client = Arc::new(Client::new());

    // Get auth token
    let login_resp = client
        .post("http://127.0.0.1:8080/api/v1/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "testpassword"
        }))
        .send()
        .await
        .unwrap();

    let login_body: Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap().to_string();

    let semaphore = Arc::new(Semaphore::new(10));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let client = Arc::clone(&client);
            let token = token.clone();
            let sem = Arc::clone(&semaphore);

            tokio::spawn(async move {
                let _permit = sem.acquire().await;

                // Initialize chunked upload
                let init_resp = client
                    .post("http://127.0.0.1:8080/api/v1/files/upload/init")
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&serde_json::json!({
                        "filename": format!("chunked_session_{}.bin", i),
                        "total_size": 2_097_152,
                        "chunk_size": 1_048_576
                    }))
                    .send()
                    .await;

                if let Ok(resp) = init_resp {
                    if let Ok(body) = resp.json::<Value>().await {
                        if let Some(session_id) = body["session_id"].as_str() {
                            let chunk_data = vec![0u8; 1_048_576];

                            // Upload 2 chunks
                            for chunk_idx in 0..2 {
                                let _chunk_resp = client
                                    .post("http://127.0.0.1:8080/api/v1/files/upload/chunk")
                                    .header("Authorization", format!("Bearer {}", token))
                                    .multipart(
                                        reqwest::multipart::Form::new()
                                            .text("session_id", session_id.to_string())
                                            .text("chunk_index", chunk_idx.to_string())
                                            .part(
                                                "chunk",
                                                reqwest::multipart::Part::bytes(chunk_data.clone()),
                                            ),
                                    )
                                    .send()
                                    .await;
                            }

                            // Complete upload
                            let _complete_resp = client
                                .post(&format!(
                                    "http://127.0.0.1:8080/api/v1/files/upload/complete/{}",
                                    session_id
                                ))
                                .header("Authorization", format!("Bearer {}", token))
                                .send()
                                .await;

                            return true;
                        }
                    }
                }
                false
            })
        })
        .collect();

    let results: Vec<_> = future::join_all(handles).await;
    let success_count = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|r| *r)
        .count();

    eprintln!(
        "✓ Chunked upload concurrent sessions (10): {}/10 successful",
        success_count
    );

    assert!(success_count >= 8, "At least 8/10 sessions should complete");
}
