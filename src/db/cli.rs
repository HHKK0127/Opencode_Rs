#![allow(dead_code)]
use log::info;
use std::process::Command;

/// sqlx-cli wrapper for migration management
pub struct MigrationCli;

impl MigrationCli {
    /// Create a new migration file
    /// Usage: sqlx migrate add -r migration_name
    pub fn create_migration(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Creating migration: {}", name);

        let output = Command::new("sqlx")
            .args(["migrate", "add", "-r", name])
            .current_dir(".")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to create migration: {}", stderr).into());
        }

        info!("✓ Migration created: {}", name);
        Ok(())
    }

    /// Run all pending migrations
    /// Usage: sqlx migrate run --database-url sqlite:./data.db
    pub fn run_migrations(database_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Running migrations on {}", database_url);

        let output = Command::new("sqlx")
            .args(["migrate", "run", "--database-url", database_url])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Migration failed: {}", stderr).into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        info!("✓ Migrations completed");
        Ok(stdout)
    }

    /// Revert the last migration
    /// Usage: sqlx migrate revert --database-url sqlite:./data.db
    pub fn revert_migration(database_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Reverting last migration on {}", database_url);

        let output = Command::new("sqlx")
            .args(["migrate", "revert", "--database-url", database_url])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Revert failed: {}", stderr).into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        info!("✓ Migration reverted");
        Ok(stdout)
    }

    /// Get migration information
    /// Usage: sqlx migrate info --database-url sqlite:./data.db
    pub fn migration_info(database_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Fetching migration info for {}", database_url);

        let output = Command::new("sqlx")
            .args(["migrate", "info", "--database-url", database_url])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to get migration info: {}", stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Validate all migrations without running them
    pub fn validate_migrations(database_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        info!("Validating migrations for {}", database_url);

        let output = Command::new("sqlx")
            .args(["migrate", "info", "--database-url", database_url])
            .output()?;

        let info_output = String::from_utf8_lossy(&output.stdout);

        // Check if any migration is in error state
        let has_errors = info_output.contains("ERROR") || info_output.contains("FAILED");

        if has_errors {
            info!("⚠️  Migration validation failed");
            return Ok(false);
        }

        info!("✓ All migrations validated successfully");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_cli_creation() {
        // Verify that MigrationCli can be instantiated
        let _ = MigrationCli;
    }
}
