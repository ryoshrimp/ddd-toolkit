use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Result, parse_quote};

use crate::path::core_path;

/// **English:**<br>
/// Generates identity access, equality, and hashing from one marked
/// named field; rejects invalid shapes, misplaced attributes, or ambiguous identity.
///
/// **日本語:**<br>
/// 指定された一つの名前付きフィールドから識別子の取得・比較・ハッシュを
/// 生成し、不正な形状、属性の誤配置、識別子の曖昧さをエラーにします。
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let mut trait_only = false;
    let mut validator: Option<syn::Path> = None;
    let mut error: Option<syn::Type> = None;
    let mut serde = false;
    for attr in &input.attrs {
        if !attr.path().is_ident("entity") {
            continue;
        }
        let mut option_found = false;
        attr.parse_nested_meta(|meta| {
            option_found = true;
            if meta.path.is_ident("validate") {
                if validator.is_some() {
                    return Err(meta.error("duplicate validate attribute"));
                }
                validator = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("error") {
                if error.is_some() {
                    return Err(meta.error("duplicate error attribute"));
                }
                error = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("serde") {
                if !meta.input.is_empty() && !meta.input.peek(syn::Token![,]) {
                    return Err(meta.error("serde is a flag and takes no value"));
                }
                if serde {
                    return Err(meta.error("duplicate serde attribute"));
                }
                serde = true;
                return Ok(());
            }
            if !meta.path.is_ident("trait_only") {
                return Err(meta.error("entity(id) must be placed on a named field"));
            }
            if !meta.input.is_empty() && !meta.input.peek(syn::Token![,]) {
                return Err(meta.error("trait_only is a flag and takes no value"));
            }
            if trait_only {
                return Err(meta.error("duplicate trait_only attribute"));
            }
            trait_only = true;
            Ok(())
        })?;
        if !option_found {
            return Err(syn::Error::new_spanned(
                attr,
                "entity requires at least one option",
            ));
        }
    }

    if trait_only && (validator.is_some() || error.is_some() || serde) {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "trait_only cannot be combined with validate, error, or serde",
        ));
    }
    if validator.is_some() != error.is_some() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "validate and error must be specified together",
        ));
    }

    crate::serde_attributes::validate(&input, serde, false)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "Entity requires a named-field struct",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Entity can only be derived for a named-field struct",
            ));
        }
    };

    let mut id_field = None;
    for field in fields {
        if has_id_attribute(field)? {
            if id_field.is_some() {
                return Err(syn::Error::new_spanned(
                    field,
                    "Entity requires exactly one #[entity(id)] field",
                ));
            }
            id_field = Some(field);
        }
    }
    let id_field = id_field.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "Entity requires exactly one #[entity(id)] field",
        )
    })?;
    let id_name = id_field.ident.as_ref().expect("named fields have names");
    let id_ty = &id_field.ty;
    let core = core_path()?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let trait_impl = quote! {
        impl #impl_generics #core::Entity for #name #ty_generics #where_clause {
            type Id = #id_ty;

            fn id(&self) -> &Self::Id {
                &self.#id_name
            }
        }
    };
    if trait_only {
        return Ok(trait_impl);
    }

    if validator.is_some()
        && let Some(field) = fields
            .iter()
            .find(|field| matches!(field.vis, syn::Visibility::Public(_)))
    {
        return Err(syn::Error::new_spanned(
            field,
            "validated Entity fields must not be public",
        ));
    }
    let arguments: Vec<_> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();
    let constructor = match (&validator, &error) {
        (Some(validator), Some(error)) => Some(quote! {
            /// **English:**<br>
            /// Builds the supplied state in declaration order and validates it.
            /// Propagates validator errors and panics; does not generate an ID or events.
            ///
            /// **日本語:**<br>
            /// 宣言順に指定された状態を構築して検証します。検証のエラーとパニックを
            /// 伝播し、ID の生成やイベントの発生は行いません。
            pub fn try_new(#(#arguments: #types),*) -> ::core::result::Result<Self, #error> {
                let candidate = Self { #(#arguments),* };
                #validator(&candidate)?;
                Ok(candidate)
            }
        }),
        _ => None,
    };
    let serde_impl = if serde {
        let members: Vec<_> = arguments
            .iter()
            .map(|ident| syn::Member::Named((*ident).clone()))
            .collect();
        let Data::Struct(data) = &input.data else {
            unreachable!()
        };
        Some(crate::value_object::serde_impl_multi(
            name,
            &input.generics,
            &data.fields,
            &members,
            &types,
            true,
            validator.is_some(),
            &core,
            "entity validation failed",
            &input.attrs,
        )?)
    } else {
        None
    };

    let mut partial_eq_generics = input.generics.clone();
    partial_eq_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#id_ty: ::core::cmp::PartialEq));
    let (partial_eq_impl, partial_eq_ty, partial_eq_where) = partial_eq_generics.split_for_impl();

    let mut eq_generics = input.generics.clone();
    eq_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#id_ty: ::core::cmp::Eq));
    let (eq_impl, eq_ty, eq_where) = eq_generics.split_for_impl();

    let mut hash_generics = input.generics.clone();
    hash_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#id_ty: ::core::hash::Hash));
    let (hash_impl, hash_ty, hash_where) = hash_generics.split_for_impl();

    Ok(quote! {
        #trait_impl
        #serde_impl

        impl #impl_generics #name #ty_generics #where_clause {
            /// **English:**<br>
            /// Borrows the field marked as this entity's identifier.
            ///
            /// **日本語:**<br>
            /// このエンティティの識別子に指定されたフィールドを借用します。
            pub fn id(&self) -> &#id_ty {
                &self.#id_name
            }

            #constructor
        }

        impl #partial_eq_impl ::core::cmp::PartialEq for #name #partial_eq_ty #partial_eq_where {
            fn eq(&self, other: &Self) -> bool {
                self.#id_name == other.#id_name
            }
        }
        impl #eq_impl ::core::cmp::Eq for #name #eq_ty #eq_where {}
        impl #hash_impl ::core::hash::Hash for #name #hash_ty #hash_where {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                ::core::hash::Hash::hash(&self.#id_name, state);
            }
        }
    })
}

fn has_id_attribute(field: &syn::Field) -> Result<bool> {
    let mut found = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("entity") {
            continue;
        }
        let mut option_found = false;
        attr.parse_nested_meta(|meta| {
            option_found = true;
            if !meta.path.is_ident("id") {
                return Err(meta.error("unknown entity attribute"));
            }
            if !meta.input.is_empty() {
                return Err(meta.error("id is a flag and takes no value"));
            }
            if found {
                return Err(meta.error("duplicate id attribute"));
            }
            found = true;
            Ok(())
        })?;
        if !option_found {
            return Err(syn::Error::new_spanned(attr, "entity requires the id flag"));
        }
    }
    Ok(found)
}
