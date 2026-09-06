//! Shared machinery behind the per-architecture capability modules.
//!
//! [`capability!`] is available on every target because each architecture
//! module defines its capability types unconditionally, so callers can select a
//! backend without duplicating architecture `cfg` checks. Only the runtime
//! helpers those types call are restricted to architectures this crate can
//! actually probe.

/// Declares a capability type backed by a cached runtime probe.
///
/// `arch` is the `cfg` predicate naming the architectures that can run
/// `detect`. Off those architectures the generated `is_enabled` is a `const fn`
/// returning `false`, so the caller's branch folds away at compile time.
macro_rules! capability {
    (
        $(#[$meta:meta])*
        $name:ident,
        $cache:ident,
        module = $module:ident,
        arch = $arch:meta,
        feature = $feature:literal,
        env = $env:literal,
        detect = $detect:expr
    ) => {
        #[cfg($arch)]
        static $cache: $crate::intrinsics::detect::Cache =
            $crate::intrinsics::detect::Cache::new();

        $(#[$meta])*
        #[doc = concat!(
            "\n\nObtain this token with [`Self::detect`]. It is `Copy`, so it can be ",
            "passed to multiple backend calls. It does not enable compiler target ",
            "features or validate any memory-safety requirements of an intrinsic.\n\n",
            "Safe callers cannot construct the token directly:\n\n",
            "```compile_fail,E0423\n",
            "use tc_runtime::intrinsics::", stringify!($module), "::", stringify!($name), ";\n",
            "let token = ", stringify!($name), "(());\n```",
        )]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(());

        impl $name {
            /// Detects the capability and returns a proof token when available.
            ///
            /// Returns `None` on other architectures, when detection cannot
            /// establish support, or when the capability is disabled. Use a
            /// portable fallback in those cases rather than unwrapping.
            #[doc = concat!(
                "\n\n# Examples\n\n```\n",
                "use tc_runtime::intrinsics::", stringify!($module), "::", stringify!($name), ";\n",
                "fn select_backend(_proof: ", stringify!($name), ") -> &'static str {\n",
                "    // A backend can require the token in its public signature.\n",
                "    \"optimized\"\n}\n",
                "let backend = match ", stringify!($name), "::detect() {\n",
                "    Some(proof) => select_backend(proof),\n",
                "    None => \"portable\",\n",
                "};\n",
                "assert_eq!(backend == \"optimized\", ", stringify!($name), "::is_enabled());\n```",
            )]
            pub fn detect() -> Option<Self> {
                Self::is_enabled().then_some(Self(()))
            }

            #[cfg($arch)]
            #[doc = concat!(
                "Reports whether the capability is enabled on this processor.\n\n",
                "The `", $feature, "` Cargo feature always returns `false`. With ",
                "this crate's `std` feature enabled, the `", $env, "` environment ",
                "variable has the same effect when it is present before the first ",
                "call. Results are cached; configure overrides before detection starts. ",
                "Disabling detection does not prevent the compiler from emitting ",
                "instructions enabled by the build's target features.\n\n",
                "Use [`Self::detect`] when a backend needs a proof token.\n\n",
                "# Examples\n\n```\n",
                "use tc_runtime::intrinsics::", stringify!($module), "::", stringify!($name), ";\n",
                "assert_eq!(", stringify!($name), "::is_enabled(), ", stringify!($name), "::detect().is_some());\n",
                "#[cfg(feature = ", stringify!($feature), ")]\n",
                "assert!(!", stringify!($name), "::is_enabled());\n```",
            )]
            pub fn is_enabled() -> bool {
                $cache.get_or_init(|| {
                    !cfg!(feature = $feature)
                        && !$crate::intrinsics::detect::disabled_by_env($env)
                        && $detect
                })
            }

            /// Reports whether the capability is enabled on this processor.
            ///
            /// Always returns `false` on this architecture. This method is
            /// `const` here, so it can also be used in constant expressions.
            #[doc = concat!(
                "\n\n# Examples\n\n```\n",
                "use tc_runtime::intrinsics::", stringify!($module), "::", stringify!($name), ";\n",
                "const AVAILABLE: bool = ", stringify!($name), "::is_enabled();\n",
                "assert!(!AVAILABLE);\n```",
            )]
            #[cfg(not($arch))]
            pub const fn is_enabled() -> bool {
                false
            }
        }
    };
}

pub(crate) use capability;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
pub(crate) use probe::{Cache, disabled_by_env};

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
mod probe {
    use core::sync::atomic::{AtomicU8, Ordering};

    const UNKNOWN: u8 = 0;
    const DISABLED: u8 = 1;
    const ENABLED: u8 = 2;

    /// Caches one capability probe for the lifetime of the process.
    pub(crate) struct Cache(AtomicU8);

    impl Cache {
        pub(crate) const fn new() -> Self {
            Self(AtomicU8::new(UNKNOWN))
        }

        /// Returns the cached answer, running `detect` on the first call.
        ///
        /// Racing callers may both run `detect`; it is pure, so they agree.
        pub(crate) fn get_or_init(&self, detect: impl FnOnce() -> bool) -> bool {
            match self.0.load(Ordering::Relaxed) {
                ENABLED => true,
                DISABLED => false,
                UNKNOWN => {
                    let enabled = detect();
                    self.0
                        .store(if enabled { ENABLED } else { DISABLED }, Ordering::Relaxed);
                    enabled
                }
                _ => unreachable!(),
            }
        }
    }

    #[cfg(feature = "std")]
    pub(crate) fn disabled_by_env(name: &str) -> bool {
        std::env::var_os(name).is_some()
    }

    #[cfg(not(feature = "std"))]
    pub(crate) const fn disabled_by_env(_name: &str) -> bool {
        false
    }
}
