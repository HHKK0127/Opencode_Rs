use sqlx::PgPool;
use log::info;

/// Apply performance optimizations — PostgreSQL uses server-side config (postgresql.conf).
/// This function sets session-level options where applicable.
pub async fn optimize_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("PostgreSQL: performance tuning handled by server config (postgresql.conf)");
    // Set client-side statement timeout for safety
    sqlx::query("SET statement_timeout = '30s'")
        .execute(pool)
        .await?;
    info!("✓ Statement timeout set to 30s");
    Ok(())
}

/// Run ANALYZE to update planner statistics
pub async fn analyze_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running ANALYZE on database tables...");
    sqlx::query("ANALYZE").execute(pool).await?;
    info!("✓ Database analysis completed");
    Ok(())
}

/// Run VACUUM to reclaim space
pub async fn vacuum_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running VACUUM on database...");
    sqlx::query("VACUUM").execute(pool).await?;
    info!("✓ Database vacuum completed");
    Ok(())
}

/// Get database statistics from pg_database
pub async fn get_database_stats(pool: &PgPool) -> Result<DatabaseStats, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT pg_database_size(current_database())"
    )
    .fetch_one(pool)
    .await?;

    let total_bytes = row.0 as u64;
    let page_size: u64 = 8192; // PostgreSQL default block size

    Ok(DatabaseStats {
        page_count: total_bytes / page_size,
        page_size,
        journal_mode: "wal".to_string(), // PostgreSQL always uses WAL
        total_size_bytes: total_bytes,
    })
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub page_count: u64,
    pub page_size: u64,
    pub journal_mode: String,
    pub total_size_bytes: u64,
}
