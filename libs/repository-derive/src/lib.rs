use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for implementing the Repository trait.
/// 
/// This macro automatically implements the Repository trait for a struct
/// that has a `base: BaseRepository` field. It delegates the `get_pool` method
/// to the base repository.
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
        impl crate::api::v1::common::base::Repository for #name {
            fn get_pool(&self) -> &crate::database::DieselPool {
                self.base.get_pool()
            }
        }
    };

    // Convert back to token stream and return
    TokenStream::from(expanded)
}
