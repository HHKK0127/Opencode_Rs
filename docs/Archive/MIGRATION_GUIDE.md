# Database Migration Operations Guide

## Overview

This guide covers database migration management using sqlx-cli and our custom migration tools.

## Prerequisites

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features sqlite

# Verify installation
sqlx --version
```

## Creating a New Migration

### Using sqlx-cli directly

```bash
# Create a reversible migration (recommended)
sqlx migrate add -r migration_name

# Create a non-reversible migration
sqlx migrate add migration_name
```

### Using custom CLI

```rust
// In your Rust code
use opencode_poc::db::MigrationCli;

MigrationCli::create_migration("add_new_feature_table")?;
```

### Example: Add Refresh Tokens Table

```bash
sqlx migrate add -r add_refresh_tokens_table
```

This creates two files:
- `migrations/20260527HHMMSS_add_refresh_tokens_table.up.sql`
- `migrations/20260527HHMMSS_add_refresh_tokens_table.down.sql`

Write your SQL:

```sql
-- up.sql
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

-- down.sql
DROP TABLE IF EXISTS refresh_tokens;
```

## Running Migrations

### Development Environment

```bash
# Run all pending migrations
sqlx migrate run --database-url sqlite:./data/dev.db

# Verify migrations
sqlx migrate info --database-url sqlite:./data/dev.db
```

### Staging Environment

```bash
# Set environment variable
export DATABASE_URL="postgresql://user:pass@staging-db.example.com/opencode"

# Run migrations
sqlx migrate run

# Verify
sqlx migrate info
```

### Production Environment

```bash
# Always verify in staging first!
# Then deploy via CI/CD (GitHub Actions)

# Or manual (emergency only)
export DATABASE_URL="postgresql://user:pass@prod-db.example.com/opencode"

# List migrations (no changes)
sqlx migrate info

# Run with confirmation
sqlx migrate run
```

## Rolling Back Migrations

### Revert Last Migration

```bash
# Using sqlx-cli
sqlx migrate revert --database-url sqlite:./data/dev.db

# Using custom CLI
use opencode_poc::db::MigrationCli;
MigrationCli::revert_migration(&database_url)?;
```

### Revert Multiple Migrations

```bash
# Revert last N migrations
for i in {1..3}; do
  sqlx migrate revert --database-url $DATABASE_URL
done
```

### Emergency Rollback

```bash
# If automatic rollback fails:
# 1. Stop the application
# 2. Restore database from backup
# 3. Revert migrations manually:

sqlite3 ./data/prod.db < down.sql
# ... repeat for each migration to revert

# 4. Redeploy application
```

## Monitoring Migrations

### API Endpoints

```bash
# Check database status
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/status

# View migration history
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/migrations

# Get migration info from CLI
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/migrate/info

# Validate migrations
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/migrate/validate
```

### Command Line

```bash
# View migration status
sqlx migrate info --database-url sqlite:./data/dev.db

# Check database performance
sqlite3 ./data/dev.db "PRAGMA page_count; PRAGMA page_size;"

# Analyze tables
sqlite3 ./data/dev.db "ANALYZE; SELECT * FROM sqlite_stat1 LIMIT 10;"
```

## Best Practices

### ✅ DO

- **Test migrations locally first**
  ```bash
  sqlx migrate run --database-url sqlite:./test.db
  ```

- **Always create reversible migrations** (use `-r` flag)
  ```bash
  sqlx migrate add -r feature_name
  ```

- **Write idempotent SQL**
  ```sql
  CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY
  );
  ```

- **Keep migrations small and focused**
  ```sql
  -- Good: Single responsibility
  CREATE TABLE tokens (...);
  CREATE INDEX idx_tokens_user ON tokens(user_id);
  
  -- Avoid: Multiple unrelated changes
  CREATE TABLE tokens (...);
  ALTER TABLE users ADD COLUMN ...;
  CREATE TABLE projects (...);
  ```

- **Include rollback testing**
  ```bash
  # Run migration
  sqlx migrate run --database-url sqlite:./test.db
  
  # Test application with new schema
  cargo test
  
  # Revert migration
  sqlx migrate revert --database-url sqlite:./test.db
  
  # Verify rollback worked
  sqlx migrate info --database-url sqlite:./test.db
  ```

- **Document complex migrations**
  ```sql
  -- Migration: Add user preferences
  -- Purpose: Store user UI preferences (theme, language, etc)
  -- Context: Required for feature request #123
  -- Rollback: Safe to rollback, no data dependencies
  
  CREATE TABLE user_preferences (
    user_id TEXT PRIMARY KEY,
    theme TEXT DEFAULT 'light',
    language TEXT DEFAULT 'en',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
  );
  ```

### ❌ DON'T

- **Don't skip test runs**
  ```bash
  # Bad: Deploy without testing
  git push && # CI/CD runs
  
  # Good: Test locally first
  sqlx migrate run --database-url sqlite:./test.db
  cargo test
  git push
  ```

- **Don't create non-reversible migrations** (unless absolutely necessary)
  ```bash
  # Bad
  sqlx migrate add delete_all_old_data
  
  # Good
  sqlx migrate add -r archive_old_data
  ```

- **Don't mix multiple concerns in one migration**
  ```sql
  -- Bad: Multiple unrelated changes
  CREATE TABLE new_feature (...);
  ALTER TABLE users DROP COLUMN old_field;
  CREATE TABLE analytics (...);
  
  -- Good: One migration per concern
  -- 001_create_new_feature.sql
  CREATE TABLE new_feature (...);
  
  -- 002_remove_old_user_field.sql
  ALTER TABLE users DROP COLUMN old_field;
  
  -- 003_create_analytics.sql
  CREATE TABLE analytics (...);
  ```

- **Don't manually edit migration files** after they've run
  - If you need to fix a migration, create a new one
  ```sql
  -- 001_initial_schema.sql (✓ already run)
  CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT);
  
  -- 002_fix_schema.sql (✗ don't edit 001_initial_schema.sql)
  -- Instead, create new migration to fix:
  ALTER TABLE users ADD COLUMN email TEXT UNIQUE;
  ```

## Troubleshooting

### Migration Status Unknown

```bash
# Check migration table
sqlite3 ./data/dev.db "SELECT * FROM _sqlx_migrations ORDER BY version;"

# If table is corrupt, check database integrity
sqlite3 ./data/dev.db "PRAGMA integrity_check;"
```

### Migration Won't Rollback

```bash
# Check if down.sql exists
ls migrations/*.down.sql

# Manually inspect migration
cat migrations/20260527HHMMSS_name.down.sql

# If migration is marked as success but can't rollback:
# 1. Restore from backup
# 2. Investigate root cause
# 3. Create new migration to fix inconsistency
```

### Database Locked During Migration

```bash
# Check for open connections
sqlite3 ./data/dev.db "PRAGMA journal_mode=WAL;"

# Kill blocking processes (if using server database)
# For SQLite: Close all database handles

# Retry migration
sqlx migrate run --database-url sqlite:./data/dev.db
```

## CI/CD Pipeline

### Automatic Migrations

Migrations are **automatically run** when pushed to `main`:

```
git push
    ↓
GitHub Actions trigger (migrate.yml)
    ↓
✓ Validate migrations (sqlx-cli)
    ↓
✓ Run on staging DB
    ↓
✓ Approve production migration
    ↓
✓ Run on production DB
    ↓
✓ Create release tag
    ↓
✓ Notify Slack
```

### Manual Workflow Dispatch

```bash
# Trigger manually from GitHub Actions UI
# Or via CLI:
gh workflow run migrate.yml \
  -f environment=production
```

## Performance Considerations

After running migrations, optimize the database:

```bash
# Run ANALYZE for query optimization
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/analyze

# Vacuum to reclaim space (staging/dev only)
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/vacuum

# Check database statistics
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/v1/admin/db/status
```

## Emergency Contacts

- **Database Admin**: [contact]
- **DevOps Team**: [contact]
- **On-call Runbook**: [link]

## References

- [sqlx-cli Documentation](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
- [SQLx Migrations](https://github.com/launchbadge/sqlx/blob/main/sqlx-core/src/migrate)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
