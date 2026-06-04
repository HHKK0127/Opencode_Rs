pub mod optimization;
pub mod migration;
pub mod cli;

pub use optimization::{optimize_database, analyze_tables, vacuum_database, get_database_stats, DatabaseStats};
pub use migration::{init_database, get_migration_history, MigrationRecord};
pub use cli::MigrationCli;
