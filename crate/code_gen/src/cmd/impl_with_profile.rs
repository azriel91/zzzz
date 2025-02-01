use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, token::Comma, FieldValue, Token};

use crate::cmd::{
    CmdCtxBuilderTypeBuilder, FlowCount, ImplHeaderBuilder, ProfileCount, Scope, ScopeStruct,
};

/// Generates the `with_profile` method for the command context builder.
pub fn impl_with_profile(scope_struct: &ScopeStruct) -> proc_macro2::TokenStream {
    let scope = scope_struct.scope();
    let scope_builder_name = &scope_struct.item_struct().ident;

    if scope_struct.scope().profile_count() != ProfileCount::One {
        // `with_profile` is not supported.
        return proc_macro2::TokenStream::new();
    };

    let scope_builder_fields_profile_not_selected =
        scope_builder_fields_profile_not_selected(scope);
    let scope_builder_fields_profile_selected = scope_builder_fields_profile_selected(scope);

    // ```rust,ignore
    // crate::ctx::CmdCtxBuilder<
    //     'ctx,
    //     crate::ctx::CmdCtxBuilderTypesCollector<
    //         Output,
    //         AppError,
    //         peace_rt_model::params::ParamsKeysImpl<
    //             WorkspaceParamsKMaybe,
    //             ProfileParamsKMaybe,
    //             FlowParamsKMaybe,
    //         >,
    //         WorkspaceParamsSelection,
    //         ProfileParamsSelection,
    //         FlowParamsSelection,
    //         crate::scopes::type_params::ProfileNotSelected,
    //         FlowSelection,
    //     >,
    // >
    // ```

    let builder_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone())
        .with_profile_selection(parse_quote!(crate::scopes::type_params::ProfileNotSelected))
        .build();
    let impl_header = ImplHeaderBuilder::new(builder_type)
        .with_profile_selection(None)
        .build();
    let return_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone())
        .with_profile_selection(parse_quote!(crate::scopes::type_params::ProfileSelected))
        .build();

    let mut tokens = quote! {
        #impl_header
        {
            pub fn with_profile(
                self,
                profile: peace_profile_model::Profile,
            ) ->
                // crate::ctx::CmdCtxBuilder<
                //     'ctx,
                //     crate::ctx::CmdCtxBuilderTypesCollector<
                //         Output,
                //         AppError,
                //         peace_rt_model::params::ParamsKeysImpl<
                //             WorkspaceParamsKMaybe,
                //             ProfileParamsKMaybe,
                //             FlowParamsKMaybe,
                //         >,
                //         WorkspaceParamsSelection,
                //         ProfileParamsSelection,
                //         FlowParamsSelection,
                //         crate::scopes::type_params::ProfileSelected,
                //         FlowSelection,
                //     >,
                // >
                #return_type
            {
                let Self {
                    output,
                    interruptibility,
                    workspace,
                    resources,
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
                    // profile_selection: ProfileSelected(profile),
                    // flow_selection,
                    // params_type_regs_builder,
                    // workspace_params_selection,
                    // profile_params_selection,
                    // flow_params_selection,
                    // params_specs_provided,
                    #scope_builder_fields_profile_selected
                };

                crate::ctx::CmdCtxBuilder {
                    output,
                    interruptibility,
                    workspace,
                    resources,
                    scope_builder,
                }
            }
        }
    };

    tokens.extend(impl_with_profile_from_workspace_param(scope_struct));

    tokens
}

/// Generates the `with_profile` method for the command context builder.
pub fn impl_with_profile_from_workspace_param(
    scope_struct: &ScopeStruct,
) -> proc_macro2::TokenStream {
    let scope = scope_struct.scope();
    let scope_builder_name = &scope_struct.item_struct().ident;

    let scope_builder_fields_profile_not_selected =
        scope_builder_fields_profile_not_selected(scope);
    let scope_builder_fields_profile_from_workspace =
        scope_builder_fields_profile_from_workspace(scope);

    // ```rust,ignore
    // crate::ctx::CmdCtxBuilder<
    //     'ctx,
    //     crate::ctx::CmdCtxBuilderTypesCollector<
    //         Output,
    //         AppError,
    //         peace_rt_model::params::ParamsKeysImpl<
    //             WorkspaceParamsKMaybe,
    //             ProfileParamsKMaybe,
    //             FlowParamsKMaybe,
    //         >,
    //         crate::scopes::type_params::WorkspaceParamsSome<WorkspaceParamsK>,
    //         ProfileParamsSelection,
    //         FlowParamsSelection,
    //         crate::scopes::type_params::ProfileNotSelected,
    //         FlowSelection,
    //     >,
    // >
    // ```

    let builder_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone())
        .with_workspace_params_k_maybe(parse_quote!(
            peace_rt_model::params::KeyKnown<WorkspaceParamsK>
        ))
        .with_workspace_params_selection(parse_quote!(
            crate::scopes::type_params::WorkspaceParamsSome<WorkspaceParamsK>
        ))
        .with_profile_selection(parse_quote!(crate::scopes::type_params::ProfileNotSelected))
        .build();
    let impl_header = ImplHeaderBuilder::new(builder_type)
        .with_workspace_params_k_maybe(None)
        .with_workspace_params_k(Some(parse_quote!(WorkspaceParamsK)))
        .with_workspace_params_selection(None)
        .with_profile_selection(None)
        .build();
    let return_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone())
        .with_workspace_params_k_maybe(parse_quote!(
            peace_rt_model::params::KeyKnown<WorkspaceParamsK>
        ))
        .with_workspace_params_selection(parse_quote!(
            crate::scopes::type_params::WorkspaceParamsSome<WorkspaceParamsK>
        ))
        .with_profile_selection(parse_quote!(
            crate::scopes::type_params::ProfileFromWorkspaceParam<'key, WorkspaceParamsK>
        ))
        .build();

    quote! {
        #impl_header
        {
            pub fn with_profile_from_workspace_param<'key>(
                self,
                workspace_param_k: own::OwnedOrRef<'key, WorkspaceParamsK>,
            ) ->
                // crate::ctx::CmdCtxBuilder<
                //     'ctx,
                //     crate::ctx::CmdCtxBuilderTypesCollector<
                //         Output,
                //         AppError,
                //         peace_rt_model::params::ParamsKeysImpl<
                //             WorkspaceParamsKMaybe = peace_rt_model::params::KeyKnown<WorkspaceParamsK>,
                //             ProfileParamsKMaybe,
                //             FlowParamsKMaybe,
                //         >,
                //         crate::scopes::type_params::WorkspaceParamsSome<WorkspaceParamsK>,
                //         ProfileParamsSelection,
                //         FlowParamsSelection,
                //         crate::scopes::type_params::ProfileFromWorkspaceParam<'key, WorkspaceParamsK>,
                //         FlowSelection,
                //     >,
                // >
                #return_type
            {
                let Self {
                    output,
                    interruptibility,
                    workspace,
                    resources,
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
                    // profile_selection: ProfileFromWorkspaceParam(workspace_param_k),
                    // flow_selection,
                    // params_type_regs_builder,
                    // workspace_params_selection,
                    // profile_params_selection,
                    // flow_params_selection,
                    // params_specs_provided,
                    #scope_builder_fields_profile_from_workspace
                };

                crate::ctx::CmdCtxBuilder {
                    output,
                    interruptibility,
                    workspace,
                    resources,
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

fn scope_builder_fields_profile_selected(scope: Scope) -> Punctuated<FieldValue, Comma> {
    let mut field_values = Punctuated::<FieldValue, Token![,]>::new();
    field_values.push(parse_quote!(
        profile_selection: crate::scopes::type_params::ProfileSelected(profile)
    ));
    scope_builder_fields_remainder_push(scope, &mut field_values);

    field_values
}

fn scope_builder_fields_profile_from_workspace(scope: Scope) -> Punctuated<FieldValue, Comma> {
    let mut field_values = Punctuated::<FieldValue, Token![,]>::new();
    field_values.push(parse_quote!(
        profile_selection: crate::scopes::type_params::ProfileFromWorkspaceParam(workspace_param_k)
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
