# tc_zeroize

[![crates.io](https://img.shields.io/crates/v/tc_zeroize.svg)](https://crates.io/crates/tc_zeroize)
[![docs.rs](https://docs.rs/tc_zeroize/badge.svg)](https://docs.rs/tc_zeroize)
[![CI](https://github.com/TomiCheng/tc_core/actions/workflows/ci.yml/badge.svg)](https://github.com/TomiCheng/tc_core/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
![rustc](https://img.shields.io/badge/rustc-1.85+-blue.svg)

Explicit memory erasure with volatile writes and opt-in scope guards. The crate
is `no_std` and has no dependencies. A single, default-off `alloc` feature adds
heap-backed containers through the sysroot `alloc` crate. Its primitive
implementations use small, documented `unsafe` blocks for volatile writes.

Requires Rust 1.85 or later (edition 2024).

## API

| Item | Contract |
| --- | --- |
| `Zeroize::zeroize(&mut self)` | Explicitly erases the contents of the current value |
| `ZeroizeOnDrop` | Marks a type whose own destructor performs erasure; the marker generates no behavior |
| `Zeroizing::new(value)` | Owns a value and calls its `zeroize` before dropping it |
| `Deref` / `DerefMut` on `Zeroizing<T>` | Borrows the guarded value for ordinary operations |

| Types | Result of `zeroize` |
| --- | --- |
| `u8`, `u16`, `u32`, `u64`, `u128`, `usize` | Zero |
| `i8`, `i16`, `i32`, `i64`, `i128`, `isize` | Zero |
| `bool` | `false` |
| `char` | `'\0'` |
| `[T; N]`, `[T]` where `T: Zeroize` | Every element is cleared; length is preserved |
| `Option<T>` where `T: Zeroize` | A present payload is cleared, then dropped, and the option becomes `None` |
| `MaybeUninit<T>` for any `T` | A typed volatile zero store clears storage, excluding any padding guarantee; the slot remains logically uninitialized |
| `Vec<T>` where `T: Zeroize` (`alloc`) | Live elements are cleared and dropped, then the current allocation including spare capacity is cleared; length becomes zero and capacity is retained, subject to the padding limitation |
| `Box<T>` where `T: Zeroize + ?Sized` (`alloc`) | The contents are cleared in place; the box and allocation are retained, including the length of a boxed slice |

Empty arrays and slices are supported. Slice lengths are public. Array and slice
implementations visit every element and inherit the erasure behavior of `T`.
An option branches on whether a value is present. This is not a constant-time API.

## Features

| Features | Support |
| --- | --- |
| None (default) | Core-only `no_std`: primitives, arrays, slices, options, `MaybeUninit`, and scope guards |
| `alloc` | All core-only support plus `Vec<T>` and `Box<T: ?Sized>`; still `no_std`, with no external dependencies |

`String`, `VecDeque`, `BTreeMap`, and `Cow` are not supported.

## Capability and policy

`Zeroize` provides a capability, not a mandatory lifetime policy. General-purpose
storage types do not know whether their contents are secret; they should offer
explicit erasure without automatically imposing cleanup on all uses.

Types that know they hold secrets, such as private-key containers and private
RSA engines, can choose the `ZeroizeOnDrop` policy. Implementors must write their
own `Drop` implementation and call `Zeroize::zeroize` there. The marker alone
does not enforce or implement this behavior.

`Zeroizing<T>` applies that policy to a local value. The guard implements neither
`Clone` nor `Copy`. Dereferencing it can still allow a caller to copy or clone the
inner value; those separate values are not guarded. A big-integer type can
implement and re-export the erasure API; a private-key container can call
`Zeroize::zeroize` from its own destructor and guard local blinding values with
`Zeroizing`.

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
tc_zeroize = "0.1.0"
```

Heap-backed containers need the default-off `alloc` feature:

```toml
[dependencies]
tc_zeroize = { version = "0.1.0", features = ["alloc"] }
```

```rust
use tc_zeroize::{Zeroize, Zeroizing};

let mut scratch = [1_u8, 2, 3, 4];
scratch.zeroize();
assert_eq!(scratch, [0; 4]);

let mut optional = Some([7_u32, 9]);
optional.zeroize();
assert_eq!(optional, None);

{
    let mut secret = Zeroizing::new([42_u8; 32]);
    secret[0] = 7;
    assert_eq!(secret[0], 7);
} // The guard clears its current array before dropping it.
```

API documentation also has executable examples for implementing both traits.

## Mechanism and limitations

Primitive erasure uses `core::ptr::write_volatile` to prevent deletion of the
stores and ends with `compiler_fence(Ordering::SeqCst)` to constrain compiler
reordering. Composite implementations delegate to their contents. Every unsafe
block documents pointer validity and the validity of the replacement value.

These operations do not flush caches or supply a hardware memory barrier. They
do not erase copies left elsewhere by the compiler or operating system, such as
registers, stack spills, swap, or core dumps. Padding bytes are not covered.

`Copy` values permit implicit copies on by-value use. Clearing one binding does
not clear other copies; a `Copy` fixed-width integer type is one example. A
`Copy` type cannot implement `Drop`, so it cannot perform its own scope-exit
cleanup. Even moves of non-`Copy` types can leave bytes at an old location.
`Zeroizing` clears only its current contents, not those old copies.

With `alloc`, `Vec<T>::zeroize` clears live elements before their destructors run,
then clears the entire current allocation, including spare capacity. Typed
volatile stores do not guarantee erasure of padding in `T`; padding-free bytes
and integer limbs are fully covered. The empty vector retains its capacity.
This covers the current allocation, not every buffer used during its lifetime:
growth, `shrink_to_fit`, and `into_boxed_slice` can leave inaccessible old buffers.
Reserve enough capacity before storing secrets to avoid this gap. A `Box<[T]>`
is a good fit for fixed-size secret storage because its allocation has no spare
capacity and does not grow; converting an existing vector can still reallocate.

Drop-based cleanup requires the destructor to run. Forgetting or leaking the
guard, or aborting the process, bypasses cleanup. A panicking custom `zeroize`
implementation can also leave a composite value partially cleared.

## Validation

Integration tests import the public API and cover every supported primitive,
empty and nested containers, non-`Copy` elements, slice boundaries, clearing
before a payload's destructor, and guard cleanup on scope exit and unwinding.
With `alloc`, tests also check spare capacity in still-live vector allocations,
element destruction order, boxed slices, and nested vectors.
Crate and type documentation contains executable examples. Missing public docs
and unsafe operations without explicit unsafe blocks in unsafe functions are
rejected by crate-level lints.

Functional tests establish resulting values and cleanup order. They do not
prove that a compiler preserves the wipe; generated code must be inspected for
the target compiler and hardware when validating that property.

Run these commands from the workspace root:

```text
cargo test -p tc_zeroize --locked
cargo test -p tc_zeroize --locked --features alloc
cargo clippy -p tc_zeroize --all-targets --all-features --locked -- -D warnings
cargo fmt -p tc_zeroize --check
cargo doc -p tc_zeroize --no-deps --all-features --locked
```

Before a release, check the archive contents and run publication validation
from a committed checkout:

```text
cargo package -p tc_zeroize --list --locked
cargo publish -p tc_zeroize --dry-run --locked
```

The archive includes both license texts, this README, the changelog, the source,
and integration tests. It must not include `target/` or other build artifacts.
The publication dry run packages and verifies the crate without uploading it.

## License

Licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
