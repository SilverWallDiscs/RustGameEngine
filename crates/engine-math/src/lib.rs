#![forbid(unsafe_code)]
//! Data-oriented math primitives built on glam.
//!
//! Coordinate conventions, frustum extraction, bounds, and transform helpers are added
//! in the math phase.
//!
//! # Examples
//!
//! ~~~rust
//! use glam::Vec3;
//! let forward = Vec3::new(0.0, 0.0, -1.0).normalize();
//! assert_eq!(forward, Vec3::NEG_Z);
//! ~~~
