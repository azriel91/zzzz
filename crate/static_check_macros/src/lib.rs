use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Ident, Path};

use self::lit_str_maybe::LitStrMaybe;

mod lit_str_maybe;

/// Returns a `const AppName` validated at compile time.
///
/// This defaults to the crate name.
///
/// # Examples
///
/// Instantiate a valid `AppName` at compile time:
///
/// ```rust
/// # use peace_static_check_macros::app_name;
/// // use peace::cfg::{app_name, AppName};
///
/// let _my_flow: AppName = app_name!("valid_id"); // Ok!
///
/// # struct AppName(&'static str);
/// # impl AppName {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
///
/// If the ID is invalid, a compilation error is produced:
///
/// ```rust,compile_fail
/// # use peace_static_check_macros::app_name;
/// // use peace::cfg::{app_name, AppName};
///
/// let _my_flow: AppName = app_name!("-invalid"); // Compile error
/// //                     ^^^^^^^^^^^^^^^^^^^^^^^
/// // error: "-invalid" is not a valid `AppName`.
/// //        `AppName`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.
/// #
/// # struct AppName(&'static str);
/// # impl AppName {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
#[proc_macro]
pub fn app_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let crate_name = std::env::var("CARGO_PKG_NAME")
        .expect("Failed to read `CARGO_PKG_NAME` environmental variable to infer `AppName`.");
    ensure_valid_id(
        parse_quote!(peace::cfg),
        &parse_macro_input!(input as LitStrMaybe),
        "AppName",
        Some(crate_name.as_str()),
    )
    .into()
}

/// Returns a `const ItemId` validated at compile time.
///
/// # Examples
///
/// Instantiate a valid `ItemId` at compile time:
///
/// ```rust
/// # use peace_static_check_macros::item_id;
/// // use peace::item_model::{item_id, ItemId};
///
/// let _my_item_id: ItemId = item_id!("valid_id"); // Ok!
///
/// # struct ItemId(&'static str);
/// # impl ItemId {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
///
/// If the ID is invalid, a compilation error is produced:
///
/// ```rust,compile_fail
/// # use peace_static_check_macros::item_id;
/// // use peace::item_model::{item_id, ItemId};
///
/// let _my_item_id: ItemId = item_id!("-invalid_id"); // Compile error
/// //                        ^^^^^^^^^^^^^^^^^^^^^^^^
/// // error: "-invalid_id" is not a valid `ItemId`.
/// //        `ItemId`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.
/// #
/// # struct ItemId(&'static str);
/// # impl ItemId {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
#[proc_macro]
pub fn item_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ensure_valid_id(
        parse_quote!(peace::item_model),
        &parse_macro_input!(input as LitStrMaybe),
        "ItemId",
        None,
    )
    .into()
}

/// Returns a `const Profile` validated at compile time.
///
/// # Examples
///
/// Instantiate a valid `Profile` at compile time:
///
/// ```rust
/// # use peace_static_check_macros::profile;
/// // use peace::profile_model::{profile, Profile};
///
/// let _my_profile: Profile = profile!("valid_id"); // Ok!
///
/// # struct Profile(&'static str);
/// # impl Profile {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
///
/// If the ID is invalid, a compilation error is produced:
///
/// ```rust,compile_fail
/// # use peace_static_check_macros::profile;
/// // use peace::profile_model::{profile, Profile};
///
/// let _my_profile: Profile = profile!("-invalid_id"); // Compile error
/// //                         ^^^^^^^^^^^^^^^^^^^^^^^
/// // error: "-invalid_id" is not a valid `Profile`.
/// //        `Profile`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.
/// #
/// # struct Profile(&'static str);
/// # impl Profile {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
#[proc_macro]
pub fn profile(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ensure_valid_id(
        parse_quote!(peace::profile_model),
        &parse_macro_input!(input as LitStrMaybe),
        "Profile",
        None,
    )
    .into()
}

/// Returns a `const FlowId` validated at compile time.
///
/// # Examples
///
/// Instantiate a valid `FlowId` at compile time:
///
/// ```rust
/// # use peace_static_check_macros::flow_id;
/// // use peace::flow_model::{flow_id, FlowId};
///
/// let _my_flow: FlowId = flow_id!("valid_id"); // Ok!
///
/// # struct FlowId(&'static str);
/// # impl FlowId {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
///
/// If the ID is invalid, a compilation error is produced:
///
/// ```rust,compile_fail
/// # use peace_static_check_macros::flow_id;
/// // use peace::flow_model::{flow_id, FlowId};
///
/// let _my_flow: FlowId = flow_id!("-invalid_id"); // Compile error
/// //                     ^^^^^^^^^^^^^^^^^^^^^^^
/// // error: "-invalid_id" is not a valid `FlowId`.
/// //        `FlowId`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.
/// #
/// # struct FlowId(&'static str);
/// # impl FlowId {
/// #     fn new_unchecked(s: &'static str) -> Self { Self(s) }
/// # }
/// ```
#[proc_macro]
pub fn flow_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ensure_valid_id(
        parse_quote!(peace::flow_model),
        &parse_macro_input!(input as LitStrMaybe),
        "FlowId",
        None,
    )
    .into()
}

fn ensure_valid_id(
    crate_path: Path,
    proposed_id: &LitStrMaybe,
    ty_name: &str,
    default: Option<&str>,
) -> proc_macro2::TokenStream {
    let proposed_id = proposed_id.as_ref().map(|lit_str| lit_str.value());
    let proposed_id = proposed_id.as_deref().or(default);

    if let Some(proposed_id) = proposed_id {
        if is_valid_id(proposed_id) {
            let ty_name = Ident::new(ty_name, Span::call_site());
            quote!( #crate_path:: #ty_name ::new_unchecked( #proposed_id ))
        } else {
            let message = format!(
                "\"{proposed_id}\" is not a valid `{ty_name}`.\n\
                `{ty_name}`s must begin with a letter or underscore, and contain only letters, numbers, or underscores."
            );
            compile_fail(message)
        }
    } else {
        let message = format!(
            "`` is not a valid `{ty_name}`.\n\
            `{ty_name}`s must begin with a letter or underscore, and contain only letters, numbers, or underscores."
        );
        compile_fail(message)
    }
}

fn compile_fail(message: String) -> proc_macro2::TokenStream {
    quote!(compile_error!(#message))
}

fn is_valid_id(proposed_id: &str) -> bool {
    let mut chars = proposed_id.chars();
    let first_char = chars.next();
    let first_char_valid = first_char
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false);
    let remainder_chars_valid =
        chars.all(|c| c.is_ascii_alphabetic() || c == '_' || c.is_ascii_digit());

    first_char_valid && remainder_chars_valid
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::{parse_quote, LitStr};

    use crate::LitStrMaybe;

    use super::ensure_valid_id;

    #[test]
    fn name_beginning_with_underscore_is_valid() {
        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("_", Span::call_site()))),
            "Ty",
            None,
        );

        assert_eq!(
            r#"peace :: cfg :: Ty :: new_unchecked ("_")"#,
            tokens.to_string()
        );
    }

    #[test]
    fn name_beginning_with_alpha_is_valid() {
        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("a", Span::call_site()))),
            "Ty",
            None,
        );
        assert_eq!(
            r#"peace :: cfg :: Ty :: new_unchecked ("a")"#,
            tokens.to_string()
        );

        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("A", Span::call_site()))),
            "Ty",
            None,
        );

        assert_eq!(
            r#"peace :: cfg :: Ty :: new_unchecked ("A")"#,
            tokens.to_string()
        );
    }

    #[test]
    fn name_beginning_with_number_is_invalid() {
        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("1", Span::call_site()))),
            "Ty",
            None,
        );

        assert_eq!(
            "compile_error ! (\"\\\"1\\\" is not a valid `Ty`.\\n\
            `Ty`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.\")",
            tokens.to_string()
        );
    }

    #[test]
    fn name_containing_space_is_invalid() {
        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("a b", Span::call_site()))),
            "Ty",
            None,
        );

        assert_eq!(
            "compile_error ! (\"\\\"a b\\\" is not a valid `Ty`.\\n\
            `Ty`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.\")",
            tokens.to_string()
        );
    }

    #[test]
    fn name_containing_hyphen_is_invalid() {
        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("a-b", Span::call_site()))),
            "Ty",
            None,
        );

        assert_eq!(
            "compile_error ! (\"\\\"a-b\\\" is not a valid `Ty`.\\n\
            `Ty`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.\")",
            tokens.to_string()
        );
    }

    #[test]
    fn name_empty_string_is_invalid() {
        let tokens = ensure_valid_id(
            parse_quote!(peace::cfg),
            &LitStrMaybe(Some(LitStr::new("", Span::call_site()))),
            "Ty",
            None,
        );

        assert_eq!(
            "compile_error ! (\"\\\"\\\" is not a valid `Ty`.\\n\
            `Ty`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.\")",
            tokens.to_string()
        );
    }

    #[test]
    fn name_none_is_invalid() {
        let tokens = ensure_valid_id(parse_quote!(peace::cfg), &LitStrMaybe(None), "Ty", None);

        assert_eq!(
            "compile_error ! (\"`` is not a valid `Ty`.\\n\
            `Ty`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.\")",
            tokens.to_string()
        );
    }
}
