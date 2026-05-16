use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

/// Derives a process-global `OnceLock<Self>` and inherent `get()` / `init()`
/// methods on the struct.
///
/// Generates a static named `<UPPER_SNAKE>` of the type stripped of its
/// `HyperParams` suffix. Read via `T::get()`, write once via `T::init(h)`
/// (or `h.init()`). The struct must implement `Default` and `Copy`.
#[proc_macro_derive(HyperParams)]
pub fn derive_hyper_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ty = &input.ident;
    let raw = ty.to_string();
    let short = raw
        .strip_suffix("HyperParams")
        .unwrap_or(&raw)
        .to_uppercase();
    let static_ident = format_ident!("{}", short);
    let expanded = quote! {
        static #static_ident: ::std::sync::OnceLock<#ty> = ::std::sync::OnceLock::new();

        impl #ty {
            /// Returns the active process-global value, lazily initialized
            /// from `Default` if `init` was never called.
            pub fn get() -> &'static Self {
                #static_ident.get_or_init(<Self as ::std::default::Default>::default)
            }

            /// Sets the process-global value. Returns `Err(self)` if already initialized.
            pub fn init(self) -> ::std::result::Result<(), Self> {
                #static_ident.set(self)
            }
        }
    };
    expanded.into()
}
