#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, DataStruct, DeriveInput, Field, Fields,
    FieldsNamed, FieldsUnnamed, Ident, Lifetime, Type, TypePath, WhereClause, WherePredicate,
};

/// Used to `#[derive]` the `Data` trait.
///
/// For regular usage, use `#[derive(Data)]`
///
/// For peace crates, also add the `#[peace_internal]` attribute, which
/// references the `peace_data` crate instead of the `peace::data` re-export.
#[proc_macro_derive(Data, attributes(peace_internal))]
pub fn data_access(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Data derive: Code failed to be parsed.");

    let gen = impl_data_access(&ast);

    gen.into()
}

fn impl_data_access(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (peace_data_path, peace_item_model_path) = ast
        .attrs
        .iter()
        .find(peace_internal)
        .map(
            #[cfg_attr(coverage_nightly, coverage(off))]
            |_| (quote!(peace_data), quote!(peace_item_model)),
        )
        .unwrap_or_else(|| (quote!(peace::data), quote!(peace::item_model)));

    let mut generics = ast.generics.clone();

    let (tys, field_names, borrow_return) = data_borrow_impl(&ast.data, name);
    let tys = &tys;
    // Assumes that the first lifetime is the borrow lifetime
    let def_borrow_lt = ast
        .generics
        .lifetimes()
        .next()
        .expect("Struct must have at least one lifetime");
    let impl_borrow_lt = &def_borrow_lt.lifetime;

    {
        let where_clause = generics.make_where_clause();
        constrain_data_access_types(where_clause, impl_borrow_lt, tys);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #peace_data_path::DataAccess
            for #name #ty_generics
            #where_clause
        {
            fn borrows() -> #peace_data_path::TypeIds {
                let mut r = #peace_data_path::TypeIds::new();

                #( {
                        let mut borrows = <#tys as #peace_data_path::DataAccess>::borrows();
                        r.append(&mut borrows);
                    } )*

                r
            }

            fn borrow_muts() -> #peace_data_path::TypeIds {
                let mut r = #peace_data_path::TypeIds::new();

                #( {
                        let mut borrow_muts = <#tys as #peace_data_path::DataAccess>::borrow_muts();
                        r.append(&mut borrow_muts);
                    } )*

                r
            }
        }

        impl #impl_generics #peace_data_path::DataAccessDyn
            for #name #ty_generics
            #where_clause
        {
            fn borrows(&self) -> #peace_data_path::TypeIds {
                let mut r = #peace_data_path::TypeIds::new();

                #( {
                        let mut borrows = <#tys as #peace_data_path::DataAccessDyn>::borrows(&self.#field_names);
                        r.append(&mut borrows);
                    } )*

                r
            }

            fn borrow_muts(&self) -> #peace_data_path::TypeIds {
                let mut r = #peace_data_path::TypeIds::new();

                #( {
                        let mut borrow_muts = <#tys as #peace_data_path::DataAccessDyn>::borrow_muts(&self.#field_names);
                        r.append(&mut borrow_muts);
                    } )*

                r
            }
        }

        impl #impl_generics #peace_data_path::Data< #impl_borrow_lt >
            for #name #ty_generics
            #where_clause
        {
            fn borrow(item_id: & #impl_borrow_lt #peace_item_model_path::ItemId, resources: & #impl_borrow_lt #peace_data_path::Resources) -> Self {
                #borrow_return
            }
        }
    }
}

fn peace_internal(attr: &&Attribute) -> bool {
    attr.path().is_ident("peace_internal")
}

/// Adds a `Data<'lt>` bound on each of the system data types.
fn constrain_data_access_types(clause: &mut WhereClause, borrow_lt: &Lifetime, tys: &[&Type]) {
    for ty in tys.iter() {
        let where_predicate: WherePredicate = parse_quote!(#ty : Data< #borrow_lt >);
        clause.predicates.push(where_predicate);
    }
}

fn data_borrow_impl<'ast>(
    ast: &'ast syn::Data,
    name: &Ident,
) -> (
    Vec<&'ast Type>,
    Vec<proc_macro2::TokenStream>,
    proc_macro2::TokenStream,
) {
    enum DataType {
        Struct,
        Tuple,
    }

    let (data_type, fields) = match ast {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => (DataType::Struct, named),

        syn::Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => (DataType::Tuple, unnamed),

        _ => ({
            #[cfg_attr(coverage_nightly, coverage(off))]
            || -> ! { panic!("Enums are not supported") }
        })(),
    };

    let tys = field_types(fields);

    let (field_names_tokens, borrow_return) = match data_type {
        DataType::Struct => {
            let field_names = field_names(fields);

            let field_names_tokens = field_names
                .normal_fields
                .iter()
                .map(|ident| quote!(#ident))
                .collect::<Vec<_>>();
            let phantom_data_fields = &field_names.phantom_data_fields;

            let borrow_return = quote! {
                #name {
                    #( #field_names_tokens: Data::borrow(item_id, resources) ),*
                    #(, #phantom_data_fields: ::std::marker::PhantomData)*
                }
            };

            (field_names_tokens, borrow_return)
        }
        DataType::Tuple => {
            let count = tys.len();
            let field_names_tokens = (0..count)
                .map(Literal::usize_unsuffixed)
                .map(|n| quote!(#n))
                .collect::<Vec<_>>();

            let borrow = vec![quote! { Data::borrow(item_id, resources) }; count];
            let borrow_return = quote! {
                #name ( #( #borrow ),* )
            };

            (field_names_tokens, borrow_return)
        }
    };

    (tys, field_names_tokens, borrow_return)
}

fn field_types(fields: &Punctuated<Field, Comma>) -> Vec<&Type> {
    fields
        .iter()
        .filter_map(|field| {
            if !is_phantom_data(field) {
                Some(&field.ty)
            } else {
                None
            }
        })
        .collect()
}

fn field_names(fields: &Punctuated<Field, Comma>) -> FieldNames<'_> {
    fields
        .iter()
        .fold(FieldNames::default(), |mut field_names, field| {
            if is_phantom_data(field) {
                if let Some(field_name) = field.ident.as_ref() {
                    field_names.phantom_data_fields.push(field_name);
                }
            } else if let Some(field_name) = field.ident.as_ref() {
                field_names.normal_fields.push(field_name);
            }

            field_names
        })
}

fn is_phantom_data(field: &Field) -> bool {
    matches!(&field.ty, Type::Path(TypePath { path, .. })
        if matches!(path.segments.last(), Some(segment) if segment.ident == "PhantomData"))
}

#[derive(Default)]
struct FieldNames<'field> {
    normal_fields: Vec<&'field Ident>,
    phantom_data_fields: Vec<&'field Ident>,
}
