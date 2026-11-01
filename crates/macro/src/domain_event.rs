use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Result};

use crate::path::core_path;

/// **English:** Emits a marker implementation for structs and enums, returning
/// a syntax error for unions or an unresolved core dependency.
///
/// **日本語:** 構造体と列挙型のマーカー実装を生成し、共用体やコア依存を
/// 解決できない場合は構文エラーを返します。
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(_) | Data::Enum(_) => {}
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "DomainEvent can only be derived for a struct or enum",
            ));
        }
    }

    let core = core_path()?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #core::DomainEvent for #name #ty_generics #where_clause {}
    })
}
