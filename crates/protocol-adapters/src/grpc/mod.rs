//! gRPC Protocol Adapter
//!
//! Provides gRPC functionality with dynamic transcoding using prost-reflect:
//! - JSON ↔ Protobuf conversion without compile-time proto
//! - DescriptorPool for runtime proto loading
//! - Dynamic service handling

pub mod transcoder;
pub mod service;

pub use transcoder::{GrpcTranscoder, TranscodeError};
pub use service::DynamicGrpcService;
