fn main() {
    // Start opencode-core server in background thread
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = opencode_core::config::OpenCodeConfig::default();
            let server = opencode_core::OpenCodeServer::new(config);
            server.start().await.unwrap();
        });
    });

    // Brief pause to let server start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Start Tauri application
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
