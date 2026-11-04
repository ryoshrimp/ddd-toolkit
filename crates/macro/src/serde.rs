//! Serde owns wire-format interpretation; domain construction stays here.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, Generics, Lifetime, LitStr, Member, Meta,
    Result, Token, Type, ext::IdentExt, parse_quote, punctuated::Punctuated,
};

// This is an allowlist, not a second implementation of Serde attributes.
// Serde itself checks placement, syntax and interactions on the helper types.
fn attributes(attrs: &[Attribute], allowed: &[&str]) -> Result<Vec<Attribute>> {
    let mut result = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let options = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for option in options {
            if !allowed.iter().any(|name| option.path().is_ident(name)) {
                return Err(syn::Error::new_spanned(
                    option,
                    "unsupported serde attribute for generated domain serde; see docs/macro-serde.md",
                ));
            }
        }
        result.push(attr.clone());
    }
    Ok(result)
}

const CONTAINER: &[&str] = &[
    "rename",
    "rename_all",
    "rename_all_fields",
    "deny_unknown_fields",
    "tag",
    "content",
    "untagged",
    "transparent",
    "bound",
];
const VARIANT: &[&str] = &[
    "rename",
    "rename_all",
    "alias",
    "skip",
    "skip_serializing",
    "skip_deserializing",
    "other",
    "untagged",
    "bound",
];
const FIELD: &[&str] = &[
    "rename",
    "alias",
    "default",
    "flatten",
    "skip",
    "skip_serializing",
    "skip_deserializing",
    "skip_serializing_if",
    "serialize_with",
    "deserialize_with",
    "with",
    "borrow",
    "bound",
];

pub(crate) fn check(input: &DeriveInput) -> Result<()> {
    attributes(&input.attrs, CONTAINER)?;
    match &input.data {
        Data::Struct(data) => {
            for field in &data.fields {
                attributes(&field.attrs, FIELD)?;
            }
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                attributes(&variant.attrs, VARIANT)?;
                for field in &variant.fields {
                    attributes(&field.attrs, FIELD)?;
                }
            }
        }
        Data::Union(_) => {}
    }
    Ok(())
}

fn fresh_lifetime(generics: &Generics, stem: &str) -> Lifetime {
    let mut name = stem.to_owned();
    while generics
        .lifetimes()
        .any(|param| param.lifetime.ident == name)
    {
        name.push('x');
    }
    Lifetime::new(&format!("'{name}"), Span::call_site())
}

fn fields(fields: &Fields, borrow: Option<&Lifetime>) -> Result<TokenStream> {
    let definitions = fields
        .iter()
        .map(|field| {
            let attrs = attributes(&field.attrs, FIELD)?;
            let ty = &field.ty;
            let ty = borrow.map_or_else(|| quote!(#ty), |lt| quote!(&#lt #ty));
            let ident = &field.ident;
            Ok(if ident.is_some() {
                quote!(#(#attrs)* pub #ident: #ty)
            } else {
                quote!(#(#attrs)* pub #ty)
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(match fields {
        Fields::Named(_) => quote!({ #(#definitions),* }),
        Fields::Unnamed(_) => quote!(( #(#definitions),* )),
        Fields::Unit => quote!(),
    })
}

fn members(fields: &Fields) -> Vec<Member> {
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            field
                .ident
                .clone()
                .map(Member::Named)
                .unwrap_or_else(|| Member::Unnamed(i.into()))
        })
        .collect()
}

fn construct(path: TokenStream, fields: &Fields, values: &[TokenStream]) -> TokenStream {
    let members = members(fields);
    match fields {
        Fields::Named(_) => quote!(#path { #(#members: #values),* }),
        Fields::Unnamed(_) => quote!(#path(#(#values),*)),
        Fields::Unit => path,
    }
}

pub(crate) fn expand(
    input: &DeriveInput,
    validator: Option<&syn::Path>,
    core: &TokenStream,
    entity: bool,
) -> Result<TokenStream> {
    check(input)?;
    let name = &input.ident;
    let serde = quote!(#core::__private::serde);
    let serde_path = LitStr::new(
        &format!("{}::__private::serde", core.to_string().replace(' ', "")),
        Span::call_site(),
    );
    let module = format_ident!("__ddd_serde_{}", name);
    let ser_name = format_ident!("__DddSerialize{}", name);
    let de_name = format_ident!("__DddDeserialize{}", name);
    let original = &input.generics;
    let fresh_type = |stem: &str| {
        let mut name = stem.to_owned();
        while original.type_params().any(|p| p.ident == name)
            || original.const_params().any(|p| p.ident == name)
        {
            name.push('x');
        }
        format_ident!("{}", name)
    };
    let serializer = fresh_type("__DddSerializer");
    let deserializer = fresh_type("__DddDeserializer");
    let (_, original_ty, _) = original.split_for_impl();
    let lt = fresh_lifetime(original, "__ddd_ser");
    let has_fields = match &input.data {
        Data::Struct(data) => !data.fields.is_empty(),
        Data::Enum(data) => data.variants.iter().any(|v| !v.fields.is_empty()),
        _ => false,
    };
    let mut ser_generics = original.clone();
    if has_fields {
        ser_generics
            .params
            .insert(0, GenericParam::Lifetime(parse_quote!(#lt)));
    }
    let (ser_params, ser_ty, ser_where) = ser_generics.split_for_impl();
    let (de_params, _, de_helper_where) = original.split_for_impl();
    let attrs = attributes(&input.attrs, CONTAINER)?;
    let default_name = LitStr::new(&name.unraw().to_string(), name.span());
    let (mut renamed_ser, mut renamed_de) = (false, false);
    for attr in &attrs {
        for option in attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            if option.path().is_ident("rename") {
                if let Meta::List(list) = option {
                    for direction in
                        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                    {
                        renamed_ser |= direction.path().is_ident("serialize");
                        renamed_de |= direction.path().is_ident("deserialize");
                    }
                } else {
                    renamed_ser = true;
                    renamed_de = true;
                }
            }
        }
    }
    let ser_rename = (!renamed_ser).then(|| quote!(#[serde(rename(serialize = #default_name))]));
    let de_rename = (!renamed_de).then(|| quote!(#[serde(rename(deserialize = #default_name))]));
    let mut serialize_arms = Vec::new();
    let mut deserialize_arms = Vec::new();
    let (ser_definition, de_definition, serialize_body, deserialize_body) = match &input.data {
        Data::Struct(data) => {
            // ValueObject single fields have always used the inner value's wire format.
            let transparent = (!entity
                && data.fields.len() == 1
                && !input.attrs.iter().any(|a| {
                    a.path().is_ident("serde")
                        && a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                            .is_ok_and(|options| {
                                options.iter().any(|m| m.path().is_ident("transparent"))
                            })
                }))
            .then(|| quote!(#[serde(transparent)]));
            let sf = fields(&data.fields, Some(&lt))?;
            let df = fields(&data.fields, None)?;
            let (mut ser_def, de_def) = match &data.fields {
                Fields::Named(_) => (
                    quote!(pub struct #ser_name #ser_params #ser_where #sf),
                    quote!(pub struct #de_name #de_params #de_helper_where #df),
                ),
                _ => (
                    quote!(pub struct #ser_name #ser_params #sf #ser_where;),
                    quote!(pub struct #de_name #de_params #df #de_helper_where;),
                ),
            };
            // Preserve the pre-existing serialize_tuple protocol (not
            // serialize_tuple_struct) by letting Serde serialize an untagged
            // tuple variant. The decoding helper remains a tuple struct.
            let tuple = matches!(data.fields, Fields::Unnamed(_)) && data.fields.len() > 1;
            if tuple {
                let mut sf = data.fields.clone();
                for field in &mut sf {
                    field.vis = syn::Visibility::Inherited;
                    field.attrs = attributes(&field.attrs, FIELD)?;
                    let ty = &field.ty;
                    field.ty = parse_quote!(&#lt #ty);
                }
                ser_def = quote! {
                    #[serde(untagged)]
                    /// **English:**<br>
                    /// Borrows tuple fields for Serde's tuple protocol.
                    ///
                    /// **日本語:**<br>
                    /// Serde のタプル形式に渡すフィールドを借用します。
                    pub enum #ser_name #ser_params #ser_where { Fields #sf }
                };
            }
            let serialize_path = if tuple {
                quote!(#module::#ser_name::Fields)
            } else {
                quote!(#module::#ser_name)
            };
            let ms = members(&data.fields);
            let borrowed = ms.iter().map(|m| quote!(&self.#m)).collect::<Vec<_>>();
            let owned = ms.iter().map(|m| quote!(value.#m)).collect::<Vec<_>>();
            (
                quote!(#transparent #ser_def),
                quote!(#transparent #de_def),
                construct(serialize_path, &data.fields, &borrowed),
                construct(quote!(Self), &data.fields, &owned),
            )
        }
        Data::Enum(data) => {
            let mut ser_variants = Vec::new();
            let mut de_variants = Vec::new();
            for variant in &data.variants {
                let v = &variant.ident;
                let va = attributes(&variant.attrs, VARIANT)?;
                // Enum fields cannot have visibility qualifiers.
                let mut sf = variant.fields.clone();
                let mut df = variant.fields.clone();
                for field in &mut sf {
                    field.attrs = attributes(&field.attrs, FIELD)?;
                    let ty = &field.ty;
                    field.ty = parse_quote!(&#lt #ty);
                }
                for field in &mut df {
                    field.attrs = attributes(&field.attrs, FIELD)?;
                }
                ser_variants.push(quote!(#(#va)* #v #sf));
                de_variants.push(quote!(#(#va)* #v #df));
                let bindings = (0..variant.fields.len())
                    .map(|i| {
                        let ident = format_ident!("__ddd_field_{i}");
                        quote!(#ident)
                    })
                    .collect::<Vec<_>>();
                let sp = construct(quote!(Self::#v), &variant.fields, &bindings);
                let sv = construct(quote!(#module::#ser_name::#v), &variant.fields, &bindings);
                let dp = construct(quote!(#module::#de_name::#v), &variant.fields, &bindings);
                let dv = construct(quote!(Self::#v), &variant.fields, &bindings);
                serialize_arms.push(quote!(#sp => #sv));
                deserialize_arms.push(quote!(#dp => #dv));
            }
            (
                quote!(pub enum #ser_name #ser_params #ser_where { #(#ser_variants),* }),
                quote!(pub enum #de_name #de_params #de_helper_where { #(#de_variants),* }),
                quote!(match self { #(#serialize_arms),* }),
                quote!(match value { #(#deserialize_arms),* }),
            )
        }
        Data::Union(_) => unreachable!(),
    };
    let mut ser_impl_generics = original.clone();
    let helper_ser_type: Type = syn::parse2(quote!(#module::#ser_name #ser_ty))?;
    ser_impl_generics
        .make_where_clause()
        .predicates
        .push(if has_fields {
            parse_quote!(for<#lt> #helper_ser_type: #serde::Serialize)
        } else {
            parse_quote!(#helper_ser_type: #serde::Serialize)
        });
    let (ser_impl, _, ser_impl_where) = ser_impl_generics.split_for_impl();
    let de_lt = fresh_lifetime(original, "__ddd_de");
    let mut de_generics = original.clone();
    de_generics
        .params
        .insert(0, GenericParam::Lifetime(parse_quote!(#de_lt)));
    de_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#module::#de_name #original_ty: #serde::Deserialize<#de_lt>));
    let (de_impl, _, de_where) = de_generics.split_for_impl();
    let error = if entity {
        "entity validation failed"
    } else {
        "value object validation failed"
    };
    let validation_target = match &input.data {
        Data::Struct(data) if !entity && data.fields.len() == 1 => {
            let member = &members(&data.fields)[0];
            quote!(&candidate.#member)
        }
        _ => quote!(&candidate),
    };
    let validation = validator.map(|validator| quote! {
        #validator(#validation_target).map_err(|_| <#deserializer::Error as #serde::de::Error>::custom(#error))?;
    });
    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module {
            #[allow(unused_imports)]
            use super::*;
            #[derive(#serde::Serialize)]
            #[serde(crate = #serde_path)]
            #ser_rename
            #de_rename
            #(#attrs)*
            #ser_definition
            #[derive(#serde::Deserialize)]
            #[serde(crate = #serde_path)]
            #ser_rename
            #de_rename
            #(#attrs)*
            #de_definition
        }
        impl #ser_impl #serde::Serialize for #name #original_ty #ser_impl_where {
            fn serialize<#serializer>(&self, serializer: #serializer) -> ::core::result::Result<#serializer::Ok, #serializer::Error>
            where #serializer: #serde::Serializer {
                #serde::Serialize::serialize(&(#serialize_body), serializer)
            }
        }
        impl #de_impl #serde::Deserialize<#de_lt> for #name #original_ty #de_where {
            fn deserialize<#deserializer>(deserializer: #deserializer) -> ::core::result::Result<Self, #deserializer::Error>
            where #deserializer: #serde::Deserializer<#de_lt> {
                let value = <#module::#de_name #original_ty as #serde::Deserialize<#de_lt>>::deserialize(deserializer)?;
                let candidate = #deserialize_body;
                #validation
                Ok(candidate)
            }
        }
    })
}
