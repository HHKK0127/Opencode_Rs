use actix_web::web;

pub mod health;
pub mod auth;
pub mod files;
pub mod file_search;
pub mod upload_chunks;
pub mod upload_progress;
pub mod users;
pub mod projects;
pub mod admin;
pub mod cache_integration;  // Wave 4 Day 13: Cache integration helpers
// pub mod presigned_urls;       // Wave 3 Day 2: Reimplemented with new Storage trait
// pub mod file_metadata;         // Wave 3 Day 2: Reimplemented with new Storage trait
pub mod metrics;
// pub mod s3_operations;         // Wave 3 Day 2: Reimplemented with new Storage trait

#[cfg(test)]
mod tests;

#[cfg(test)]
mod security_tests;

#[cfg(test)]
mod integration_tests;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure)
            .configure(auth::configure)
            .configure(files::configure)
            .configure(file_search::configure)
            .configure(upload_chunks::configure)
            .configure(upload_progress::configure)
            .configure(users::configure)
            .configure(projects::configure)
            .configure(admin::configure)
            // .configure(presigned_urls::configure)  // Wave 3 Day 2
            // .configure(file_metadata::configure)   // Wave 3 Day 2
            .configure(metrics::configure)
            // .configure(s3_operations::configure)   // Wave 3 Day 2: S3 storage operations
    );
}
