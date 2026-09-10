# Changelog

All notable changes to `tc_zeroize` are documented in this file.

## 0.1.0 - Unreleased

Initial release.

### Added

- `Zeroize`, a capability trait whose `zeroize` explicitly erases the contents
  reached through the current mutable borrow. Implementing it does not impose
  automatic cleanup, so general-purpose storage types can offer erasure without
  deciding that their contents are secret.
- `ZeroizeOnDrop`, a marker for types that know they hold secrets and clear
  themselves on drop. The marker generates no behavior: implementors write their
  own `Drop` and call `Zeroize::zeroize` there.
- `Zeroizing<T>`, a guard that owns a value, borrows it through `Deref` and
  `DerefMut`, and clears its current contents before dropping them. It
  implements neither `Clone` nor `Copy`.
- Implementations for `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`,
  `i32`, `i64`, `i128`, `isize`, `bool`, and `char`. Arrays and slices visit
  every element and keep their length; `Option<T>` clears a present payload,
  drops it, and becomes `None`; `MaybeUninit<T>` takes a typed volatile zero
  store and stays logically uninitialized.
- A default-off `alloc` feature that adds `Vec<T>` and `Box<T: ?Sized>`,
  including boxed slices, through the sysroot `alloc` crate. `Vec<T>` clears its
  live elements before dropping them, then clears the current allocation
  including spare capacity, retaining capacity and leaving the length at zero.
- Dependency-free, `no_std` builds. Primitive erasure uses
  `core::ptr::write_volatile` followed by `compiler_fence(Ordering::SeqCst)`;
  every unsafe block documents pointer validity and the validity of the
  replacement value. Missing public documentation and unsafe operations outside
  an explicit unsafe block are rejected by crate-level lints.
- Integration tests grouped by `zeroize` and `alloc`, covering every supported
  primitive, empty and nested containers, non-`Copy` elements, slice boundaries,
  clearing before a payload's destructor, guard cleanup on scope exit and on
  unwinding, and, with `alloc`, spare capacity in still-live vector allocations,
  element destruction order, boxed slices, and nested vectors. Doctests cover
  both traits and the guard.

### Compatibility

- Requires Rust 1.85 or later and uses Rust edition 2024.
- Volatile stores and the compiler fence do not flush caches or supply a
  hardware memory barrier, and functional tests establish results rather than
  generated code. Whether a compiler preserves the wipe must be checked for the
  target compiler and hardware.
- Erasure reaches only the storage behind the current mutable borrow. Copies
  left in registers, stack spills, swap, or core dumps are out of reach, as are
  earlier heap buffers left by growth, `shrink_to_fit`, or `into_boxed_slice`.
  Padding bytes are not covered.
- This is not a constant-time API: an option's presence and custom
  implementations may affect control flow. Slice lengths are public.
- Drop-based cleanup requires the destructor to run; forgetting or leaking a
  guard, or aborting the process, bypasses it.
- `String`, `VecDeque`, `BTreeMap`, and `Cow` are not supported.
- Licensed under MIT OR Apache-2.0; both license texts are included in the
  published package.
