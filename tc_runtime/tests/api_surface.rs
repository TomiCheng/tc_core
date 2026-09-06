//! Locks the publicly reachable API surface in place.
//!
//! Integration tests link `tc_runtime` as an external crate, so this file stops
//! compiling if a public item is renamed, removed, or has its signature changed.
//! Function-pointer coercions additionally pin each compatibility alias to the
//! exact type it aliases, which a value comparison alone would not catch.

use tc_runtime::intrinsics::aarch64::{Aes as ArmAes, Dit, Neon, Sha2, Sha3, Sm4};
use tc_runtime::intrinsics::x86::{
    Aes, AesNi, Avx2, Bmi1X64, Bmi2, Bmi2X64, Pclmulqdq, PclmulqdqV256, PclmulqdqV512, Sse2, Sse41,
    Ssse3, bmi1, bmi2, pclmulqdq,
};

/// Fails to compile unless the capability keeps every derived trait.
fn assert_traits<T: Copy + Clone + core::fmt::Debug + PartialEq + Eq>() {}

#[test]
fn every_x86_capability_keeps_its_signatures() {
    macro_rules! pin {
        ($($ty:ty),+ $(,)?) => {$(
            let _: fn() -> Option<$ty> = <$ty>::detect;
            let _: fn() -> bool = <$ty>::is_enabled;
            assert_traits::<$ty>();
        )+};
    }

    pin!(
        Aes,
        Avx2,
        Bmi1X64,
        Bmi2,
        Bmi2X64,
        Pclmulqdq,
        PclmulqdqV256,
        PclmulqdqV512,
        Sse2,
        Sse41,
        Ssse3,
    );
}

#[test]
fn compatibility_aliases_stay_bound_to_the_same_types() {
    // Coercing to `fn() -> Option<Aes>` only type-checks while `AesNi` *is* `Aes`.
    let _: fn() -> Option<Aes> = AesNi::detect;
    let _: fn() -> Option<Bmi1X64> = bmi1::X64::detect;
    let _: fn() -> Option<Bmi2X64> = bmi2::X64::detect;
    let _: fn() -> Option<PclmulqdqV256> = pclmulqdq::V256::detect;
    let _: fn() -> Option<PclmulqdqV512> = pclmulqdq::V512::detect;
}

#[test]
fn detect_agrees_with_is_enabled() {
    assert_eq!(Aes::detect().is_some(), Aes::is_enabled());
    assert_eq!(Avx2::detect().is_some(), Avx2::is_enabled());
    assert_eq!(Bmi1X64::detect().is_some(), Bmi1X64::is_enabled());
    assert_eq!(Bmi2::detect().is_some(), Bmi2::is_enabled());
    assert_eq!(Bmi2X64::detect().is_some(), Bmi2X64::is_enabled());
    assert_eq!(Pclmulqdq::detect().is_some(), Pclmulqdq::is_enabled());
    assert_eq!(
        PclmulqdqV256::detect().is_some(),
        PclmulqdqV256::is_enabled()
    );
    assert_eq!(
        PclmulqdqV512::detect().is_some(),
        PclmulqdqV512::is_enabled()
    );
    assert_eq!(Sse2::detect().is_some(), Sse2::is_enabled());
    assert_eq!(Sse41::detect().is_some(), Sse41::is_enabled());
    assert_eq!(Ssse3::detect().is_some(), Ssse3::is_enabled());
}

#[cfg(target_arch = "x86")]
#[test]
fn x64_only_tokens_are_unavailable_in_a_32_bit_process() {
    assert_eq!(core::mem::size_of::<usize>(), 4);
    assert!(!Bmi1X64::is_enabled());
    assert!(!Bmi2X64::is_enabled());
    assert_eq!(Bmi1X64::detect(), None);
    assert_eq!(Bmi2X64::detect(), None);
}

// With no overrides, compare the 32-bit CPUID path against Rust's detector.
#[cfg(all(
    target_arch = "x86",
    not(feature = "std"),
    not(feature = "disable-x86-sse2")
))]
#[test]
fn x86_sse2_matches_standard_library_detection() {
    assert_eq!(Sse2::is_enabled(), std::is_x86_feature_detected!("sse2"));
}

/// `is_enabled` is a `const fn` off x86, so callers can branch at compile time.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[test]
fn non_x86_capabilities_fold_away_at_compile_time() {
    const _: () = assert!(!Sse2::is_enabled());
    const _: () = assert!(!Aes::is_enabled());
}

#[test]
fn every_aarch64_capability_keeps_its_signatures() {
    macro_rules! pin {
        ($($ty:ty),+ $(,)?) => {$(
            let _: fn() -> Option<$ty> = <$ty>::detect;
            let _: fn() -> bool = <$ty>::is_enabled;
            assert_traits::<$ty>();
            assert_eq!(<$ty>::detect().is_some(), <$ty>::is_enabled());
        )+};
    }

    pin!(ArmAes, Dit, Neon, Sha2, Sha3, Sm4);
}

/// The two architecture modules are independent: each keeps its own `Aes`.
#[test]
fn architecture_modules_do_not_share_capability_types() {
    fn only_x86(_: fn() -> Option<Aes>) {}
    fn only_aarch64(_: fn() -> Option<ArmAes>) {}

    only_x86(Aes::detect);
    only_aarch64(ArmAes::detect);
}

#[cfg(not(target_arch = "aarch64"))]
#[test]
fn non_aarch64_capabilities_fold_away_at_compile_time() {
    const _: () = assert!(!Neon::is_enabled());
    const _: () = assert!(!ArmAes::is_enabled());
}
