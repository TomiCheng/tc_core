//! CPU feature detection and capability proof tokens for x86 and AArch64.
//!
//! This crate is `no_std` by default and has no default dependencies. The `std`
//! feature enables environment-variable overrides; `aarch64-detect` enables
//! optional runtime detection on supported AArch64 operating systems.
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
//!
//! # Calling an optimized backend
//!
//! A token records detected support. It does not apply `#[target_feature]` to
//! your function or remove an intrinsic's other safety requirements. Keep
//! architecture-specific code behind `cfg`, and provide a portable fallback.
//! This example computes the same wrapping addition on every target:
//!
//! ```
//! use tc_runtime::intrinsics::x86::Sse2;
//!
//! #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//! #[target_feature(enable = "sse2")]
//! unsafe fn add_sse2(a: i32, b: i32, _proof: Sse2) -> i32 {
//!     #[cfg(target_arch = "x86")]
//!     use core::arch::x86::{_mm_add_epi32, _mm_cvtsi128_si32, _mm_set1_epi32};
//!     #[cfg(target_arch = "x86_64")]
//!     use core::arch::x86_64::{_mm_add_epi32, _mm_cvtsi128_si32, _mm_set1_epi32};
//!     _mm_cvtsi128_si32(_mm_add_epi32(_mm_set1_epi32(a), _mm_set1_epi32(b)))
//! }
//!
//! fn add(a: i32, b: i32) -> i32 {
//!     #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//!     if let Some(proof) = Sse2::detect() {
//!         // SAFETY: the token establishes SSE2 support. No pointers are used.
//!         return unsafe { add_sse2(a, b, proof) };
//!     }
//!     a.wrapping_add(b)
//! }
//!
//! assert_eq!(add(20, 22), 42);
//! assert_eq!(add(i32::MAX, 1), i32::MIN);
//! ```
//!
//! # Feature selection
//!
//! Default builds need neither `std` nor allocation. Enable `aarch64-detect`
//! for optional runtime probing on supported AArch64 operating systems; see
//! [`intrinsics::aarch64`] for coverage. The `std` feature enables `TC_DISABLE_*`
//! environment overrides. Set these before starting the process: their presence
//! disables a capability regardless of their value, and results are cached.
//!
//! Each capability's `is_enabled` documentation names its Cargo disable feature
//! and environment override on its native architecture. Cargo disable features
//! take precedence over detected support. These controls affect backend
//! selection, not instructions the compiler may emit elsewhere in the program.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod intrinsics;
