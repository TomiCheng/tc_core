//! `aarch64` CPU-feature detection.
//!
//! The capability types remain available on other architectures so callers can
//! select portable backends without duplicating architecture `cfg` checks, in
//! the same way as [`x86`](super::x86). A successful [`detect`](Aes::detect)
//! call returns a proof token that can be passed to code whose safety contract
//! requires the corresponding feature.
//!
//! Unlike x86, `aarch64` exposes no unprivileged feature-query instruction: the
//! `ID_AA64ISAR*_EL1` registers are readable only at EL1. Detection therefore
//! goes through the operating system, which the `cpufeatures` crate wraps.
//! Detection needs no `std`, because the probe reaches the OS directly.
//!
//! # Platform coverage
//!
//! `cpufeatures` probes Linux and Android through `getauxval(AT_HWCAP)`, and
//! Apple platforms through `sysctlbyname`. Every other operating system,
//! Windows on ARM included, reports each capability as unavailable, so those
//! targets fall back to portable backends even on hardware that supports the
//! instructions. Building with the matching `target_feature` enabled overrides
//! that, because the probe then short-circuits to `true` at compile time.

use super::detect::capability;

// LLVM models `aarch64` target features more coarsely than the individual Arm
// architectural features, so one probe can cover several of them. Each
// capability below documents what its probe actually guarantees.
#[cfg(target_arch = "aarch64")]
cpufeatures::new!(probe_aes, "aes");
#[cfg(target_arch = "aarch64")]
cpufeatures::new!(probe_dit, "dit");
#[cfg(target_arch = "aarch64")]
cpufeatures::new!(probe_sha2, "sha2");
#[cfg(target_arch = "aarch64")]
cpufeatures::new!(probe_sha3, "sha3");
#[cfg(target_arch = "aarch64")]
cpufeatures::new!(probe_sm4, "sm4");

capability! {
    /// Proof that the processor can execute AES and polynomial-multiply
    /// instructions.
    ///
    /// This covers `FEAT_AES` (`AESE`, `AESD`, `AESMC`, `AESIMC`) together with
    /// `FEAT_PMULL` (`PMULL`, `PMULL2`), because Linux requires both HWCAP bits
    /// and the `aes` target feature implies both. There is deliberately no
    /// separate `Pmull` capability, so a GHASH backend takes this token.
    Aes,
    AES_CACHE,
    arch = target_arch = "aarch64",
    feature = "disable-aarch64-aes",
    env = "TC_DISABLE_AARCH64_AES",
    detect = probe_aes::get()
}

capability! {
    /// Proof that the processor supports data-independent timing.
    ///
    /// `FEAT_DIT` guarantees that the instructions it covers take time that
    /// does not depend on their operand values, which constant-time
    /// implementations can rely on once `PSTATE.DIT` is set.
    Dit,
    DIT_CACHE,
    arch = target_arch = "aarch64",
    feature = "disable-aarch64-dit",
    env = "TC_DISABLE_AARCH64_DIT",
    detect = probe_dit::get()
}

capability! {
    /// Proof that the processor can execute Advanced SIMD instructions.
    ///
    /// `FEAT_AdvSIMD` (NEON) is mandatory on `aarch64`, so this needs no
    /// runtime probe and is always available on the architecture, mirroring
    /// SSE2 on x86_64. It exists so a NEON backend can be gated through the
    /// same token API as the rest, and so `disable-aarch64-neon` can turn that
    /// backend off.
    Neon,
    NEON_CACHE,
    arch = target_arch = "aarch64",
    feature = "disable-aarch64-neon",
    env = "TC_DISABLE_AARCH64_NEON",
    detect = true
}

capability! {
    /// Proof that the processor can execute SHA-1 and SHA-256 instructions.
    ///
    /// This covers `FEAT_SHA1` and `FEAT_SHA256`.
    Sha2,
    SHA2_CACHE,
    arch = target_arch = "aarch64",
    feature = "disable-aarch64-sha2",
    env = "TC_DISABLE_AARCH64_SHA2",
    detect = probe_sha2::get()
}

capability! {
    /// Proof that the processor can execute SHA-3 and SHA-512 instructions.
    ///
    /// The `sha3` target feature covers `FEAT_SHA3` (`EOR3`, `RAX1`, `XAR`,
    /// `BCAX`) and `FEAT_SHA512` (`SHA512H`, `SHA512SU0`) together, so a
    /// SHA-512 backend takes this token as well.
    Sha3,
    SHA3_CACHE,
    arch = target_arch = "aarch64",
    feature = "disable-aarch64-sha3",
    env = "TC_DISABLE_AARCH64_SHA3",
    detect = probe_sha3::get()
}

capability! {
    /// Proof that the processor can execute SM3 and SM4 instructions.
    ///
    /// This covers `FEAT_SM3` and `FEAT_SM4`. Apple platforms always report it
    /// as unavailable.
    Sm4,
    SM4_CACHE,
    arch = target_arch = "aarch64",
    feature = "disable-aarch64-sm4",
    env = "TC_DISABLE_AARCH64_SM4",
    detect = probe_sm4::get()
}

#[cfg(test)]
mod tests {
    use super::{Aes, Dit, Neon, Sha2, Sha3, Sm4};

    #[test]
    fn every_token_matches_boolean_detection() {
        assert_eq!(Aes::detect().is_some(), Aes::is_enabled());
        assert_eq!(Dit::detect().is_some(), Dit::is_enabled());
        assert_eq!(Neon::detect().is_some(), Neon::is_enabled());
        assert_eq!(Sha2::detect().is_some(), Sha2::is_enabled());
        assert_eq!(Sha3::detect().is_some(), Sha3::is_enabled());
        assert_eq!(Sm4::detect().is_some(), Sm4::is_enabled());
    }

    #[cfg(all(target_arch = "aarch64", not(feature = "disable-aarch64-neon")))]
    #[test]
    fn aarch64_baseline_supports_neon() {
        assert!(Neon::is_enabled());
    }

    #[cfg(any(
        feature = "disable-aarch64-aes",
        feature = "disable-aarch64-dit",
        feature = "disable-aarch64-neon",
        feature = "disable-aarch64-sha2",
        feature = "disable-aarch64-sha3",
        feature = "disable-aarch64-sm4"
    ))]
    #[test]
    fn configured_cargo_features_disable_their_capabilities() {
        #[cfg(feature = "disable-aarch64-aes")]
        {
            assert!(!Aes::is_enabled());
            assert_eq!(None, Aes::detect());
        }
        #[cfg(feature = "disable-aarch64-dit")]
        assert!(!Dit::is_enabled());
        #[cfg(feature = "disable-aarch64-neon")]
        assert!(!Neon::is_enabled());
        #[cfg(feature = "disable-aarch64-sha2")]
        assert!(!Sha2::is_enabled());
        #[cfg(feature = "disable-aarch64-sha3")]
        assert!(!Sha3::is_enabled());
        #[cfg(feature = "disable-aarch64-sm4")]
        assert!(!Sm4::is_enabled());
    }

    #[cfg(all(feature = "std", target_arch = "aarch64"))]
    #[test]
    fn runtime_environment_overrides_disable_every_capability() {
        const CHILD_MARKER: &str = "TC_RUNTIME_AARCH64_ENV_TEST_CHILD";
        const TEST_NAME: &str =
            "intrinsics::aarch64::tests::runtime_environment_overrides_disable_every_capability";
        const DISABLE_VARIABLES: &[&str] = &[
            "TC_DISABLE_AARCH64_AES",
            "TC_DISABLE_AARCH64_DIT",
            "TC_DISABLE_AARCH64_NEON",
            "TC_DISABLE_AARCH64_SHA2",
            "TC_DISABLE_AARCH64_SHA3",
            "TC_DISABLE_AARCH64_SM4",
        ];

        if std::env::var_os(CHILD_MARKER).is_some() {
            assert!(!Aes::is_enabled());
            assert!(!Dit::is_enabled());
            assert!(!Neon::is_enabled());
            assert!(!Sha2::is_enabled());
            assert!(!Sha3::is_enabled());
            assert!(!Sm4::is_enabled());
            return;
        }

        let mut child = std::process::Command::new(
            std::env::current_exe().expect("current test executable should be available"),
        );
        child
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .env(CHILD_MARKER, "1");
        for variable in DISABLE_VARIABLES {
            child.env(variable, "1");
        }

        let status = child.status().expect("environment test child should run");
        assert!(status.success());
    }

    #[cfg(not(target_arch = "aarch64"))]
    #[test]
    fn non_aarch64_targets_report_every_capability_as_unavailable() {
        assert!(!Aes::is_enabled());
        assert!(!Dit::is_enabled());
        assert!(!Neon::is_enabled());
        assert!(!Sha2::is_enabled());
        assert!(!Sha3::is_enabled());
        assert!(!Sm4::is_enabled());
    }
}
