use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Result;

/// **English:** Resolves the calling crate's `ddd-toolkit-core` dependency,
/// preserving a dependency rename; returns a call-site error when it is absent.
///
/// **日本語:** 呼び出し元の `ddd-toolkit-core` 依存を別名も維持して解決し、
/// 依存がない場合は呼び出し位置を示すエラーを返します。
pub fn core_path() -> Result<TokenStream> {
    let package = "ddd-toolkit-core";
    if let Ok(found) = proc_macro_crate::crate_name(package) {
        if let Some(path) = path_for_found_crate(package, found) {
            return Ok(path);
        }
    }

    Err(syn::Error::new(
        Span::call_site(),
        "DDD derives require a ddd or ddd-core dependency",
    ))
}

fn path_for_found_crate(package: &str, found: proc_macro_crate::FoundCrate) -> Option<TokenStream> {
    match found {
        proc_macro_crate::FoundCrate::Itself if package == "ddd" => None,
        proc_macro_crate::FoundCrate::Itself => {
            let ident = Ident::new(&package.replace('-', "_"), Span::call_site());
            Some(quote!(::#ident))
        }
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            Some(quote!(::#ident))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::path_for_found_crate;
    use proc_macro_crate::FoundCrate;

    #[test]
    fn skips_facade_itself_and_falls_back_to_core() {
        assert!(path_for_found_crate("ddd", FoundCrate::Itself).is_none());
        assert_eq!(
            path_for_found_crate("ddd-core", FoundCrate::Itself)
                .unwrap()
                .to_string(),
            ":: ddd_core"
        );
    }

    #[test]
    fn preserves_renamed_facade_dependency() {
        assert_eq!(
            path_for_found_crate("ddd", FoundCrate::Name("facade".into()))
                .unwrap()
                .to_string(),
            ":: facade"
        );
    }
}
