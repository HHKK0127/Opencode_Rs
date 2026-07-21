#![allow(dead_code)]
use log::info;
use sqlx::sqlite::SqlitePool;

/// Initialize database with initial schema and migrations
pub async fn init_database(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing database schema...");
    create_migration_history_table(pool).await?;
    run_all_migrations(pool).await?;
    info!("Database initialization completed");
    Ok(())
}

async fn create_migration_history_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            execution_time BIGINT NOT NULL,
            success BOOLEAN NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("✓ Migration history table ready");
    Ok(())
}

async fn run_all_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let migrations = vec![
        (
            1,
            "001_create_users_table",
            include_str!("../../migrations/001_create_users_table.sql"),
        ),
        (
            2,
            "002_create_files_table",
            include_str!("../../migrations/002_create_files_table.sql"),
        ),
        (
            3,
            "003_performance_indexes",
            include_str!("../../migrations/003_performance_indexes.sql"),
        ),
        (
            4,
            "004_file_metadata",
            include_str!("../../migrations/004_file_metadata.sql"),
        ),
        (
            5,
            "005_s3_file_metadata",
            include_str!("../../migrations/005_s3_file_metadata.sql"),
        ),
    ];

    for (version, name, sql) in migrations {
        if !migration_exists(pool, version).await? {
            info!("Running migration {}: {}...", version, name);

            let start = std::time::Instant::now();
            sqlx::query(sql).execute(pool).await?;
            let duration = start.elapsed();

            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, execution_time, success)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(version as i64)
            .bind(name)
            .bind(duration.as_millis() as i64)
            .bind(true)
            .execute(pool)
            .await?;

            info!(
                "✓ Migration {} completed in {:.2}ms",
                version,
                duration.as_millis()
            );
        }
    }

    Ok(())
}

async fn migration_exists(pool: &SqlitePool, version: i32) -> Result<bool, sqlx::Error> {
    let result: Option<(i64,)> =
        sqlx::query_as("SELECT version FROM _sqlx_migrations WHERE version = $1")
            .bind(version as i64)
            .fetch_optional(pool)
            .await?;

    Ok(result.is_some())
}

pub async fn get_migration_history(pool: &SqlitePool) -> Result<Vec<MigrationRecord>, sqlx::Error> {
    let records = sqlx::query_as::<_, (i64, String, String, i64, bool)>(
        "SELECT version, description, installed_on::text, execution_time, success
         FROM _sqlx_migrations ORDER BY version ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(records
        .into_iter()
        .map(
            |(version, description, installed_on, execution_time, success)| MigrationRecord {
                version,
                description,
                installed_on,
                execution_time,
                success,
            },
        )
        .collect())
}

#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: i64,
    pub description: String,
    pub installed_on: String,
    pub execution_time: i64,
    pub success: bool,
}
