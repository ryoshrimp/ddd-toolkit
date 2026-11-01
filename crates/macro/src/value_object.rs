use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Field, Fields, GenericParam, Generics, Lifetime, LitStr, Member, Result,
    Type, parse_quote,
};

use crate::path::core_path;

struct Attributes {
    validator: Option<syn::Path>,
    error: Option<Type>,
    serde: bool,
}

/// **English:**<br>
/// Expands a struct or enum into value-object implementations;
/// returns a syntax error for unsupported shapes or invalid attributes.
///
/// **日本語:**<br>
/// 構造体・列挙型から値オブジェクト実装を生成し、非対応の形状や
/// 不正な属性には構文エラーを返します。
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let attrs = parse_attributes(&input)?;
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) if fields.named.is_empty() => Err(syn::Error::new_spanned(
                &input.ident,
                "ValueObject requires at least one field",
            )),
            Fields::Unnamed(fields) if fields.unnamed.is_empty() => Err(syn::Error::new_spanned(
                &input.ident,
                "ValueObject requires at least one field",
            )),
            Fields::Named(fields) if fields.named.len() == 1 => {
                let field = &fields.named[0];
                let ident = field.ident.clone().expect("named field has an identifier");
                expand_single(
                    &input,
                    attrs,
                    field,
                    Member::Named(ident.clone()),
                    Some(ident),
                )
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => expand_single(
                &input,
                attrs,
                &fields.unnamed[0],
                Member::Unnamed(0.into()),
                None,
            ),
            Fields::Named(_) => expand_multi(&input, attrs, &data.fields, true),
            Fields::Unnamed(_) => expand_multi(&input, attrs, &data.fields, false),
            Fields::Unit => Err(syn::Error::new_spanned(
                &input.ident,
                "ValueObject requires at least one field",
            )),
        },
        Data::Enum(data) => expand_enum(&input, attrs, data),
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "ValueObject can only be derived for a struct or enum",
        )),
    }
}

struct EnumVariant<'a> {
    variant: &'a syn::Variant,
    members: Vec<Member>,
    types: Vec<&'a Type>,
    named: bool,
    unit: bool,
    suffix: syn::Ident,
}

fn expand_enum(
    input: &DeriveInput,
    attrs: Attributes,
    data: &syn::DataEnum,
) -> Result<TokenStream> {
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "ValueObject requires at least one variant",
        ));
    }

    let mut variants = Vec::with_capacity(data.variants.len());
    let mut suffixes = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        let suffix = variant.ident.to_string().to_lowercase();
        if suffixes.iter().any(|known| known == &suffix) {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "ValueObject variant method names must be unique after lower-casing",
            ));
        }
        suffixes.push(suffix.clone());

        let (members, types, named, unit) = match &variant.fields {
            Fields::Named(fields) => (
                fields
                    .named
                    .iter()
                    .map(|field| Member::Named(field.ident.clone().expect("named field")))
                    .collect(),
                fields.named.iter().map(|field| &field.ty).collect(),
                true,
                false,
            ),
            Fields::Unnamed(fields) => (
                fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, _)| Member::Unnamed(index.into()))
                    .collect(),
                fields.unnamed.iter().map(|field| &field.ty).collect(),
                false,
                false,
            ),
            Fields::Unit => (Vec::new(), Vec::new(), false, true),
        };
        variants.push(EnumVariant {
            variant,
            members,
            types,
            named,
            unit,
            suffix: format_ident!("{suffix}"),
        });
    }

    let core = core_path()?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let constructors = enum_constructors(&variants, &attrs);
    let accessors = enum_accessors(&variants);
    let standard_impls = standard_impls_enum(name, &input.generics, &variants);
    let serde_impl = if attrs.serde {
        Some(serde_impl_enum(
            name,
            &input.generics,
            &variants,
            attrs.validator.as_ref(),
            &core,
        )?)
    } else {
        None
    };

    Ok(quote! {
        impl #impl_generics #core::ValueObject for #name #ty_generics #where_clause {
            type Inner = Self;

            fn as_inner(&self) -> &Self::Inner {
                self
            }

            fn into_inner(self) -> Self::Inner {
                self
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// **English:**<br>
            /// Borrows the enum itself, including its current variant.
            ///
            /// **日本語:**<br>
            /// 現在のバリアントを含む列挙型自身を借用します。
            pub fn as_inner(&self) -> &Self {
                self
            }

            /// **English:**<br>
            /// Transfers ownership of this enum without changing its variant.
            ///
            /// **日本語:**<br>
            /// バリアントを変更せずに列挙型の所有権を移します。
            pub fn into_inner(self) -> Self {
                self
            }

            #constructors
            #accessors
        }

        #standard_impls
        #serde_impl
    })
}

fn enum_arguments(variant: &EnumVariant<'_>) -> Vec<syn::Ident> {
    if variant.named {
        variant
            .members
            .iter()
            .map(|member| match member {
                Member::Named(ident) => ident.clone(),
                Member::Unnamed(_) => unreachable!("named variant member"),
            })
            .collect()
    } else {
        (0..variant.types.len())
            .map(|index| format_ident!("field_{index}"))
            .collect()
    }
}

fn enum_construct(variant: &EnumVariant<'_>, arguments: &[syn::Ident]) -> TokenStream {
    let variant_name = &variant.variant.ident;
    if variant.unit {
        quote! { Self::#variant_name }
    } else if variant.named {
        let members = &variant.members;
        quote! { Self::#variant_name { #(#members: #arguments),* } }
    } else {
        quote! { Self::#variant_name(#(#arguments),*) }
    }
}

fn enum_constructors(variants: &[EnumVariant<'_>], attrs: &Attributes) -> TokenStream {
    let methods = variants.iter().map(|variant| {
        let method = if attrs.validator.is_some() {
            format_ident!("try_new_{}", variant.suffix)
        } else {
            format_ident!("new_{}", variant.suffix)
        };
        let arguments = enum_arguments(variant);
        let types = &variant.types;
        let construct = enum_construct(variant, &arguments);
        match (&attrs.validator, &attrs.error) {
            (Some(validator), Some(error)) => quote! {
                /// **English:**<br>
                /// Constructs this variant after validating the candidate;
                /// propagates the validator's error or panic.
                ///
                /// **日本語:**<br>
                /// 候補を検証してこのバリアントを構築し、検証関数の
                /// エラーやパニックを伝播します。
                pub fn #method(#(#arguments: #types),*) -> ::core::result::Result<Self, #error> {
                    let candidate = #construct;
                    #validator(&candidate)?;
                    Ok(candidate)
                }
            },
            (None, None) => quote! {
                /// **English:**<br>
                /// Constructs this variant from its fields without validation.
                ///
                /// **日本語:**<br>
                /// 検証せずにフィールドからこのバリアントを構築します。
                pub fn #method(#(#arguments: #types),*) -> Self {
                    #construct
                }
            },
            _ => unreachable!("validation attributes were checked together"),
        }
    });
    quote! { #(#methods)* }
}

fn enum_pattern(variant: &EnumVariant<'_>, bindings: &[syn::Ident]) -> TokenStream {
    let variant_name = &variant.variant.ident;
    if variant.unit {
        quote! { Self::#variant_name }
    } else if variant.named {
        let members = &variant.members;
        quote! { Self::#variant_name { #(#members: #bindings),* } }
    } else {
        quote! { Self::#variant_name(#(#bindings),*) }
    }
}

fn enum_accessors(variants: &[EnumVariant<'_>]) -> TokenStream {
    let methods = variants.iter().map(|variant| {
        let is_method = format_ident!("is_{}", variant.suffix);
        let as_method = format_ident!("as_{}", variant.suffix);
        let variant_name = &variant.variant.ident;
        let is_pattern = if variant.unit {
            quote! { Self::#variant_name }
        } else if variant.named {
            quote! { Self::#variant_name { .. } }
        } else {
            quote! { Self::#variant_name(..) }
        };
        let bindings: Vec<_> = (0..variant.types.len())
            .map(|index| format_ident!("__ddd_value_object_field_{index}"))
            .collect();
        let pattern = enum_pattern(variant, &bindings);
        let reference_types = variant.types.iter().map(|ty| quote! { &#ty });
        let references = bindings.iter().map(|binding| quote! { #binding });
        let return_type = if variant.unit {
            quote! { ::core::option::Option<()> }
        } else {
            quote! { ::core::option::Option<(#(#reference_types,)*)> }
        };
        let success = if variant.unit {
            quote! { Some(()) }
        } else {
            quote! { Some((#(#references,)*)) }
        };
        quote! {
            /// **English:**<br>
            /// Tests whether this value has the corresponding variant.
            ///
            /// **日本語:**<br>
            /// この値が対応するバリアントかどうかを判定します。
            pub fn #is_method(&self) -> bool {
                matches!(self, #is_pattern)
            }

            /// **English:**<br>
            /// Returns the variant's field references as a tuple, or
            /// `Some(())` for a unit variant; returns `None` for a different variant.
            ///
            /// **日本語:**<br>
            /// バリアントのフィールド参照のタプルを返します。
            /// ユニットなら `Some(())`、異なるバリアントなら `None` を返します。
            pub fn #as_method(&self) -> #return_type {
                match self {
                    #pattern => #success,
                    _ => None,
                }
            }
        }
    });
    quote! { #(#methods)* }
}

fn standard_impls_enum(
    name: &syn::Ident,
    original: &Generics,
    variants: &[EnumVariant<'_>],
) -> TokenStream {
    let mut debug_generics = original.clone();
    for variant in variants {
        for ty in &variant.types {
            debug_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::fmt::Debug));
        }
    }
    let (debug_impl, debug_ty, debug_where) = debug_generics.split_for_impl();
    let debug_arms = variants.iter().map(|variant| {
        let bindings: Vec<_> = (0..variant.types.len())
            .map(|index| format_ident!("__ddd_debug_{index}"))
            .collect();
        let pattern = enum_pattern(variant, &bindings);
        let variant_name = &variant.variant.ident;
        if variant.unit {
            quote! { #pattern => formatter.write_str(concat!(stringify!(#name), "::", stringify!(#variant_name))), }
        } else if variant.named {
            let fields = variant.members.iter().enumerate().map(|(index, member)| {
                let binding = &bindings[index];
                let label = match member {
                    Member::Named(ident) => LitStr::new(&ident.to_string(), ident.span()),
                    Member::Unnamed(_) => unreachable!("named enum field"),
                };
                quote! { debug.field(#label, #binding); }
            });
            quote! {
                #pattern => {
                    let mut debug = formatter.debug_struct(concat!(stringify!(#name), "::", stringify!(#variant_name)));
                    #(#fields)*
                    debug.finish()
                },
            }
        } else {
            let fields = bindings.iter().map(|binding| quote! { debug.field(#binding); });
            quote! {
                #pattern => {
                    let mut debug = formatter.debug_tuple(concat!(stringify!(#name), "::", stringify!(#variant_name)));
                    #(#fields)*
                    debug.finish()
                },
            }
        }
    });

    let mut clone_generics = original.clone();
    for variant in variants {
        for ty in &variant.types {
            clone_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::clone::Clone));
        }
    }
    let (clone_impl, clone_ty, clone_where) = clone_generics.split_for_impl();
    let clone_arms = variants.iter().map(|variant| {
        let bindings: Vec<_> = (0..variant.types.len())
            .map(|index| format_ident!("__ddd_clone_{index}"))
            .collect();
        let pattern = enum_pattern(variant, &bindings);
        let values = bindings.iter().map(|binding| quote! { #binding.clone() });
        let variant_name = &variant.variant.ident;
        if variant.unit {
            quote! { #pattern => Self::#variant_name, }
        } else if variant.named {
            let members = &variant.members;
            quote! { #pattern => Self::#variant_name { #(#members: #values),* }, }
        } else {
            quote! { #pattern => Self::#variant_name(#(#values),*), }
        }
    });

    let mut partial_eq_generics = original.clone();
    for variant in variants {
        for ty in &variant.types {
            partial_eq_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::cmp::PartialEq));
        }
    }
    let (partial_eq_impl, partial_eq_ty, partial_eq_where) = partial_eq_generics.split_for_impl();
    let partial_eq_branches = variants.iter().map(|variant| {
        let is_method = format_ident!("is_{}", variant.suffix);
        let as_method = format_ident!("as_{}", variant.suffix);
        if variant.unit {
            quote! {
                if self.#is_method() {
                    return other.#is_method();
                }
            }
        } else {
            let indexes = (0..variant.types.len()).map(syn::Index::from);
            let comparisons = indexes.map(|index| {
                quote! { ::core::cmp::PartialEq::eq(&*left.#index, &*right.#index) }
            });
            quote! {
                if self.#is_method() {
                    return if other.#is_method() {
                        let left = self.#as_method().expect("matching ValueObject variant");
                        let right = other.#as_method().expect("matching ValueObject variant");
                        true #(&& #comparisons)*
                    } else {
                        false
                    };
                }
            }
        }
    });

    let mut eq_generics = original.clone();
    for variant in variants {
        for ty in &variant.types {
            eq_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::cmp::Eq));
        }
    }
    let (eq_impl, eq_ty, eq_where) = eq_generics.split_for_impl();

    let mut hash_generics = original.clone();
    for variant in variants {
        for ty in &variant.types {
            hash_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::hash::Hash));
        }
    }
    let (hash_impl, hash_ty, hash_where) = hash_generics.split_for_impl();
    let hash_arms = variants.iter().map(|variant| {
        let bindings: Vec<_> = (0..variant.types.len())
            .map(|index| format_ident!("__ddd_hash_{index}"))
            .collect();
        let pattern = enum_pattern(variant, &bindings);
        let hashes = bindings.iter().map(|binding| {
            quote! { ::core::hash::Hash::hash(#binding, state); }
        });
        quote! { #pattern => { #(#hashes)* }, }
    });

    quote! {
        impl #debug_impl ::core::fmt::Debug for #name #debug_ty #debug_where {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self { #(#debug_arms)* }
            }
        }
        impl #clone_impl ::core::clone::Clone for #name #clone_ty #clone_where {
            fn clone(&self) -> Self {
                match self { #(#clone_arms)* }
            }
        }
        impl #partial_eq_impl ::core::cmp::PartialEq for #name #partial_eq_ty #partial_eq_where {
            fn eq(&self, other: &Self) -> bool {
                #(#partial_eq_branches)*
                false
            }
        }
        impl #eq_impl ::core::cmp::Eq for #name #eq_ty #eq_where {}
        impl #hash_impl ::core::hash::Hash for #name #hash_ty #hash_where {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                ::core::hash::Hash::hash(&::core::mem::discriminant(self), state);
                match self { #(#hash_arms)* }
            }
        }
    }
}

fn enum_variant_definition(variant: &EnumVariant<'_>) -> TokenStream {
    let name = &variant.variant.ident;
    if variant.unit {
        quote! { #name }
    } else if variant.named {
        let fields = variant
            .members
            .iter()
            .zip(variant.types.iter())
            .map(|(member, ty)| {
                quote! { #member: #ty }
            });
        quote! { #name { #(#fields),* } }
    } else {
        let types = &variant.types;
        quote! { #name(#(#types),*) }
    }
}

fn serde_impl_enum(
    name: &syn::Ident,
    original: &Generics,
    variants: &[EnumVariant<'_>],
    validator: Option<&syn::Path>,
    core: &TokenStream,
) -> Result<TokenStream> {
    let serde = quote!(#core::__private::serde);
    let serde_path = LitStr::new(
        &format!(
            "{}::__private::serde",
            core.to_string().replace(' ', "").trim_start_matches("::")
        ),
        Span::call_site(),
    );
    let helper = format_ident!("__ddd_value_object_{}", name);
    let (helper_impl, _, helper_where) = original.split_for_impl();
    let definitions = variants.iter().map(enum_variant_definition);
    let helper_definition = quote! {
        /// **English:**<br>
        /// Mirrors event-free value-object variants for serde decoding.
        ///
        /// **日本語:**<br>
        /// serde の復号用に値オブジェクトのバリアント構成を保持します。
        pub enum Enum #helper_impl { #(#definitions),* } #helper_where
    };

    let mut serialize_generics = original.clone();
    for variant in variants {
        for ty in &variant.types {
            serialize_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: #serde::Serialize));
        }
    }
    let (serialize_impl, serialize_ty, serialize_where) = serialize_generics.split_for_impl();
    let serialize_arms = variants.iter().enumerate().map(|(index, variant)| {
        let bindings: Vec<_> = (0..variant.types.len())
            .map(|field| format_ident!("__ddd_serialize_{field}"))
            .collect();
        let pattern = enum_pattern(variant, &bindings);
        let variant_name = &variant.variant.ident;
        let variant_label = LitStr::new(&variant_name.to_string(), variant_name.span());
        let type_label = LitStr::new(&name.to_string(), name.span());
        if variant.unit {
            quote! {
                #pattern => #serde::Serializer::serialize_unit_variant(serializer, #type_label, #index as u32, #variant_label),
            }
        } else if variant.named {
            let fields = variant.members.iter().enumerate().map(|(field, member)| {
                let binding = &bindings[field];
                let label = match member {
                    Member::Named(ident) => LitStr::new(&ident.to_string(), ident.span()),
                    Member::Unnamed(_) => unreachable!("named enum field"),
                };
                quote! { #serde::ser::SerializeStructVariant::serialize_field(&mut state, #label, #binding)?; }
            });
            let count = variant.types.len();
            quote! {
                #pattern => {
                    let mut state = #serde::Serializer::serialize_struct_variant(serializer, #type_label, #index as u32, #variant_label, #count)?;
                    #(#fields)*
                    #serde::ser::SerializeStructVariant::end(state)
                },
            }
        } else if bindings.len() == 1 {
            let binding = &bindings[0];
            quote! {
                #pattern => #serde::Serializer::serialize_newtype_variant(serializer, #type_label, #index as u32, #variant_label, #binding),
            }
        } else {
            let fields = bindings.iter().map(|binding| {
                quote! { #serde::ser::SerializeTupleVariant::serialize_field(&mut state, #binding)?; }
            });
            let count = variant.types.len();
            quote! {
                #pattern => {
                    let mut state = #serde::Serializer::serialize_tuple_variant(serializer, #type_label, #index as u32, #variant_label, #count)?;
                    #(#fields)*
                    #serde::ser::SerializeTupleVariant::end(state)
                },
            }
        }
    });

    let mut de_generics = original.clone();
    let de_lifetime = fresh_lifetime(&de_generics);
    de_generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote!(#de_lifetime)));
    for variant in variants {
        for ty in &variant.types {
            de_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: #serde::Deserialize<#de_lifetime>));
        }
    }
    let (de_impl, _, de_where) = de_generics.split_for_impl();
    let (_, original_ty, _) = original.split_for_impl();
    let deserialize_arms = variants.iter().map(|variant| {
        let bindings: Vec<_> = (0..variant.types.len())
            .map(|field| format_ident!("__ddd_deserialize_{field}"))
            .collect();
        let helper_name = &variant.variant.ident;
        let self_name = &variant.variant.ident;
        let helper_pattern = if variant.unit {
            quote! { #helper::Enum::#helper_name }
        } else if variant.named {
            let members = &variant.members;
            quote! { #helper::Enum::#helper_name { #(#members: #bindings),* } }
        } else {
            quote! { #helper::Enum::#helper_name(#(#bindings),*) }
        };
        let self_value = if variant.unit {
            quote! { Self::#self_name }
        } else if variant.named {
            let members = &variant.members;
            quote! { Self::#self_name { #(#members: #bindings),* } }
        } else {
            quote! { Self::#self_name(#(#bindings),*) }
        };
        let success = if let Some(validator) = validator {
            quote! {
                let candidate = #self_value;
                #validator(&candidate).map_err(|_| {
                    <D::Error as #serde::de::Error>::custom("value object validation failed")
                })?;
                Ok(candidate)
            }
        } else {
            quote! { Ok(#self_value) }
        };
        quote! { #helper_pattern => { #success } }
    });

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #helper {
            #[derive(#serde::Deserialize)]
            #[serde(crate = #serde_path)]
            #helper_definition
        }

        impl #serialize_impl #serde::Serialize for #name #serialize_ty #serialize_where {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: #serde::Serializer,
            {
                match self { #(#serialize_arms)* }
            }
        }

        impl #de_impl #serde::Deserialize<#de_lifetime> for #name #original_ty #de_where {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: #serde::Deserializer<#de_lifetime>,
            {
                let value = <#helper::Enum #original_ty as #serde::Deserialize<#de_lifetime>>::deserialize(deserializer)?;
                match value { #(#deserialize_arms),* }
            }
        }
    })
}

fn expand_single(
    input: &DeriveInput,
    attrs: Attributes,
    field: &Field,
    member: Member,
    named_field: Option<syn::Ident>,
) -> Result<TokenStream> {
    if attrs.validator.is_some() && matches!(field.vis, syn::Visibility::Public(_)) {
        return Err(syn::Error::new_spanned(
            field,
            "validated ValueObject inner fields must not be public",
        ));
    }

    let core = core_path()?;
    let name = &input.ident;
    let inner_ty = &field.ty;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let construct = match &named_field {
        Some(field) => quote! { Self { #field: inner } },
        None => quote! { Self(inner) },
    };

    let constructor = match (&attrs.validator, &attrs.error) {
        (Some(validator), Some(error)) => quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// **English:**<br>
                /// Validates the inner value before wrapping it, returning
                /// the validator's error on rejection; validator panics propagate.
                ///
                /// **日本語:**<br>
                /// 内包値を検証してから包み、拒否された場合は検証エラーを
                /// 返します。検証関数のパニックは伝播します。
                pub fn try_new(inner: #inner_ty) -> ::core::result::Result<Self, #error> {
                    #validator(&inner)?;
                    Ok(#construct)
                }
            }
        },
        (None, None) => quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// **English:**<br>
                /// Wraps the supplied inner value without validation.
                ///
                /// **日本語:**<br>
                /// 指定された内包値を検証せずに包みます。
                pub fn new(inner: #inner_ty) -> Self {
                    #construct
                }
            }
        },
        _ => unreachable!("validation attributes were checked together"),
    };

    let standard_impls = standard_impls_single(
        name,
        &input.generics,
        inner_ty,
        &member,
        named_field.as_ref(),
    );
    let serde_impl = if attrs.serde {
        Some(serde_impl_single(
            name,
            &input.generics,
            inner_ty,
            attrs.validator.is_some(),
            &core,
            &member,
        )?)
    } else {
        None
    };

    Ok(quote! {
        impl #impl_generics #core::ValueObject for #name #ty_generics #where_clause {
            type Inner = #inner_ty;

            fn as_inner(&self) -> &Self::Inner {
                &self.#member
            }

            fn into_inner(self) -> Self::Inner {
                self.#member
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// **English:**<br>
            /// Borrows the wrapped field without consuming this wrapper.
            ///
            /// **日本語:**<br>
            /// ラッパーを消費せずに内包フィールドを借用します。
            pub fn as_inner(&self) -> &#inner_ty {
                &self.#member
            }

            /// **English:**<br>
            /// Consumes this wrapper and moves out its field without validation.
            ///
            /// **日本語:**<br>
            /// ラッパーを消費し、検証せずに内包フィールドを取り出します。
            pub fn into_inner(self) -> #inner_ty {
                self.#member
            }
        }

        #constructor
        #standard_impls
        #serde_impl
    })
}

fn expand_multi(
    input: &DeriveInput,
    attrs: Attributes,
    fields: &Fields,
    named: bool,
) -> Result<TokenStream> {
    let field_list: Vec<&Field> = match fields {
        Fields::Named(fields) => fields.named.iter().collect(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect(),
        Fields::Unit => unreachable!("unit fields are rejected before expansion"),
    };
    if attrs.validator.is_some()
        && let Some(field) = field_list
            .iter()
            .find(|field| matches!(field.vis, syn::Visibility::Public(_)))
    {
        return Err(syn::Error::new_spanned(
            field,
            "validated ValueObject fields must not be public",
        ));
    }

    let members: Vec<Member> = field_list
        .iter()
        .enumerate()
        .map(|(index, field)| {
            if named {
                Member::Named(field.ident.clone().expect("named field has an identifier"))
            } else {
                Member::Unnamed(index.into())
            }
        })
        .collect();
    let arguments: Vec<syn::Ident> = if named {
        field_list
            .iter()
            .map(|field| field.ident.clone().expect("named field has an identifier"))
            .collect()
    } else {
        (0..field_list.len())
            .map(|index| format_ident!("field_{index}"))
            .collect()
    };
    let types: Vec<&Type> = field_list.iter().map(|field| &field.ty).collect();
    let core = core_path()?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let construct = construct_self(named, &members, &arguments);
    let constructor = match (&attrs.validator, &attrs.error) {
        (Some(validator), Some(error)) => quote! {
            /// **English:**<br>
            /// Builds fields in declaration order and validates the resulting
            /// struct, propagating the validator's error or panic.
            ///
            /// **日本語:**<br>
            /// 宣言順のフィールドから構造体を作って検証し、検証関数の
            /// エラーやパニックを伝播します。
            pub fn try_new(#(#arguments: #types),*) -> ::core::result::Result<Self, #error> {
                let candidate = #construct;
                #validator(&candidate)?;
                Ok(candidate)
            }
        },
        (None, None) => quote! {
            /// **English:**<br>
            /// Builds this struct from fields in declaration order without validation.
            ///
            /// **日本語:**<br>
            /// 宣言順のフィールドから検証せずに構造体を構築します。
            pub fn new(#(#arguments: #types),*) -> Self {
                #construct
            }
        },
        _ => unreachable!("validation attributes were checked together"),
    };
    let accessors =
        arguments
            .iter()
            .zip(members.iter())
            .zip(types.iter())
            .map(|((argument, member), ty)| {
                quote! {
                    /// **English:**<br>
                    /// Borrows this field, keeping ownership in the value object.
                    ///
                    /// **日本語:**<br>
                    /// 値オブジェクトに所有権を残してこのフィールドを借用します。
                    pub fn #argument(&self) -> &#ty {
                        &self.#member
                    }
                }
            });
    let standard_impls = standard_impls_multi(name, &input.generics, &members, &types, named);
    let serde_impl = if attrs.serde {
        Some(serde_impl_multi(
            name,
            &input.generics,
            fields,
            &members,
            &types,
            named,
            attrs.validator.is_some(),
            &core,
        )?)
    } else {
        None
    };

    Ok(quote! {
        impl #impl_generics #core::ValueObject for #name #ty_generics #where_clause {
            type Inner = Self;

            fn as_inner(&self) -> &Self::Inner {
                self
            }

            fn into_inner(self) -> Self::Inner {
                self
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            #constructor

            /// **English:**<br>
            /// Borrows this multi-field struct as its own inner value.
            ///
            /// **日本語:**<br>
            /// 複数フィールド構造体自身を内包値として借用します。
            pub fn as_inner(&self) -> &Self {
                self
            }

            /// **English:**<br>
            /// Returns this multi-field struct by value without revalidation.
            ///
            /// **日本語:**<br>
            /// 再検証せずに複数フィールド構造体自身を値として返します。
            pub fn into_inner(self) -> Self {
                self
            }

            #(#accessors)*
        }

        #standard_impls
        #serde_impl
    })
}

fn construct_self(named: bool, members: &[Member], arguments: &[syn::Ident]) -> TokenStream {
    if named {
        quote! { Self { #(#members: #arguments),* } }
    } else {
        quote! { Self(#(#arguments),*) }
    }
}

fn parse_attributes(input: &DeriveInput) -> Result<Attributes> {
    let matching: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("value_object"))
        .collect();
    if matching.len() > 1 {
        return Err(syn::Error::new_spanned(
            matching[1],
            "value_object attributes may be specified only once",
        ));
    }

    let mut attributes = Attributes {
        validator: None,
        error: None,
        serde: false,
    };
    if let Some(attr) = matching.first() {
        let mut option_found = false;
        attr.parse_nested_meta(|meta| {
            option_found = true;
            if meta.path.is_ident("validate") {
                if attributes.validator.is_some() {
                    return Err(meta.error("duplicate validate attribute"));
                }
                attributes.validator = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("error") {
                if attributes.error.is_some() {
                    return Err(meta.error("duplicate error attribute"));
                }
                attributes.error = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("serde") {
                if !meta.input.is_empty() {
                    return Err(meta.error("serde is a flag and takes no value"));
                }
                if attributes.serde {
                    return Err(meta.error("duplicate serde attribute"));
                }
                attributes.serde = true;
                return Ok(());
            }
            Err(meta.error("unknown value_object attribute"))
        })?;
        if !option_found {
            return Err(syn::Error::new_spanned(
                attr,
                "value_object requires at least one option",
            ));
        }
    }

    if attributes.validator.is_some() != attributes.error.is_some() {
        return Err(syn::Error::new_spanned(
            matching.first().expect("one attribute exists"),
            "validate and error must be specified together",
        ));
    }
    Ok(attributes)
}

fn standard_impls_single(
    name: &syn::Ident,
    original: &Generics,
    inner_ty: &Type,
    member: &Member,
    named_field: Option<&syn::Ident>,
) -> TokenStream {
    let mut debug_generics = original.clone();
    debug_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#inner_ty: ::core::fmt::Debug));
    let (debug_impl, debug_ty, debug_where) = debug_generics.split_for_impl();

    let mut clone_generics = original.clone();
    clone_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#inner_ty: ::core::clone::Clone));
    let (clone_impl, clone_ty, clone_where) = clone_generics.split_for_impl();

    let mut partial_eq_generics = original.clone();
    partial_eq_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#inner_ty: ::core::cmp::PartialEq));
    let (partial_eq_impl, partial_eq_ty, partial_eq_where) = partial_eq_generics.split_for_impl();

    let mut eq_generics = original.clone();
    eq_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#inner_ty: ::core::cmp::Eq));
    let (eq_impl, eq_ty, eq_where) = eq_generics.split_for_impl();

    let mut hash_generics = original.clone();
    hash_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#inner_ty: ::core::hash::Hash));
    let (hash_impl, hash_ty, hash_where) = hash_generics.split_for_impl();

    let clone = match named_field {
        Some(field) => quote! { Self { #field: self.#member.clone() } },
        None => quote! { Self(self.#member.clone()) },
    };

    quote! {
        impl #debug_impl ::core::fmt::Debug for #name #debug_ty #debug_where {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(&self.#member, formatter)
            }
        }
        impl #clone_impl ::core::clone::Clone for #name #clone_ty #clone_where {
            fn clone(&self) -> Self {
                #clone
            }
        }
        impl #partial_eq_impl ::core::cmp::PartialEq for #name #partial_eq_ty #partial_eq_where {
            fn eq(&self, other: &Self) -> bool {
                self.#member == other.#member
            }
        }
        impl #eq_impl ::core::cmp::Eq for #name #eq_ty #eq_where {}
        impl #hash_impl ::core::hash::Hash for #name #hash_ty #hash_where {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                ::core::hash::Hash::hash(&self.#member, state);
            }
        }
    }
}

fn standard_impls_multi(
    name: &syn::Ident,
    original: &Generics,
    members: &[Member],
    types: &[&Type],
    named: bool,
) -> TokenStream {
    let mut debug_generics = original.clone();
    for ty in types {
        debug_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::core::fmt::Debug));
    }
    let (debug_impl, debug_ty, debug_where) = debug_generics.split_for_impl();

    let mut clone_generics = original.clone();
    for ty in types {
        clone_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::core::clone::Clone));
    }
    let (clone_impl, clone_ty, clone_where) = clone_generics.split_for_impl();

    let mut partial_eq_generics = original.clone();
    for ty in types {
        partial_eq_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::core::cmp::PartialEq));
    }
    let (partial_eq_impl, partial_eq_ty, partial_eq_where) = partial_eq_generics.split_for_impl();

    let mut eq_generics = original.clone();
    for ty in types {
        eq_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::core::cmp::Eq));
    }
    let (eq_impl, eq_ty, eq_where) = eq_generics.split_for_impl();

    let mut hash_generics = original.clone();
    for ty in types {
        hash_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::core::hash::Hash));
    }
    let (hash_impl, hash_ty, hash_where) = hash_generics.split_for_impl();

    let clone_values = members.iter().map(|member| quote! { self.#member.clone() });
    let clone = if named {
        quote! { Self { #(#members: #clone_values),* } }
    } else {
        quote! { Self(#(#clone_values),*) }
    };
    let comparisons = members
        .iter()
        .map(|member| quote! { self.#member == other.#member });
    let hashes = members
        .iter()
        .map(|member| quote! { ::core::hash::Hash::hash(&self.#member, state); });
    let debug_names: Vec<LitStr> = members
        .iter()
        .enumerate()
        .map(|(index, member)| match member {
            Member::Named(ident) => LitStr::new(&ident.to_string(), ident.span()),
            Member::Unnamed(_) => LitStr::new(&format!("field_{index}"), Span::call_site()),
        })
        .collect();
    let debug = if named {
        quote! {
            let mut debug = formatter.debug_struct(stringify!(#name));
            #(debug.field(#debug_names, &self.#members);)*
            debug.finish()
        }
    } else {
        quote! {
            let mut debug = formatter.debug_tuple(stringify!(#name));
            #(debug.field(&self.#members);)*
            debug.finish()
        }
    };

    quote! {
        impl #debug_impl ::core::fmt::Debug for #name #debug_ty #debug_where {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #debug
            }
        }
        impl #clone_impl ::core::clone::Clone for #name #clone_ty #clone_where {
            fn clone(&self) -> Self {
                #clone
            }
        }
        impl #partial_eq_impl ::core::cmp::PartialEq for #name #partial_eq_ty #partial_eq_where {
            fn eq(&self, other: &Self) -> bool {
                true #(&& #comparisons)*
            }
        }
        impl #eq_impl ::core::cmp::Eq for #name #eq_ty #eq_where {}
        impl #hash_impl ::core::hash::Hash for #name #hash_ty #hash_where {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                #(#hashes)*
            }
        }
    }
}

fn serde_impl_single(
    name: &syn::Ident,
    original: &Generics,
    inner_ty: &Type,
    validated: bool,
    core: &TokenStream,
    member: &Member,
) -> Result<TokenStream> {
    let serde = quote!(#core::__private::serde);
    let mut serialize_generics = original.clone();
    serialize_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#inner_ty: #serde::Serialize));
    let (serialize_impl, serialize_ty, serialize_where) = serialize_generics.split_for_impl();

    let mut de_generics = original.clone();
    let de_lifetime = fresh_lifetime(&de_generics);
    de_generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote!(#de_lifetime)));
    de_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(
            #inner_ty: #serde::Deserialize<#de_lifetime>
        ));
    let (de_impl, _, de_where) = de_generics.split_for_impl();
    let (_, original_ty, _) = original.split_for_impl();
    let build = if validated {
        quote! {
            Self::try_new(inner).map_err(|_| {
                <D::Error as #serde::de::Error>::custom("value object validation failed")
            })
        }
    } else {
        quote!(Ok(Self::new(inner)))
    };

    Ok(quote! {
        impl #serialize_impl #serde::Serialize for #name #serialize_ty #serialize_where {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: #serde::Serializer,
            {
                #serde::Serialize::serialize(&self.#member, serializer)
            }
        }

        impl #de_impl #serde::Deserialize<#de_lifetime> for #name #original_ty #de_where {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: #serde::Deserializer<#de_lifetime>,
            {
                let inner = <#inner_ty as #serde::Deserialize<#de_lifetime>>::deserialize(deserializer)?;
                #build
            }
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn serde_impl_multi(
    name: &syn::Ident,
    original: &Generics,
    fields: &Fields,
    members: &[Member],
    types: &[&Type],
    named: bool,
    validated: bool,
    core: &TokenStream,
) -> Result<TokenStream> {
    let serde = quote!(#core::__private::serde);
    let serde_path = LitStr::new(
        &format!(
            "{}::__private::serde",
            core.to_string().replace(' ', "").trim_start_matches("::")
        ),
        Span::call_site(),
    );
    let helper = format_ident!("__ddd_value_object_{}", name);
    let (helper_impl, _, helper_where) = original.split_for_impl();
    let helper_definition = if named {
        let defs = fields.iter().map(|field| {
            let ident = field.ident.as_ref().expect("named field has an identifier");
            let ty = &field.ty;
            quote! { pub #ident: #ty }
        });
        quote! {
            /// **English:**<br>
            /// Holds named fields decoded before value-object validation.
            ///
            /// **日本語:**<br>
            /// 値オブジェクトの検証前に復号された名前付きフィールドを保持します。
            pub struct Fields #helper_impl { #(#defs),* } #helper_where
        }
    } else {
        quote! {
            /// **English:**<br>
            /// Holds tuple fields decoded before value-object construction.
            ///
            /// **日本語:**<br>
            /// 値オブジェクト構築前に復号されたタプルフィールドを保持します。
            pub struct Fields #helper_impl (#(pub #types),*) #helper_where;
        }
    };
    let helper_values: Vec<TokenStream> = members
        .iter()
        .map(|member| quote! { fields.#member })
        .collect();
    let construct = if named {
        quote! { Self { #(#members: #helper_values),* } }
    } else {
        quote! { Self(#(#helper_values),*) }
    };

    let mut serialize_generics = original.clone();
    for ty in types {
        serialize_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: #serde::Serialize));
    }
    let (serialize_impl, serialize_ty, serialize_where) = serialize_generics.split_for_impl();
    let mut de_generics = original.clone();
    let de_lifetime = fresh_lifetime(&de_generics);
    de_generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote!(#de_lifetime)));
    for ty in types {
        de_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: #serde::Deserialize<#de_lifetime>));
    }
    let (de_impl, _, de_where) = de_generics.split_for_impl();
    let (_, original_ty, _) = original.split_for_impl();

    let field_count = members.len();
    let serialize_body = if named {
        let names: Vec<LitStr> = fields
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().expect("named field has an identifier");
                LitStr::new(&ident.to_string(), ident.span())
            })
            .collect();
        quote! {
            let mut state = #serde::Serializer::serialize_struct(serializer, stringify!(#name), #field_count)?;
            #( #serde::ser::SerializeStruct::serialize_field(&mut state, #names, &self.#members)?; )*
            #serde::ser::SerializeStruct::end(state)
        }
    } else {
        quote! {
            let mut state = #serde::Serializer::serialize_tuple(serializer, stringify!(#name), #field_count)?;
            #( #serde::ser::SerializeTuple::serialize_element(&mut state, &self.#members)?; )*
            #serde::ser::SerializeTuple::end(state)
        }
    };
    let deserialize_build = if validated {
        quote! {
            Self::try_new(#(#helper_values),*).map_err(|_| {
                <D::Error as #serde::de::Error>::custom("value object validation failed")
            })
        }
    } else {
        quote! { Ok(#construct) }
    };
    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #helper {
            #[derive(#serde::Deserialize)]
            #[serde(crate = #serde_path)]
            #helper_definition
        }

        impl #serialize_impl #serde::Serialize for #name #serialize_ty #serialize_where {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: #serde::Serializer,
            {
                #serialize_body
            }
        }

        impl #de_impl #serde::Deserialize<#de_lifetime> for #name #original_ty #de_where {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: #serde::Deserializer<#de_lifetime>,
            {
                let fields = <#helper::Fields #original_ty as #serde::Deserialize<#de_lifetime>>::deserialize(deserializer)?;
                #deserialize_build
            }
        }
    })
}

fn fresh_lifetime(generics: &Generics) -> Lifetime {
    let mut suffix = String::new();
    loop {
        let candidate = format!("'__ddd_de{suffix}");
        let exists = generics.params.iter().any(|param| match param {
            GenericParam::Lifetime(param) => param.lifetime.ident == candidate[1..],
            _ => false,
        });
        if !exists {
            return Lifetime::new(&candidate, Span::call_site());
        }
        suffix.push('x');
    }
}
