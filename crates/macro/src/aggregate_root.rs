use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Result, Type, parse_quote};

use crate::path::core_path;

/// **English:**<br>
/// Generates append-and-drain aggregate methods for one marked vector;
/// reports invalid struct shapes, queue types, or queue attributes as syntax errors.
///
/// **日本語:**<br>
/// 指定された一つのベクターに追加・取り出しを行う集約メソッドを生成し、
/// 不正な構造体形状、キュー型、キュー属性を構文エラーにします。
pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    for attr in &input.attrs {
        if attr.path().is_ident("aggregate") {
            return Err(syn::Error::new_spanned(
                attr,
                "aggregate(events) must be placed on an event queue field",
            ));
        }
    }

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "AggregateRoot requires a named-field struct",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AggregateRoot can only be derived for a named-field struct",
            ));
        }
    };

    let mut queue = None;
    for field in fields {
        for attr in &field.attrs {
            if !attr.path().is_ident("aggregate") {
                continue;
            }
            parse_queue_attribute(attr)?;
            if queue.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "AggregateRoot requires exactly one #[aggregate(events)] field",
                ));
            }
            queue = Some(field);
        }
    }

    let queue = queue.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "AggregateRoot requires exactly one #[aggregate(events)] field",
        )
    })?;
    let queue_name = queue.ident.as_ref().expect("named fields have names");
    let event_type = vec_element_type(&queue.ty)?.clone();
    let core = core_path()?;
    let name = &input.ident;
    let mut impl_generics = input.generics.clone();
    impl_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#event_type: #core::DomainEvent));
    let (impl_generics, ty_generics, where_clause) = impl_generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #core::AggregateRoot for #name #ty_generics #where_clause {
            type Event = #event_type;

            fn record_event(&mut self, event: Self::Event) {
                self.#queue_name.push(event);
            }

            fn pull_events(&mut self) -> ::std::vec::Vec<Self::Event> {
                ::core::mem::take(&mut self.#queue_name)
            }
        }
    })
}

fn parse_queue_attribute(attr: &syn::Attribute) -> Result<()> {
    let mut found = false;
    attr.parse_nested_meta(|meta| {
        if found {
            return Err(meta.error("aggregate accepts only the events flag"));
        }
        if !meta.path.is_ident("events") {
            return Err(meta.error("aggregate accepts only the events flag"));
        }
        if !meta.input.is_empty() {
            return Err(meta.error("events is a flag and takes no value"));
        }
        found = true;
        Ok(())
    })?;
    if !found {
        return Err(syn::Error::new_spanned(
            attr,
            "aggregate requires the events flag",
        ));
    }
    Ok(())
}

fn vec_element_type(ty: &Type) -> Result<&Type> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "aggregate event queue must have type Vec<Event>",
        ));
    };
    let segments = &type_path.path.segments;
    let is_standard_vec = type_path.qself.is_none()
        && match segments.len() {
            1 => {
                segments[0].ident == "Vec"
                    && matches!(&segments[0].arguments, PathArguments::AngleBracketed(_))
            }
            3 => {
                segments[0].ident == "std"
                    && segments[1].ident == "vec"
                    && matches!(&segments[0].arguments, PathArguments::None)
                    && matches!(&segments[1].arguments, PathArguments::None)
                    && segments[2].ident == "Vec"
                    && matches!(&segments[2].arguments, PathArguments::AngleBracketed(_))
            }
            _ => false,
        };
    if !is_standard_vec {
        return Err(syn::Error::new_spanned(
            ty,
            "aggregate event queue must have type Vec<Event>",
        ));
    }
    let segment = &segments[segments.len() - 1];
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            "aggregate event queue must have type Vec<Event>",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            ty,
            "aggregate event queue must have exactly one event type",
        ));
    }
    match arguments.args.first() {
        Some(GenericArgument::Type(event_type)) => Ok(event_type),
        _ => Err(syn::Error::new_spanned(
            ty,
            "aggregate event queue must have a type event argument",
        )),
    }
}
