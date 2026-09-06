//! Swappable runtime-detection backends for [`super`].
//!
//! # Backend contract
//!
//! A backend is a private `imp` module providing one `fn() -> bool` per
//! probeable feature, named after the LLVM target feature it answers for:
//! [`aes`], [`dit`], [`sha2`], [`sha3`], and [`sm4`]. A backend answers only
//! the *runtime* half of the question and needs to do nothing else:
//!
//! - **No caching.** Each function is called at most once per capability,
//!   because [`Cache`](crate::intrinsics::detect) memoises the answer.
//! - **No compile-time check.** The wrappers below already fall back on
//!   `cfg!(target_feature = ...)`, so a backend never has to repeat it and can
//!   never report less than the compiler already guarantees.
//! - **No `std`.** Backends reach the operating system directly.
//!
//! Replacing the backend therefore touches this file only: [`super`] declares
//! the capabilities, and the public API does not mention a backend at all.
//!
//! # Selecting a backend
//!
//! `aarch64-detect` selects the `cpufeatures` backend. Without it the `none`
//! backend leaves every capability to the compile-time floor, which keeps the
//! crate dependency-free.

/// Wraps a backend probe in the compile-time floor.
///
/// `cfg!(target_feature = ...)` being true means the compiler may already emit
/// the instructions throughout this build, so the capability is available
/// whatever the backend reports.
macro_rules! probe {
    ($( $(#[$meta:meta])* $name:ident = $target_feature:tt );+ $(;)?) => {$(
        $(#[$meta])*
        pub(super) fn $name() -> bool {
            cfg!(target_feature = $target_feature) || imp::$name()
        }
    )+};
}

probe! {
    /// `FEAT_AES` together with `FEAT_PMULL`.
    aes = "aes";
    /// `FEAT_DIT`.
    dit = "dit";
    /// `FEAT_SHA1` and `FEAT_SHA256`.
    sha2 = "sha2";
    /// `FEAT_SHA3` and `FEAT_SHA512`.
    sha3 = "sha3";
    /// `FEAT_SM3` and `FEAT_SM4`.
    sm4 = "sm4";
}

/// Runtime detection through the [`cpufeatures`] crate.
///
/// Covers Linux and Android through `getauxval(AT_HWCAP)`, and Apple platforms
/// through `sysctlbyname`. Every other operating system reports `false`, which
/// the compile-time floor may still override.
#[cfg(feature = "aarch64-detect")]
mod imp {
    cpufeatures::new!(probe_aes, "aes");
    cpufeatures::new!(probe_dit, "dit");
    cpufeatures::new!(probe_sha2, "sha2");
    cpufeatures::new!(probe_sha3, "sha3");
    cpufeatures::new!(probe_sm4, "sm4");

    pub(super) fn aes() -> bool {
        probe_aes::get()
    }

    pub(super) fn dit() -> bool {
        probe_dit::get()
    }

    pub(super) fn sha2() -> bool {
        probe_sha2::get()
    }

    pub(super) fn sha3() -> bool {
        probe_sha3::get()
    }

    pub(super) fn sm4() -> bool {
        probe_sm4::get()
    }
}

/// No runtime detection, leaving every capability to the compile-time floor.
///
/// This is the dependency-free default. It loses nothing on targets whose
/// default `target_feature` set already covers these features, such as
/// `aarch64-apple-darwin`, nor on targets no backend can probe anyway, such as
/// `aarch64-unknown-none` and Windows on ARM.
#[cfg(not(feature = "aarch64-detect"))]
mod imp {
    pub(super) const fn aes() -> bool {
        false
    }

    pub(super) const fn dit() -> bool {
        false
    }

    pub(super) const fn sha2() -> bool {
        false
    }

    pub(super) const fn sha3() -> bool {
        false
    }

    pub(super) const fn sm4() -> bool {
        false
    }
}
