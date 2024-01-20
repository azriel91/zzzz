use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, token::Comma, FieldValue, Token};

use crate::cmd::{CmdCtxBuilderTypeBuilder, FlowCount, ProfileCount, Scope, ScopeStruct};

/// Generates the `with_profile_filter` method for the command context builder.
pub fn impl_with_profile_filter(scope_struct: &ScopeStruct) -> proc_macro2::TokenStream {
    let scope = scope_struct.scope();
    let scope_builder_name = &scope_struct.item_struct().ident;

    if scope_struct.scope().profile_count() != ProfileCount::Multiple {
        // `with_profile_filter` is not supported.
        return proc_macro2::TokenStream::new();
    };

    let scope_builder_fields_profile_not_selected =
        scope_builder_fields_profile_not_selected(scope);
    let scope_builder_fields_profile_filter_fn = scope_builder_fields_profile_filter_fn(scope);

    let return_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone())
        .with_profile_selection(parse_quote!(
            crate::scopes::type_params::ProfileFilterFn<'ctx>
        ))
        .build();

    quote! {
        impl<'ctx, CmdCtxBuilderTypeParamsT> crate::ctx::CmdCtxBuilder<
            'ctx,
            CmdCtxBuilderTypeParamsT,
            #scope_builder_name<CmdCtxBuilderTypeParamsT>,
        >
        where
            CmdCtxBuilderTypeParamsT: crate::ctx::CmdCtxBuilderTypeParams<
                ProfileSelection = crate::scopes::type_params::ProfileNotSelected,
            >,
        {
            pub fn with_profile_filter<F>(
                self,
                profile_filter_fn: F,
            ) ->
                // crate::ctx::CmdCtxBuilder<
                //     'ctx,
                //     crate::ctx::CmdCtxBuilderTypeParamsCollector<
                //         CmdCtxBuilderTypeParamsT::Output,
                //         CmdCtxBuilderTypeParamsT::AppError,
                //         peace_rt_model::params::ParamsKeysImpl<
                //             <CmdCtxBuilderTypeParamsT::ParamsKeys as ParamsKeys>::WorkspaceParamsKMaybe,
                //             <CmdCtxBuilderTypeParamsT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe,
                //             <CmdCtxBuilderTypeParamsT::ParamsKeys as ParamsKeys>::FlowParamsKMaybe,
                //         >,
                //         CmdCtxBuilderTypeParamsT::WorkspaceParamsSelection,
                //         CmdCtxBuilderTypeParamsT::ProfileParamsSelection,
                //         CmdCtxBuilderTypeParamsT::FlowParamsSelection,
                //         crate::scopes::type_params::ProfileFilterFn<'ctx>,
                //         CmdCtxBuilderTypeParamsT::FlowSelection,
                //     >,
                // >
                #return_type
            where
                F: (Fn(&peace_core::Profile) -> bool) + 'ctx
            {
                let Self {
                    output,
                    interruptibility,
                    workspace,
                    scope_builder:
                        #scope_builder_name {
                            // profile_selection: ProfileNotSelected,
                            // flow_selection,
                            // params_type_regs_builder,
                            // workspace_params_selection,
                            // profile_params_selection,
                            // flow_params_selection,
                            // params_specs_provided,
                            #scope_builder_fields_profile_not_selected
                        },
                } = self;

                let scope_builder = #scope_builder_name {
                    // profile_selection: ProfileFilterFn(Box::new(profile_filter_fn)),
                    // flow_selection,
                    // params_type_regs_builder,
                    // workspace_params_selection,
                    // profile_params_selection,
                    // flow_params_selection,
                    // params_specs_provided,
                    #scope_builder_fields_profile_filter_fn
                };

                crate::ctx::CmdCtxBuilder {
                    output,
                    interruptibility,
                    workspace,
                    scope_builder,
                }
            }
        }
    }
}

fn scope_builder_fields_profile_not_selected(scope: Scope) -> Punctuated<FieldValue, Comma> {
    let mut field_values = Punctuated::<FieldValue, Token![,]>::new();
    field_values.push(parse_quote!(
        profile_selection: crate::scopes::type_params::ProfileNotSelected
    ));
    scope_builder_fields_remainder_push(scope, &mut field_values);

    field_values
}

fn scope_builder_fields_profile_filter_fn(scope: Scope) -> Punctuated<FieldValue, Comma> {
    let mut field_values = Punctuated::<FieldValue, Token![,]>::new();
    field_values.push(parse_quote!(
        profile_selection: crate::scopes::type_params::ProfileFilterFn(Box::new(profile_filter_fn))
    ));
    scope_builder_fields_remainder_push(scope, &mut field_values);

    field_values
}

fn scope_builder_fields_remainder_push(
    scope: Scope,
    field_values: &mut Punctuated<FieldValue, Comma>,
) {
    if scope.flow_count() == FlowCount::One {
        field_values.push(parse_quote!(flow_selection));
    }
    field_values.push(parse_quote!(params_type_regs_builder));
    field_values.push(parse_quote!(workspace_params_selection));
    if scope.profile_params_supported() {
        field_values.push(parse_quote!(profile_params_selection));
    }
    if scope.flow_params_supported() {
        field_values.push(parse_quote!(flow_params_selection));
    }
    if scope.flow_count() == FlowCount::One {
        field_values.push(parse_quote!(params_specs_provided));
    }
}
