//! Declarative macros (folded in from the former `solus` proc-macro crate).

/// Declare a process-global singleton for a hyperparameter struct.
///
/// The struct must implement [`Default`]. `T::get()` returns the active value,
/// lazily initialized from `Default`; `T::init(v)` sets it once, returning
/// `Err(v)` if it was already set. Replaces the former `#[derive(HyperParams)]`.
///
/// ```ignore
/// #[derive(Clone, Copy, Default)]
/// struct FooHyperParams { threshold: f32 }
/// pokerkit::hyperparams!(FooHyperParams);
/// // FooHyperParams::get(); FooHyperParams { threshold: 0.5 }.init();
/// ```
#[macro_export]
macro_rules! hyperparams {
    ($ty:ty) => {
        impl $ty {
            fn cell() -> &'static ::std::sync::OnceLock<$ty> {
                static CELL: ::std::sync::OnceLock<$ty> = ::std::sync::OnceLock::new();
                &CELL
            }
            /// Active process-global value; lazily `Default`-initialized.
            pub fn get() -> &'static Self {
                Self::cell().get_or_init(<Self as ::std::default::Default>::default)
            }
            /// Set the process-global value once; `Err(self)` if already set.
            pub fn init(self) -> ::std::result::Result<(), Self> {
                Self::cell().set(self)
            }
        }
    };
}
