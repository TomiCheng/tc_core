# Changelog

All notable changes to `tc_constant_time` are documented in this file.

## 0.1.0 - Unreleased

Initial release, prepared for publication on crates.io.

### Added

- `Choice`, a one-bit value with a private field, `from_lsb` and `unwrap_u8`,
  and `!`, `&`, `|`, `^` combinators that keep predicates unrevealed. There is
  no implicit conversion to `bool`. `from_lsb` keeps the least significant bit
  and is not a nonzero test.
- `ConditionallySelectable` with `conditional_select`, `conditional_assign`,
  and `conditional_swap`. The assignment and swap defaults support types
  without `Copy` or `Clone`; integers and arrays override them to update
  storage in place.
- `ConstantTimeEq` for equality without an early exit on a mismatch, and
  `ConditionallyNegatable` for wrapping negation. Negating the minimum value of
  a signed type leaves it unchanged.
- `ConstantTimeOrd` with `ct_lt` and derived `ct_gt`, `ct_le`, and `ct_ge`,
  following each integer type's numeric order.
- Implementations for `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`,
  `i32`, `i64`, `i128`, and `isize`. Fixed-size arrays support selection,
  assignment, swap, negation, and equality when the element type does; slices
  support equality. Empty arrays and slices compare equal.
- `fixed_time_eq` for byte slices, the only convenience function that converts
  a `Choice` to `bool`. Use it when the verification result is meant to be
  public, such as tag or checksum checks.
- Dependency-free, `no_std` builds with no feature flags, no `alloc`, and no
  `unsafe` code. Missing public documentation and unsafe code are rejected by
  crate-level lints.
- Integration tests grouped by `choice`, `integers`, `array`, `slice`, and
  `traits`, plus exhaustive 256-by-256 checks of every `u8` and `i8` API
  against public reference operations, and doctests covering bit
  normalization, empty arrays, and custom trait implementations.

### Compatibility

- Requires Rust 1.85 or later and uses Rust edition 2024.
- Slice lengths are public: unequal lengths return a zero choice immediately,
  and `fixed_time_eq` returns `false`. Neither hides the lengths.
- Array and slice ordering is left to higher layers, because limb order is a
  domain choice.
- `Choice::from_lsb` passes its bit through `core::hint::black_box`, a
  best-effort optimization barrier rather than a guarantee of constant-time
  machine code. Generated code must be reviewed for the target compiler and
  hardware. Branching on `unwrap_u8` or indexing with a revealed bit is
  outside the timing contract.
- Licensed under MIT OR Apache-2.0; both license texts are included in the
  published package.
