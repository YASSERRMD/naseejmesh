//! TokioExecutor for HTTP/2 support in Hyper 1.0.
//!
//! Hyper 1.0 is runtime-agnostic and requires an explicit executor for
//! spawning HTTP/2 stream tasks. This module provides the Tokio integration.

use std::future::Future;

/// Executor that spawns futures onto the Tokio runtime.
///
/// This is required for HTTP/2 support, as HTTP/2 multiplexing needs the
/// ability to spawn background tasks for handling concurrent streams within
/// a single TCP connection.
#[derive(Clone, Copy, Debug, Default)]
pub struct TokioExecutor;

impl TokioExecutor {
    /// Create a new TokioExecutor instance
    pub fn new() -> Self {
        Self
    }
}

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, future: F) {
        tokio::spawn(future);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_spawns() {
        let executor = TokioExecutor::new();
        let (tx, rx) = tokio::sync::oneshot::channel();

        hyper::rt::Executor::execute(&executor, async move {
            tx.send(42).unwrap();
        });

        let result = rx.await.unwrap();
        assert_eq!(result, 42);
    }
}
