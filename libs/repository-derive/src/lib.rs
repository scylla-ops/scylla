use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for implementing the Repository trait and new constructor.
///
/// This macro automatically implements:
/// 1. The Repository trait for a struct that has a `base: BaseRepository` field
/// 2. A `new` constructor that takes a DieselPool and creates the struct
///
/// # Example
///
/// ```rust
/// #[derive(Repository)]
/// pub struct MyRepository {
///     base: BaseRepository,
///     // other fields...
/// }
/// ```
#[proc_macro_derive(Repository)]
pub fn derive_repository(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl #name {
            pub fn new(pool: DieselPool) -> Self {
                Self {
                    base: BaseRepository::new(pool),
                }
            }
        }
        impl Repository for #name {
            fn get_pool(&self) -> &DieselPool {
                self.base.get_pool()
            }
        }
    };

    // Convert back to token stream and return
    TokenStream::from(expanded)
}
