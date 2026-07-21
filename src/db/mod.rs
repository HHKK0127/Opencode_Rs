#![allow(dead_code, unused_imports, clippy::all)]

pub mod cli;
pub mod migration;
pub mod optimization;

pub use cli::MigrationCli;
pub use migration::{get_migration_history, init_database, MigrationRecord};
pub use optimization::{
    analyze_tables, get_database_stats, optimize_database, vacuum_database, DatabaseStats,
};
