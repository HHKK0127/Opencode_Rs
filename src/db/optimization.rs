use sqlx::SqlitePool;
use log::info;

/// Apply performance optimizations to SQLite database
pub async fn optimize_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Applying SQLite performance optimizations...");

    // Write-Ahead Logging for better concurrency
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(pool)
        .await?;
    info!("✓ Enabled WAL mode");

    // Synchronous mode: NORMAL balances safety and performance
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(pool)
        .await?;
    info!("✓ Set synchronous mode to NORMAL");

    // Cache size: 64MB for better performance
    sqlx::query("PRAGMA cache_size = -64000")
        .execute(pool)
        .await?;
    info!("✓ Set cache size to 64MB");

    // Use memory for temporary storage
    sqlx::query("PRAGMA temp_store = MEMORY")
        .execute(pool)
        .await?;
    info!("✓ Set temp_store to MEMORY");

    // Foreign key support
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await?;
    info!("✓ Enabled foreign key constraints");

    // Query optimization
    sqlx::query("PRAGMA query_only = FALSE")
        .execute(pool)
        .await?;
    info!("✓ Query optimization enabled");

    Ok(())
}

/// Analyze database tables for query optimization
pub async fn analyze_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Running ANALYZE on database tables...");

    sqlx::query("ANALYZE")
        .execute(pool)
        .await?;

    info!("✓ Database analysis completed");

    Ok(())
}

/// Vacuum database to reclaim disk space
pub async fn vacuum_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Running VACUUM on database...");

    sqlx::query("VACUUM")
        .execute(pool)
        .await?;

    info!("✓ Database vacuum completed");

    Ok(())
}

/// Get database statistics
pub async fn get_database_stats(pool: &SqlitePool) -> Result<DatabaseStats, sqlx::Error> {
    let page_count: (i32,) = sqlx::query_as("PRAGMA page_count")
        .fetch_one(pool)
        .await?;

    let page_size: (i32,) = sqlx::query_as("PRAGMA page_size")
        .fetch_one(pool)
        .await?;

    let journal_mode: (String,) = sqlx::query_as("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;

    Ok(DatabaseStats {
        page_count: page_count.0 as u64,
        page_size: page_size.0 as u64,
        journal_mode: journal_mode.0,
        total_size_bytes: (page_count.0 as u64) * (page_size.0 as u64),
    })
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub page_count: u64,
    pub page_size: u64,
    pub journal_mode: String,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimization_runs_without_error() {
        let database_url = "sqlite::memory:";
        let pool = SqlitePool::connect(database_url).await.unwrap();

        let result = optimize_database(&pool).await;
        assert!(result.is_ok());
    }
}
