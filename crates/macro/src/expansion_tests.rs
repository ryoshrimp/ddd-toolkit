//! **English:**<br>
//! Tests expansion branches directly, including diagnostics that do not
//! run inside the coverage-instrumented process when invoked through trybuild.
//!
//! **日本語:**<br>
//! trybuild 経由ではカバレッジ計測プロセス内で実行されない診断も含め、
//! マクロ展開の分岐を直接検証します。

use crate::{aggregate_root, entity, value_object};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

type Expander = fn(DeriveInput) -> Result<TokenStream>;

fn rejects(expand: Expander, cases: &[(&str, &str)]) {
    for (source, expected) in cases {
        let input = syn::parse_str(source).unwrap();
        let error = expand(input).unwrap_err().to_string();
        assert!(error.contains(expected), "{source}: {error}");
    }
}

#[test]
fn value_object_expands_all_shapes_with_validation_and_serde() {
    let shapes = [
        quote!(
            struct Value<T>(T);
        ),
        quote!(
            struct Value<T> {
                value: T,
            }
        ),
        quote!(
            struct Value<T>(T, u32);
        ),
        quote!(
            struct Value<T> {
                value: T,
                count: u32,
            }
        ),
        quote!(
            enum Value<T> {
                Unit,
                Newtype(T),
                Tuple(T, u32),
                Named { value: T },
            }
        ),
    ];
    for shape in shapes {
        for validated in [false, true] {
            for serde in [false, true] {
                let attributes = match (validated, serde) {
                    (false, false) => quote!(),
                    (true, false) => quote!(#[value_object(validate = check, error = Error)]),
                    (false, true) => quote!(#[value_object(serde)]),
                    (true, true) => quote!(#[value_object(validate = check, error = Error, serde)]),
                };
                let input = syn::parse2(quote!(#attributes #shape)).unwrap();
                let output = value_object::expand(input).unwrap().to_string();
                for implementation in [
                    "ValueObject for Value",
                    "Debug for Value",
                    "Clone for Value",
                    "Hash for Value",
                    "as_inner",
                    "into_inner",
                ] {
                    // Generic bounds sit between `impl` and the trait name.
                    assert!(
                        output.contains(implementation),
                        "{implementation}: {output}"
                    );
                }
                assert_eq!(output.contains("try_new"), validated);
                assert_eq!(output.contains("Serialize for Value"), serde);
                assert_eq!(output.contains("Deserialize <"), serde);
                if validated {
                    assert!(output.contains("check (&"));
                    if serde {
                        assert!(output.contains("value object validation failed"));
                    }
                }
            }
        }
    }
}

#[test]
fn serde_avoids_existing_deserialization_lifetimes() {
    let input = syn::parse2(quote! {
        #[value_object(serde)]
        struct Borrowed<'__ddd_de, '__ddd_dex, T>(&'__ddd_de T, &'__ddd_dex T);
    })
    .unwrap();
    let output = value_object::expand(input).unwrap().to_string();
    assert!(output.contains("'__ddd_dexx"));
    assert!(output.contains("'__ddd_de"));
}

#[test]
fn value_object_reports_invalid_shapes_and_options() {
    rejects(
        value_object::expand,
        &[
            ("struct Empty;", "at least one field"),
            ("struct Empty {}", "at least one field"),
            ("struct Empty();", "at least one field"),
            ("enum Empty {}", "at least one variant"),
            ("union Value { a: u32 }", "struct or enum"),
            ("enum Value { AB, Ab }", "unique after lower-casing"),
            ("#[value_object()] struct V(u32);", "at least one option"),
            (
                "#[value_object(other)] struct V(u32);",
                "unknown value_object attribute",
            ),
            (
                "#[value_object(serde = true)] struct V(u32);",
                "serde is a flag",
            ),
            (
                "#[value_object(serde)] #[value_object(serde)] struct V(u32);",
                "only once",
            ),
            (
                "#[value_object(validate = check)] struct V(u32);",
                "specified together",
            ),
            (
                "#[value_object(error = Error)] struct V(u32);",
                "specified together",
            ),
            (
                "#[value_object(validate = a, validate = b)] struct V(u32);",
                "duplicate validate",
            ),
            (
                "#[value_object(error = A, error = B)] struct V(u32);",
                "duplicate error",
            ),
            (
                "#[value_object(validate = check, error = Error)] struct V(pub u32);",
                "inner fields must not be public",
            ),
            (
                "#[value_object(validate = check, error = Error)] struct V { pub a: u32, b: u32 }",
                "fields must not be public",
            ),
        ],
    );
}

#[test]
fn entity_expansion_uses_only_marked_identity() {
    let input = syn::parse2(quote! {
        struct Item<T> { #[allow(dead_code)] payload: T, #[entity(id)] key: u64 }
    })
    .unwrap();
    let output = entity::expand(input).unwrap().to_string();
    assert!(output.contains("type Id = u64"));
    assert!(output.contains("self . key == other . key"));
    assert!(output.contains("Hash :: hash (& self . key"));
    assert!(!output.contains("self . payload"));
}

#[test]
fn entity_reports_attribute_and_shape_errors() {
    rejects(
        entity::expand,
        &[
            (
                "#[entity(id)] struct V { id: u64 }",
                "placed on a named field",
            ),
            ("struct V(u64);", "requires a named-field struct"),
            ("enum V { A }", "only be derived for a named-field struct"),
            ("struct V { id: u64 }", "exactly one"),
            (
                "struct V { #[entity(id)] a: u64, #[entity(id)] b: u64 }",
                "exactly one",
            ),
            ("struct V { #[entity()] a: u64 }", "requires the id flag"),
            (
                "struct V { #[entity(other)] a: u64 }",
                "unknown entity attribute",
            ),
            ("struct V { #[entity(id = true)] a: u64 }", "id is a flag"),
            (
                "struct V { #[entity(id)] #[entity(id)] a: u64 }",
                "duplicate id",
            ),
        ],
    );
}

#[test]
fn aggregate_expansion_preserves_event_type_and_drains_queue() {
    for queue in [quote!(Vec<Event>), quote!(::std::vec::Vec<Event>)] {
        let input = syn::parse2(quote! {
            struct Root<Event> { #[allow(dead_code)] id: u64, #[aggregate(events)] pending: #queue }
        })
        .unwrap();
        let output = aggregate_root::expand(input).unwrap().to_string();
        assert!(output.contains("type Event = Event"));
        assert!(output.contains("self . pending . push (event)"));
        assert!(output.contains("mem :: take (& mut self . pending)"));
        assert!(output.contains("Event : :: ddd_toolkit_core :: DomainEvent"));
    }
}

#[test]
fn aggregate_reports_attribute_and_queue_errors() {
    rejects(
        aggregate_root::expand,
        &[
            (
                "#[aggregate(events)] struct V { q: Vec<E> }",
                "placed on an event queue field",
            ),
            ("struct V(Vec<E>);", "requires a named-field struct"),
            ("enum V { A }", "only be derived for a named-field struct"),
            ("struct V { q: Vec<E> }", "exactly one"),
            (
                "struct V { #[aggregate(events)] a: Vec<E>, #[aggregate(events)] b: Vec<E> }",
                "exactly one",
            ),
            (
                "struct V { #[aggregate()] a: Vec<E> }",
                "requires the events flag",
            ),
            (
                "struct V { #[aggregate(other)] a: Vec<E> }",
                "only the events flag",
            ),
            (
                "struct V { #[aggregate(events = true)] a: Vec<E> }",
                "events is a flag",
            ),
        ],
    );
    for queue in [
        "&[E]",
        "fake::Vec<E>",
        "Vec",
        "std::vec::Vec",
        "other::vec::Vec<E>",
        "Vec<E, E>",
        "Vec<'a>",
        "<T as Trait>::Vec<E>",
    ] {
        let source = format!("struct V {{ #[aggregate(events)] q: {queue} }}");
        let expected = if queue == "Vec<E, E>" {
            "exactly one event type"
        } else if queue == "Vec<'a>" {
            "type event argument"
        } else {
            "must have type Vec<Event>"
        };
        rejects(aggregate_root::expand, &[(source.as_str(), expected)]);
    }
}
