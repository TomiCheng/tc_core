# tc_runtime

`tc_runtime` provides CPU feature detection and capability proof tokens for x86
and AArch64. Callers use these tokens to select supported optimized backends.
It is `no_std` unless its optional `std` feature is enabled, and supports Rust
1.85 and later.

It has no dependencies by default, on any target. Opting into aarch64 runtime
detection with `aarch64-detect` adds
[`cpufeatures`](https://crates.io/crates/cpufeatures), which in turn pulls in
`libc` on Linux, Android, and Apple targets only. Neither requires `std`.

The x86 API mirrors the capabilities currently queried by Bouncy Castle's
`Org.BouncyCastle.Runtime.Intrinsics.X86` namespace. Every capability provides:

```rust
use tc_runtime::intrinsics::x86::{Aes, Avx2, Sse2};

if let Some(sse2) = Sse2::detect() {
    // Pass the proof token to an SSE2 backend.
    let _ = sse2;
}

assert_eq!(Aes::detect().is_some(), Aes::is_enabled());
assert_eq!(Avx2::detect().is_some(), Avx2::is_enabled());
```

The private field in each proof token prevents safe caller code from creating
one without first performing detection.

The types remain available on every architecture, where `is_enabled()` returns
`false` and `detect()` returns `None`. Off its own architecture a capability's
`is_enabled()` is a `const fn`, so the caller's branch folds away at compile
time and a portable selection costs nothing:

```rust
use tc_runtime::intrinsics::{aarch64, x86};

let backend = if x86::Pclmulqdq::detect().is_some() {
    "pclmulqdq"
} else if aarch64::Aes::detect().is_some() {
    "pmull"
} else {
    "portable"
};
```

## x86 capabilities

| Bouncy Castle capability | Rust capability | Detection |
| --- | --- | --- |
| `Aes` | `Aes` (`AesNi` alias) | CPUID AES-NI |
| `Avx2` | `Avx2` | CPUID AVX/AVX2 and OS XMM/YMM state |
| `Bmi1.X64` | `bmi1::X64` / `Bmi1X64` | 64-bit process and CPUID BMI1 |
| `Bmi2` | `Bmi2` | CPUID BMI2 |
| `Bmi2.X64` | `bmi2::X64` / `Bmi2X64` | 64-bit process and CPUID BMI2 |
| `Pclmulqdq` | `Pclmulqdq` | CPUID PCLMULQDQ |
| `Pclmulqdq.V256` | `pclmulqdq::V256` / `PclmulqdqV256` | PCLMULQDQ, VPCLMULQDQ and OS XMM/YMM state |
| `Pclmulqdq.V512` | `pclmulqdq::V512` / `PclmulqdqV512` | PCLMULQDQ, VPCLMULQDQ, AVX-512F and OS ZMM state |
| `Sse2` | `Sse2` | x86_64 baseline or CPUID SSE2 on x86 |
| `Sse41` | `Sse41` | CPUID SSE4.1 |
| `Ssse3` | `Ssse3` | CPUID SSSE3 |

AVX2 and the vector-width PCLMULQDQ checks include operating-system
extended-state support; a CPU feature bit alone is not sufficient to execute
those instructions safely.

## aarch64 capabilities

aarch64 exposes no unprivileged feature-query instruction — the
`ID_AA64ISAR*_EL1` registers are readable only at EL1 — so runtime detection
goes through the operating system rather than an equivalent of CPUID.

LLVM models aarch64 target features more coarsely than the individual Arm
architectural features, so one capability can cover several of them:

| Rust capability | Arm features | Detection |
| --- | --- | --- |
| `Aes` | `FEAT_AES` **and** `FEAT_PMULL` | `aes` target feature |
| `Dit` | `FEAT_DIT` | `dit` target feature |
| `Neon` | `FEAT_AdvSIMD` | architecture baseline, no probe |
| `Sha2` | `FEAT_SHA1`, `FEAT_SHA256` | `sha2` target feature |
| `Sha3` | `FEAT_SHA3`, `FEAT_SHA512` | `sha3` target feature |
| `Sm4` | `FEAT_SM3`, `FEAT_SM4` | `sm4` target feature |

There is deliberately no separate `Pmull` capability: Linux requires both the
`HWCAP_AES` and `HWCAP_PMULL` bits, so a GHASH backend takes the `Aes` token.
For the same reason a SHA-512 backend takes the `Sha3` token.

### Detection backends

A capability is available whenever the matching `target_feature` is enabled at
compile time, because the compiler may then emit those instructions throughout
the build. This floor applies whichever backend is selected, and costs nothing
at run time. Several targets set most of these by default:

| Target | Default `target_feature` |
| --- | --- |
| `aarch64-apple-darwin` | `aes`, `dit`, `neon`, `sha2`, `sha3`, and more |
| `aarch64-unknown-linux-gnu` | `neon` |
| `aarch64-unknown-none` | `neon` |
| `aarch64-pc-windows-msvc` | `neon` |

Runtime detection beyond that floor is a swappable backend:

| Backend | Cargo feature | Covers |
| --- | --- | --- |
| none (default) | — | nothing beyond the floor; no dependencies |
| `cpufeatures` | `aarch64-detect` | Linux and Android via `getauxval(AT_HWCAP)`, Apple platforms via `sysctlbyname` |

**No backend can probe Windows on ARM or bare-metal targets**, which therefore
rely on the compile-time floor alone; `aarch64-detect` buys them nothing. On
`aarch64-apple-darwin` the floor already covers the features the backend would
report, so it changes nothing there either. In practice the feature matters on
Linux and Android.

Unlike the x86 module's use of CPUID, none of this requires `std`; a backend
reaches the operating system directly.

Backends live in `src/intrinsics/aarch64/backend.rs`, behind a contract of one
`fn() -> bool` per feature. Replacing one does not affect the public API.

## Cargo features

| Feature | Effect |
| --- | --- |
| `std` | Enables the `TC_DISABLE_*` environment-variable overrides |
| `aarch64-detect` | Selects a runtime-detection backend for aarch64 |
| `disable-*` | Turns individual capabilities off; see below |

## Disabling optimized backends

Each instruction-set backend can be disabled at compile time:

| Capability | Cargo feature | Runtime environment variable with `std` |
| --- | --- | --- |
| AES-NI | `disable-x86-aes-ni` | `TC_DISABLE_X86_AES_NI` |
| AVX2 | `disable-x86-avx2` | `TC_DISABLE_X86_AVX2` |
| BMI1 | `disable-x86-bmi1` | `TC_DISABLE_X86_BMI1` |
| BMI2 | `disable-x86-bmi2` | `TC_DISABLE_X86_BMI2` |
| PCLMULQDQ, all widths | `disable-x86-pclmulqdq` | `TC_DISABLE_X86_PCLMULQDQ` |
| PCLMULQDQ 256-bit only | `disable-x86-pclmulqdq-v256` | `TC_DISABLE_X86_PCLMULQDQ_V256` |
| PCLMULQDQ 512-bit only | `disable-x86-pclmulqdq-v512` | `TC_DISABLE_X86_PCLMULQDQ_V512` |
| SSE2 | `disable-x86-sse2` | `TC_DISABLE_X86_SSE2` |
| SSE4.1 | `disable-x86-sse41` | `TC_DISABLE_X86_SSE41` |
| SSSE3 | `disable-x86-ssse3` | `TC_DISABLE_X86_SSSE3` |
| Arm AES and PMULL | `disable-aarch64-aes` | `TC_DISABLE_AARCH64_AES` |
| Arm DIT | `disable-aarch64-dit` | `TC_DISABLE_AARCH64_DIT` |
| Arm NEON | `disable-aarch64-neon` | `TC_DISABLE_AARCH64_NEON` |
| Arm SHA-1 and SHA-256 | `disable-aarch64-sha2` | `TC_DISABLE_AARCH64_SHA2` |
| Arm SHA-3 and SHA-512 | `disable-aarch64-sha3` | `TC_DISABLE_AARCH64_SHA3` |
| Arm SM3 and SM4 | `disable-aarch64-sm4` | `TC_DISABLE_AARCH64_SM4` |

For example:

```text
cargo build --features tc_runtime/disable-x86-avx2
cargo build --features tc_runtime/disable-x86-pclmulqdq
cargo build --features tc_runtime/disable-aarch64-aes
```

Runtime environment variables are read only when the `std` feature is enabled:

```text
TC_DISABLE_X86_AVX2=1
TC_DISABLE_X86_PCLMULQDQ=1
TC_DISABLE_AARCH64_AES=1
```

The presence of a variable disables the matching capability regardless of its
value. Results are cached, so changing an environment variable after the first
check has no effect. The general PCLMULQDQ switch also disables its V256 and
V512 capabilities.

These switches prevent callers from selecting the matching optimized backend;
they do not guarantee that the Rust compiler emits no instructions belonging
to that instruction set. In particular, SSE2 is part of the x86_64 baseline and
NEON is part of the aarch64 baseline.
