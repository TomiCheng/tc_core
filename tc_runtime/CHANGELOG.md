# Changelog

All notable changes to `tc_runtime` are documented in this file.

## 0.1.0 - Unreleased

Initial release, prepared for publication on crates.io.

### Added

- Native CI on Linux x64 and ARM64, macOS ARM64, and Windows ARM64, with
  Rust 1.85 regression checks, compile-time feature-floor tests, and portable
  target checks.

- CPU capability detection for x86 and x86_64: AES-NI, AVX2, BMI1 in
  64-bit mode, BMI2, PCLMULQDQ at 128/256/512-bit widths, SSE2, SSE4.1,
  and SSSE3. AVX-family probes check the required operating-system state.
- AArch64 capability detection for AES/PMULL, DIT, NEON, SHA-1/SHA-256,
  SHA-3/SHA-512, and SM3/SM4.
- Private-constructor, copyable proof tokens with `detect()` and
  `is_enabled()` methods. Capability types are available on every target;
  unsupported architectures return `None` and `false`, respectively.
- Bouncy Castle-compatible x86 aliases and capability groupings.
- Dependency-free, `no_std` default builds. The optional `aarch64-detect`
  feature uses `cpufeatures` for runtime probing on Linux, Android, and Apple
  platforms. Without runtime probing, AArch64 uses compile-time capabilities.
- Per-capability Cargo disable features and cached `TC_DISABLE_*`
  environment-variable overrides with the optional `std` feature.
- Caller-facing API documentation and executable examples covering token
  passing, portable fallback, capability aliases, and token construction
  restrictions.

### Compatibility

- Requires Rust 1.85 or later and uses Rust edition 2024.
- AArch64 Windows and bare-metal targets rely on compile-time capabilities.
- Capability tokens do not enable compiler target features. A DIT token
  reports support but does not set `PSTATE.DIT`.
- Licensed under MIT OR Apache-2.0; both license texts are included in the
  published package.
