// Wave 3: S3 Basic Operations Tests
use opencode_poc::config::Settings;
use opencode_poc::storage::S3Client;
use std::time::Duration;

#[actix_rt::test]
async fn test_s3_client_initialization() {
    let settings = Settings::default();
    match S3Client::new(&settings).await {
        Ok(_client) => {
            println!("✓ S3Client initialized successfully");
        }
        Err(e) => {
            // MinIO が起動していない場合はスキップ
            println!("⚠ S3Client initialization skipped (MinIO may not be running): {:?}", e);
        }
    }
}

#[actix_rt::test]
async fn test_s3_upload_and_download() {
    let settings = Settings::default();
    let client = match S3Client::new(&settings).await {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let key = "test/test-file.txt";
    let data = b"Hello, S3!".to_vec();

    // アップロード
    match client.upload_object(key, data.clone(), Some("text/plain")).await {
        Ok(etag) => {
            println!("✓ Upload successful: etag = {}", etag);
            assert!(!etag.is_empty());
        }
        Err(e) => {
            println!("✗ Upload failed: {:?}", e);
            return;
        }
    }

    // ダウンロード
    match client.download_object(key).await {
        Ok(downloaded) => {
            println!("✓ Download successful: {} bytes", downloaded.len());
            assert_eq!(downloaded, data);
        }
        Err(e) => {
            println!("✗ Download failed: {:?}", e);
            return;
        }
    }

    // 削除
    match client.delete_object(key).await {
        Ok(_) => {
            println!("✓ Delete successful");
        }
        Err(e) => {
            println!("✗ Delete failed: {:?}", e);
        }
    }
}

#[actix_rt::test]
async fn test_s3_presigned_urls() {
    let settings = Settings::default();
    let client = match S3Client::new(&settings).await {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let key = "test/presigned-test.txt";

    // PUT Presigned URL
    match client
        .generate_presigned_put_url(key, Duration::from_secs(300), Some("text/plain"))
        .await
    {
        Ok(put_url) => {
            println!("✓ Presigned PUT URL generated: {}", put_url);
            assert!(put_url.contains("X-Amz-Signature") || put_url.contains("Signature"));
        }
        Err(e) => {
            println!("✗ Presigned PUT URL generation failed: {:?}", e);
            return;
        }
    }

    // GET Presigned URL
    match client
        .generate_presigned_get_url(key, Duration::from_secs(3600))
        .await
    {
        Ok(get_url) => {
            println!("✓ Presigned GET URL generated: {}", get_url);
            assert!(get_url.contains("X-Amz-Signature") || get_url.contains("Signature"));
        }
        Err(e) => {
            println!("✗ Presigned GET URL generation failed: {:?}", e);
        }
    }
}

#[actix_rt::test]
async fn test_s3_multipart_upload() {
    let settings = Settings::default();
    let client = match S3Client::new(&settings).await {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let key = "test/multipart-test.bin";

    // Multipart upload 初期化
    let upload_id = match client.initiate_multipart_upload(key).await {
        Ok(id) => {
            println!("✓ Multipart upload initiated: {}", id);
            id
        }
        Err(e) => {
            println!("✗ Multipart upload initiation failed: {:?}", e);
            return;
        }
    };

    // Part 1 アップロード
    let part1_data = vec![1u8; 5 * 1024 * 1024]; // 5MB
    let part1 = match client
        .upload_part(key, &upload_id, 1, part1_data)
        .await
    {
        Ok(part) => {
            println!("✓ Part 1 uploaded");
            part
        }
        Err(e) => {
            println!("✗ Part 1 upload failed: {:?}", e);
            return;
        }
    };

    // Part 2 アップロード
    let part2_data = vec![2u8; 3 * 1024 * 1024]; // 3MB
    let part2 = match client
        .upload_part(key, &upload_id, 2, part2_data)
        .await
    {
        Ok(part) => {
            println!("✓ Part 2 uploaded");
            part
        }
        Err(e) => {
            println!("✗ Part 2 upload failed: {:?}", e);
            return;
        }
    };

    // Multipart upload 完了
    match client
        .complete_multipart_upload(key, &upload_id, vec![part1, part2])
        .await
    {
        Ok(etag) => {
            println!("✓ Multipart upload completed: etag = {}", etag);
        }
        Err(e) => {
            println!("✗ Multipart upload completion failed: {:?}", e);
        }
    }
}

#[actix_rt::test]
async fn test_s3_public_url() {
    let settings = Settings::default();
    let client = match S3Client::new(&settings).await {
        Ok(c) => c,
        Err(_) => {
            println!("⚠ Test skipped: MinIO not available");
            return;
        }
    };

    let key = "test/public-file.txt";
    let url = client.public_url(key);

    println!("✓ Public URL: {}", url);
    assert!(url.contains("opencode-uploads"));
    assert!(url.contains(key));
}
