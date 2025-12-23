//! # SurrealDB Configuration Engine
//!
//! Provides embedded SurrealDB integration with Live Query support for
//! reactive, zero-downtime configuration updates.
//!
//! ## Architecture
//!
//! This crate isolates the heavy SurrealDB/RocksDB dependency from the
//! core gateway logic, enabling faster incremental compilation and better
//! separation of concerns.

pub mod db;
pub mod error;
pub mod schema;
pub mod watcher;

pub use db::{init_database, DatabaseConfig};
pub use error::ConfigError;
pub use schema::{create_route, delete_route, get_all_routes, get_route, update_route, seed_default_routes};
pub use watcher::start_config_watcher;
