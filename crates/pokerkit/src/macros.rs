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

/// Implements `tokio_postgres` SQL codecs (ToSql/FromSql) for a domain type
/// by delegating to a primitive representation. Every path inside is fully
/// qualified, so call sites need no imports:
///
/// ```ignore
/// #[cfg(feature = "sql")]
/// pokerkit::codec!(Edge as i64, |e| u64::from(*e) as i64, |v| Edge::from(v as u64));
/// ```
#[macro_export]
macro_rules! codec {
    ($ty:ty as $prim:ty, $to:expr, $from:expr) => {
        impl ::tokio_postgres::types::ToSql for $ty {
            fn to_sql(
                &self,
                ty: &::tokio_postgres::types::Type,
                out: &mut ::tokio_postgres::types::private::BytesMut,
            ) -> Result<::tokio_postgres::types::IsNull, Box<dyn ::std::error::Error + Sync + Send>> {
                ::tokio_postgres::types::ToSql::to_sql(&($to)(self), ty, out)
            }

            fn accepts(ty: &::tokio_postgres::types::Type) -> bool {
                <$prim as ::tokio_postgres::types::ToSql>::accepts(ty)
            }

            ::tokio_postgres::types::to_sql_checked!();
        }

        impl<'a> ::tokio_postgres::types::FromSql<'a> for $ty {
            fn from_sql(
                ty: &::tokio_postgres::types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn ::std::error::Error + Sync + Send>> {
                <$prim as ::tokio_postgres::types::FromSql>::from_sql(ty, raw).map($from)
            }

            fn accepts(ty: &::tokio_postgres::types::Type) -> bool {
                <$prim as ::tokio_postgres::types::FromSql>::accepts(ty)
            }
        }
    };
}
