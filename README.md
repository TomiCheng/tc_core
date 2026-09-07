# tc_core

A Rust workspace of `no_std`, dependency-free building blocks for cryptographic
and mathematical code. Each crate is published separately and keeps its own
README, changelog, and validation commands.

[![CI](https://github.com/TomiCheng/tc_core/actions/workflows/ci.yml/badge.svg)](https://github.com/TomiCheng/tc_core/actions/workflows/ci.yml)

## Crates

| Crate | Description |
| --- | --- |
| [`tc_constant_time`](tc_constant_time) | Masked selection, comparison, ordering, and conditional arithmetic without value-dependent branches. No dependencies, no feature flags, no `alloc`, no `unsafe`. |
| [`tc_runtime`](tc_runtime) | CPU feature detection and capability proof tokens for x86 and AArch64, used to select an optimized backend at runtime. Dependency-free by default; optional `std` and `aarch64-detect` features. |

`tc_constant_time` provides timing primitives rather than mathematical
operations, so cryptographic algorithms and mathematical backends can share it
without depending on each other. `tc_runtime` decides which backend runs;
`tc_constant_time` governs how a backend handles secret values. Neither depends
on the other.

## Requirements

Rust 1.85 or later, edition 2024. Every crate builds without `std` and without
heap allocation.

## Workspace checks

```text
cargo test --locked
cargo test --locked --all-features
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --no-deps --all-features
```

CI additionally runs these on Linux x64, i686 and ARM64, macOS ARM64, and
Windows ARM64, checks the `wasm32-unknown-unknown` and `aarch64-unknown-none`
targets, pins an MSRV job to Rust 1.85.0, and verifies each crate's package
archive. See [.github/workflows/ci.yml](.github/workflows/ci.yml).

## License

Licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
