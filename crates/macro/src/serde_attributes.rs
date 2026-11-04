use syn::{Attribute, Data, DeriveInput, Fields, LitStr, Result};

/// Naming options supported by the generated serializers in both directions.
#[derive(Default)]
pub(crate) struct Naming {
    rename: Option<LitStr>,
    rename_all: Option<LitStr>,
}

impl Naming {
    pub(crate) fn parse(attrs: &[Attribute], allow_rule: bool) -> Result<Self> {
        let mut naming = Self::default();
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
            let mut found = false;
            attr.parse_nested_meta(|meta| {
                found = true;
                let slot = if meta.path.is_ident("rename") {
                    &mut naming.rename
                } else if allow_rule && meta.path.is_ident("rename_all") {
                    &mut naming.rename_all
                } else {
                    return Err(meta.error("unsupported serde attribute with generated serde; supported naming options are rename and, on containers or struct variants, rename_all"));
                };
                if slot.is_some() {
                    return Err(meta.error("duplicate serde naming attribute"));
                }
                let value: LitStr = meta.value()?.parse()?;
                if meta.path.is_ident("rename_all")
                    && !matches!(value.value().as_str(), "lowercase" | "UPPERCASE" | "PascalCase" | "camelCase" | "snake_case" | "SCREAMING_SNAKE_CASE" | "kebab-case" | "SCREAMING-KEBAB-CASE")
                {
                    return Err(syn::Error::new_spanned(value, "unsupported serde rename_all rule"));
                }
                *slot = Some(value);
                Ok(())
            })?;
            if !found {
                return Err(syn::Error::new_spanned(
                    attr,
                    "serde requires a naming option",
                ));
            }
        }
        Ok(naming)
    }

    pub(crate) fn label(&self, ident: &syn::Ident, parent: &Self, variant: bool) -> LitStr {
        use syn::ext::IdentExt;
        if let Some(rename) = &self.rename {
            return rename.clone();
        }
        let name = ident.unraw().to_string();
        let rule = parent.rename_all.as_ref().map(|rule| rule.value());
        let snake = || {
            let mut result = String::new();
            for (index, ch) in name.char_indices() {
                if index > 0 && ch.is_uppercase() {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
            }
            result
        };
        let pascal = || {
            let mut result = String::new();
            let mut capitalize = true;
            for ch in name.chars() {
                if ch == '_' {
                    capitalize = true;
                } else {
                    result.push(if capitalize {
                        ch.to_ascii_uppercase()
                    } else {
                        ch
                    });
                    capitalize = false;
                }
            }
            result
        };
        let camel = |value: String| {
            let mut chars = value.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
                None => value,
            }
        };
        let label = match rule.as_deref() {
            Some("lowercase") if variant => name.to_ascii_lowercase(),
            Some("UPPERCASE") => name.to_ascii_uppercase(),
            Some("PascalCase") if !variant => pascal(),
            Some("camelCase") => camel(if variant { name.clone() } else { pascal() }),
            Some("snake_case") if variant => snake(),
            Some("SCREAMING_SNAKE_CASE") => {
                if variant { snake() } else { name.clone() }.to_ascii_uppercase()
            }
            Some("kebab-case") => if variant { snake() } else { name.clone() }.replace('_', "-"),
            Some("SCREAMING-KEBAB-CASE") => if variant { snake() } else { name.clone() }
                .to_ascii_uppercase()
                .replace('_', "-"),
            _ => name,
        };
        LitStr::new(&label, ident.span())
    }

    pub(crate) fn type_label(&self, ident: &syn::Ident) -> LitStr {
        self.rename.clone().unwrap_or_else(|| {
            use syn::ext::IdentExt;
            LitStr::new(&ident.unraw().to_string(), ident.span())
        })
    }
}

pub(crate) fn serde_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("serde"))
}

pub(crate) fn validate(
    input: &DeriveInput,
    generated: bool,
    transparent_single: bool,
) -> Result<()> {
    if !generated {
        // Independent Serde derives own these attributes in non-generated modes.
        return Ok(());
    }
    let container_rules = matches!(&input.data, Data::Enum(_))
        || matches!(&input.data, Data::Struct(data) if matches!(&data.fields, Fields::Named(fields) if !transparent_single || fields.named.len() > 1));
    if !container_rules && let Some(attr) = serde_attrs(&input.attrs).next() {
        return Err(syn::Error::new_spanned(
            attr,
            "serde container attributes on transparent or tuple value objects are unsupported with generated serde",
        ));
    }
    Naming::parse(&input.attrs, container_rules)?;
    let check_fields = |fields: &Fields, transparent: bool| -> Result<()> {
        for field in fields {
            if transparent || !matches!(fields, Fields::Named(_)) {
                if let Some(attr) = serde_attrs(&field.attrs).next() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "serde attributes on transparent or tuple fields are unsupported with generated serde",
                    ));
                }
            } else {
                Naming::parse(&field.attrs, false)?;
            }
        }
        Ok(())
    };
    match &input.data {
        Data::Struct(data) => {
            check_fields(&data.fields, transparent_single && data.fields.len() == 1)?
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                Naming::parse(&variant.attrs, matches!(variant.fields, Fields::Named(_)))?;
                check_fields(&variant.fields, false)?;
            }
        }
        Data::Union(_) => {}
    }
    Ok(())
}
