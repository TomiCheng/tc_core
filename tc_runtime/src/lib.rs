//! Low-level runtime support for the `tc_rust` workspace.
//!
//! Capability tokens exist on every target, so a caller picks a backend without
//! writing architecture `cfg` checks. The branches for other architectures fold
//! away at compile time.
//!
//! ```
//! use tc_runtime::intrinsics::{aarch64, x86};
//!
//! let backend = if x86::Pclmulqdq::detect().is_some() {
//!     "pclmulqdq"
//! } else if aarch64::Aes::detect().is_some() {
//!     "pmull"
//! } else {
//!     "portable"
//! };
//! # let _ = backend;
//! ```
//!
//! ```
//! use tc_runtime::intrinsics::x86::{Aes, Avx2, Sse2};
//!
//! assert_eq!(Sse2::detect().is_some(), Sse2::is_enabled());
//! assert_eq!(Aes::detect().is_some(), Aes::is_enabled());
//! assert_eq!(Avx2::detect().is_some(), Avx2::is_enabled());
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod intrinsics;
