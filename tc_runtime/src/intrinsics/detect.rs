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
        arch = $arch:meta,
        feature = $feature:literal,
        env = $env:literal,
        detect = $detect:expr
    ) => {
        #[cfg($arch)]
        static $cache: $crate::intrinsics::detect::Cache =
            $crate::intrinsics::detect::Cache::new();

        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(());

        impl $name {
            /// Detects the capability and returns a proof token when available.
            pub fn detect() -> Option<Self> {
                Self::is_enabled().then_some(Self(()))
            }

            #[cfg($arch)]
            #[doc = concat!(
                "Reports whether the capability is enabled on this processor.\n\n",
                "The `", $feature, "` Cargo feature always returns `false`. With ",
                "this crate's `std` feature enabled, the `", $env, "` environment ",
                "variable has the same effect when it is present before the first ",
                "call.",
            )]
            pub fn is_enabled() -> bool {
                $cache.get_or_init(|| {
                    !cfg!(feature = $feature)
                        && !$crate::intrinsics::detect::disabled_by_env($env)
                        && $detect
                })
            }

            /// Reports whether the capability is enabled on this processor.
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
