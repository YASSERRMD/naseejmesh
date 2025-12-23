//! Secure body handling with size limits.
//!
//! This module addresses the critical security vulnerability of unbounded
//! body consumption. Using `http_body_util::Limited`, we enforce strict
//! upper bounds on request body sizes to prevent OOM denial of service.

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Incoming;

use crate::error::GatewayError;

/// Default maximum body size: 2MB
/// This can be overridden per-route in future phases.
pub const DEFAULT_MAX_BODY_SIZE: usize = 2 * 1024 * 1024;

/// Collect an incoming body with a size limit.
///
/// This function wraps the body in a `Limited` wrapper that enforces
/// the specified size limit. If the body exceeds the limit during
/// streaming, collection fails immediately.
///
/// # Security
///
/// This prevents memory exhaustion attacks where a malicious client:
/// 1. Sends a huge `Content-Length` header but little data (reservation attack)
/// 2. Sends unbounded data without a length header (streaming attack)
///
/// # Arguments
///
/// * `body` - The incoming HTTP body stream
/// * `max_size` - Maximum allowed body size in bytes
///
/// # Returns
///
/// * `Ok(Bytes)` - The collected body data
/// * `Err(GatewayError::PayloadTooLarge)` - If the limit is exceeded
pub async fn collect_body_limited(body: Incoming, max_size: usize) -> Result<Bytes, GatewayError> {
    let limited = Limited::new(body, max_size);

    limited
        .collect()
        .await
        .map(|collected| collected.to_bytes())
        .map_err(|e| {
            // Check if this was a limit error
            if e.to_string().contains("length limit exceeded") {
                GatewayError::PayloadTooLarge {
                    size: 0, // We don't know the actual size
                    limit: max_size as u64,
                }
            } else {
                GatewayError::BodyReadError(e.to_string())
            }
        })
}

/// Collect an incoming body with the default size limit.
pub async fn collect_body(body: Incoming) -> Result<Bytes, GatewayError> {
    collect_body_limited(body, DEFAULT_MAX_BODY_SIZE).await
}

/// Zero-copy utilities for Bytes manipulation.
///
/// The `Bytes` type from the bytes crate provides reference-counted,
/// contiguous memory that can be sliced without copying.
pub mod zero_copy {
    use bytes::Bytes;

    /// Create a zero-copy slice of a Bytes buffer.
    ///
    /// This increments the reference count but does NOT copy data.
    /// Perfect for reading headers or signatures without duplicating
    /// the entire payload.
    #[inline]
    pub fn slice(data: &Bytes, start: usize, end: usize) -> Bytes {
        data.slice(start..end)
    }

    /// Split a Bytes buffer at a position.
    ///
    /// Returns (left, right) where both are zero-copy views.
    #[inline]
    pub fn split_at(data: Bytes, mid: usize) -> (Bytes, Bytes) {
        let right = data.slice(mid..);
        let left = data.slice(..mid);
        (left, right)
    }

    /// Check if a Bytes buffer starts with a specific prefix.
    #[inline]
    pub fn starts_with(data: &Bytes, prefix: &[u8]) -> bool {
        data.len() >= prefix.len() && &data[..prefix.len()] == prefix
    }
}

#[cfg(test)]
mod tests {
    use super::zero_copy::*;
    use bytes::Bytes;

    #[test]
    fn test_zero_copy_slice() {
        let data = Bytes::from("Hello, World!");
        let sliced = slice(&data, 0, 5);

        assert_eq!(&sliced[..], b"Hello");
        // Original data is still valid
        assert_eq!(data.len(), 13);
    }

    #[test]
    fn test_zero_copy_split() {
        let data = Bytes::from("HelloWorld");
        let (left, right) = split_at(data, 5);

        assert_eq!(&left[..], b"Hello");
        assert_eq!(&right[..], b"World");
    }

    #[test]
    fn test_starts_with() {
        let data = Bytes::from("HTTP/1.1 200 OK");

        assert!(starts_with(&data, b"HTTP"));
        assert!(!starts_with(&data, b"HTTPS"));
    }
}
