use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::Instant;

const JWT_SECRET: &str = "test_secret_key_for_poc_verification";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

fn main() {
    // JWT生成パフォーマンステスト
    println!("=== JWT Generation Performance Test ===\n");

    let iterations = 1000;
    let start = Instant::now();

    for i in 0..iterations {
        let now = Utc::now();
        let claims = Claims {
            sub: format!("user{}", i),
            iat: now.timestamp(),
            exp: (now + chrono::Duration::hours(24)).timestamp(),
        };

        let _ = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
        );
    }

    let duration = start.elapsed();
    let avg_us = duration.as_micros() as f64 / iterations as f64;
    let avg_ms = duration.as_millis() as f64 / iterations as f64;

    println!("Total time: {:?}", duration);
    println!("Iterations: {}", iterations);
    println!("Average per JWT: {:.3} ms ({:.1} μs)", avg_ms, avg_us);
    println!("Throughput: {:.0} JWTs/sec\n", 1000.0 / avg_ms);

    // Argon2 パスワードハッシュテスト
    println!("=== Argon2 Password Hashing Test ===\n");

    use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
    use argon2::password_hash::SaltString;
    use rand_core::OsRng;

    let password = "testpassword123";
    let iterations_argon2 = 100;

    let start = Instant::now();
    let mut hashes = Vec::new();

    for _ in 0..iterations_argon2 {
        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();
        if let Ok(hash) = argon2.hash_password(password.as_bytes(), &salt) {
            hashes.push(hash);
        }
    }

    let hash_duration = start.elapsed();
    let avg_hash_ms = hash_duration.as_millis() as f64 / iterations_argon2 as f64;

    println!("Hash generation time: {:?}", hash_duration);
    println!("Iterations: {}", iterations_argon2);
    println!("Average per hash: {:.0} ms", avg_hash_ms);
    println!("Throughput: {:.1} hashes/sec\n", 1000.0 / avg_hash_ms);

    // パスワード検証テスト
    if let Some(first_hash) = hashes.first() {
        let start = Instant::now();
        let verify_iterations = 100;

        for _ in 0..verify_iterations {
            let _ = Argon2::default()
                .verify_password(password.as_bytes(), &first_hash);
        }

        let verify_duration = start.elapsed();
        let avg_verify_ms = verify_duration.as_millis() as f64 / verify_iterations as f64;

        println!("Password verification time: {:?}", verify_duration);
        println!("Iterations: {}", verify_iterations);
        println!("Average per verification: {:.0} ms", avg_verify_ms);
        println!("Throughput: {:.1} verifications/sec\n", 1000.0 / avg_verify_ms);
    }

    println!("=== Summary ===");
    println!("JWT Generation: {:.3} ms per token", avg_ms);
    println!("Argon2 Hashing: {:.0} ms per hash", avg_hash_ms);
}
