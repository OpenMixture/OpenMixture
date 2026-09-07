//! The future, sole pixel executor for Mixture.
//!
//! M0 reserves the dependency boundary over `mixture-core`. The `wgpu` dependency,
//! explicit GPU context, shaders, and readback arrive in M1. No adapter is acquired
//! and no rendering API is provided by this foundation crate.
