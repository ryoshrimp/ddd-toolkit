use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Result;

/// **English:**<br>
/// Resolves a direct `ddd-toolkit-core` dependency first, then `ddd-toolkit`,
/// preserving dependency renames; reports an error if neither is available.
///
/// **日本語:**<br>
/// 直接依存の `ddd-toolkit-core`、次に `ddd-toolkit` を別名も維持して解決し、
/// どちらも存在しない場合はエラーを返します。
pub fn core_path() -> Result<TokenStream> {
    for package in ["ddd-toolkit-core", "ddd-toolkit"] {
        if let Ok(found) = proc_macro_crate::crate_name(package) {
            return Ok(path_for_found_crate(package, found));
        }
    }

    Err(syn::Error::new(
        Span::call_site(),
        "DDD derives require a ddd-toolkit-core or ddd-toolkit dependency; add one to [dependencies] in Cargo.toml",
    ))
}

fn path_for_found_crate(package: &str, found: proc_macro_crate::FoundCrate) -> TokenStream {
    let name = match found {
        // Use the library name, not `crate`: derives can also run in this
        // package's integration tests and doctests. Libraries alias themselves.
        proc_macro_crate::FoundCrate::Itself => package.replace('-', "_"),
        proc_macro_crate::FoundCrate::Name(name) => name,
    };
    let ident = Ident::new(&name, Span::call_site());
    quote!(::#ident)
}

#[cfg(test)]
mod tests {
    use super::path_for_found_crate;
    use proc_macro_crate::FoundCrate;

    #[test]
    fn uses_library_names_for_self_references() {
        for (package, expected) in [
            ("ddd-toolkit-core", ":: ddd_toolkit_core"),
            ("ddd-toolkit", ":: ddd_toolkit"),
        ] {
            assert_eq!(
                path_for_found_crate(package, FoundCrate::Itself).to_string(),
                expected,
            );
        }
    }

    #[test]
    fn preserves_renamed_dependencies() {
        for package in ["ddd-toolkit-core", "ddd-toolkit"] {
            assert_eq!(
                path_for_found_crate(package, FoundCrate::Name("renamed".into())).to_string(),
                ":: renamed",
            );
        }
    }
}
