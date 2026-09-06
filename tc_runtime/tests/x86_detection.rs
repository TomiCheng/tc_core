//! Hardware-dependent regression checks against Rust's independent detector.
//!
//! Run with default features on both x86 and x86_64. Excluding the crate's
//! `std` feature makes TC_DISABLE_* environment variables irrelevant, without
//! mutating the process environment or racing the capability caches. Each
//! comparison also excludes its Cargo disable switches. Hardware diversity
//! determines which positive and negative probe paths execute.

#![cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "std")))]

macro_rules! compare {
    ($test:ident, $capability:ident, [$($disable:literal),+], $expected:expr) => {
        #[cfg(not(any($(feature = $disable),+)))]
        #[test]
        fn $test() {
            assert_eq!(
                tc_runtime::intrinsics::x86::$capability::is_enabled(),
                $expected,
                stringify!($capability),
            );
        }
    };
}

compare!(
    aes,
    Aes,
    ["disable-x86-aes-ni"],
    std::is_x86_feature_detected!("aes")
);
compare!(
    avx2,
    Avx2,
    ["disable-x86-avx2"],
    std::is_x86_feature_detected!("avx2")
);
compare!(
    bmi1_x64,
    Bmi1X64,
    ["disable-x86-bmi1"],
    cfg!(target_arch = "x86_64") && std::is_x86_feature_detected!("bmi1")
);
compare!(
    bmi2,
    Bmi2,
    ["disable-x86-bmi2"],
    std::is_x86_feature_detected!("bmi2")
);
compare!(
    bmi2_x64,
    Bmi2X64,
    ["disable-x86-bmi2"],
    cfg!(target_arch = "x86_64") && std::is_x86_feature_detected!("bmi2")
);
compare!(
    pclmulqdq,
    Pclmulqdq,
    ["disable-x86-pclmulqdq"],
    std::is_x86_feature_detected!("pclmulqdq")
);
// Do not reduce this expectation to std's "vpclmulqdq" probe: that probe
// reports a raw CPUID bit outside std's AVX OS-state guard. It neither checks
// XCR0 nor requires PCLMULQDQ. Our V256 capability requires PCLMULQDQ, AVX
// state, and VPCLMULQDQ. std's "avx" probe checks both CPUID AVX and the OS's
// XMM/YMM state support (XCR0 & 6 == 6), matching avx_state_enabled(). Without
// that check, a CPU advertising VPCLMULQDQ with AVX state disabled by the OS
// would falsely fail this comparison; executing the instruction is unsafe there.
compare!(
    pclmulqdq_v256,
    PclmulqdqV256,
    ["disable-x86-pclmulqdq", "disable-x86-pclmulqdq-v256"],
    std::is_x86_feature_detected!("pclmulqdq")
        && std::is_x86_feature_detected!("avx")
        && std::is_x86_feature_detected!("vpclmulqdq")
);
// std's "avx512f" probe already includes the required OS-state checks.
compare!(
    pclmulqdq_v512,
    PclmulqdqV512,
    ["disable-x86-pclmulqdq", "disable-x86-pclmulqdq-v512"],
    std::is_x86_feature_detected!("pclmulqdq")
        && std::is_x86_feature_detected!("vpclmulqdq")
        && std::is_x86_feature_detected!("avx512f")
);
compare!(
    sse2,
    Sse2,
    ["disable-x86-sse2"],
    std::is_x86_feature_detected!("sse2")
);
compare!(
    sse41,
    Sse41,
    ["disable-x86-sse41"],
    std::is_x86_feature_detected!("sse4.1")
);
compare!(
    ssse3,
    Ssse3,
    ["disable-x86-ssse3"],
    std::is_x86_feature_detected!("ssse3")
);
