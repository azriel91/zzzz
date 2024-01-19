use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, FieldValue, Token};

use crate::cmd::{CmdCtxBuilderReturnTypeBuilder, FlowCount, ScopeStruct};

/// Generates the constructor for the command context builder for a given scope.
pub fn impl_constructor(scope_struct: &ScopeStruct) -> proc_macro2::TokenStream {
    let scope = scope_struct.scope();
    let scope_builder_name = &scope_struct.item_struct().ident;
    let constructor_method_name = Ident::new(scope.as_str(), Span::call_site());

    let scope_field_values = {
        let mut type_params = Punctuated::<FieldValue, Token![,]>::new();

        scope_field_values::profile_and_flow_selection_push(&mut type_params, scope);
        scope_field_values::params_selection_push(&mut type_params, scope);

        if scope.flow_count() == FlowCount::One {
            type_params.push(parse_quote!(
                params_specs_provided: peace_params::ParamsSpecs::new()
            ));
        }

        type_params
    };

    // crate::ctx::CmdCtxBuilder<
    //     'ctx,
    //     crate::ctx::CmdCtxBuilderTypeParamsCollector<
    //         O,
    //         AppError,
    //         peace_rt_model::params::ParamsKeysImpl<
    //             peace_rt_model::params::KeyUnknown,
    //             peace_rt_model::params::KeyUnknown,
    //             peace_rt_model::params::KeyUnknown,
    //         >,
    //         crate::scopes::type_params::WorkspaceParamsNone,
    //         crate::scopes::type_params::ProfileParamsNone,
    //         crate::scopes::type_params::FlowParamsNone,
    //         crate::scopes::type_params::ProfileNotSelected,
    //         crate::scopes::type_params::FlowNotSelected,
    //     >,
    // >

    let return_type = CmdCtxBuilderReturnTypeBuilder::new(scope_builder_name.clone())
        .with_output(parse_quote!(O))
        .with_app_error(parse_quote!(AppError))
        .with_workspace_params_k(parse_quote!(peace_rt_model::params::KeyUnknown))
        .with_profile_params_k(parse_quote!(peace_rt_model::params::KeyUnknown))
        .with_flow_params_k(parse_quote!(peace_rt_model::params::KeyUnknown))
        .with_workspace_params_selection(parse_quote!(
            crate::scopes::type_params::WorkspaceParamsNone
        ))
        .with_profile_params_selection(parse_quote!(crate::scopes::type_params::ProfileParamsNone))
        .with_flow_params_selection(parse_quote!(crate::scopes::type_params::FlowParamsNone))
        .with_profile_selection(parse_quote!(crate::scopes::type_params::ProfileNotSelected))
        .with_flow_selection(parse_quote!(crate::scopes::type_params::FlowNotSelected))
        .build();

    quote! {
        impl<'ctx, O, AppError> #return_type {
            /// Returns a `CmdCtxBuilder` for a single profile and flow.
            pub fn #constructor_method_name(
                output: &'ctx mut O,
                workspace: &'ctx peace_rt_model::Workspace,
            ) -> Self
            {
                let scope_builder = #scope_builder_name {
                    // profile_selection: ProfileNotSelected,
                    // flow_selection: FlowNotSelected,
                    // params_type_regs_builder:
                    //     peace_rt_model::params::ParamsTypeRegsBuilder::<
                    //         peace_rt_model::params::ParamsKeysImpl<
                    //             peace_rt_model::params::KeyUnknown,
                    //             peace_rt_model::params::KeyUnknown,
                    //             peace_rt_model::params::KeyUnknown,
                    //         >
                    //     >::new(),
                    // workspace_params_selection: WorkspaceParamsNone,
                    // profile_params_selection: ProfileParamsNone,

                    // // === SingleProfileSingleFlow === //
                    // params_specs_provided: peace_params::ParamsSpecs::new()

                    #scope_field_values
                };

                Self {
                    output,
                    interruptibility: interruptible::Interruptibility::NonInterruptible,
                    workspace,
                    scope_builder,
                }
            }
        }
    }
}

mod scope_field_values {
    use syn::{parse_quote, punctuated::Punctuated, FieldValue, Token};

    use crate::cmd::{FlowCount, ProfileCount, Scope};

    /// Appends profile / flow ID selection type parameters if applicable to the
    /// given scope.
    pub fn profile_and_flow_selection_push(
        field_values: &mut Punctuated<FieldValue, Token![,]>,
        scope: Scope,
    ) {
        match scope.profile_count() {
            ProfileCount::None => {}
            ProfileCount::One | ProfileCount::Multiple => {
                // Even for `ProfileCount::Multiple`, we will store the filter function in
                // the profile selection.
                field_values.push(parse_quote!(
                    profile_selection: crate::scopes::type_params::ProfileNotSelected
                ));
            }
        }
        if scope.flow_count() == FlowCount::One {
            field_values.push(parse_quote!(
                flow_selection: crate::scopes::type_params::FlowNotSelected
            ));
        }
    }

    /// Appends workspace / profile / flow params selection type parameters if
    /// applicable to the given scope.
    pub fn params_selection_push(
        field_values: &mut Punctuated<FieldValue, Token![,]>,
        scope: Scope,
    ) {
        // Workspace params are supported by all scopes.
        field_values.push(parse_quote! {
            params_type_regs_builder:
                peace_rt_model::params::ParamsTypeRegsBuilder::<
                    peace_rt_model::params::ParamsKeysImpl<
                        peace_rt_model::params::KeyUnknown,
                        peace_rt_model::params::KeyUnknown,
                        peace_rt_model::params::KeyUnknown,
                    >
                >::new()
        });

        // Workspace params are supported by all scopes.
        field_values.push(parse_quote!(
            workspace_params_selection: crate::scopes::type_params::WorkspaceParamsNone
        ));

        if scope.profile_params_supported() {
            field_values.push(parse_quote!(
                profile_params_selection: crate::scopes::type_params::ProfileParamsNone
            ));
        }

        if scope.flow_params_supported() {
            field_values.push(parse_quote!(
                flow_params_selection: crate::scopes::type_params::FlowParamsNone
            ));
        }
    }
}
