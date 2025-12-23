//! # Naseej Test Harness
//!
//! Comprehensive testing infrastructure for NaseejMesh providing:
//! - Integration test utilities
//! - Mock services and fixtures
//! - Property-based testing helpers
//! - Load test scenarios

pub mod fixtures;
pub mod http_client;
pub mod mock_backend;
pub mod assertions;
pub mod scenarios;

pub use fixtures::{TestFixture, RouteFixture, PayloadGenerator};
pub use http_client::TestClient;
pub use mock_backend::MockBackend;
pub use assertions::ResponseAssertions;
