use quote::quote;

use crate::cmd::{
    param_key_impl, with_params::params_selection_types_some, ParamsScope, ScopeStruct,
};

/// Generates the `CmdCtxBuilder::*_params_merge` methods for each params type.
///
/// The generated method attempts to load params from storage, and if it is
/// present, merges it with the params passed to the command context builder.
pub fn impl_params_merge(scope_struct: &ScopeStruct) -> proc_macro2::TokenStream {
    ParamsScope::iter().fold(
        proc_macro2::TokenStream::new(),
        |mut impl_tokens, params_scope| {
            if (!scope_struct.scope().profile_params_supported()
                && params_scope == ParamsScope::Profile)
                || (!scope_struct.scope().flow_params_supported()
                    && params_scope == ParamsScope::Flow)
            {
                // Skip `*_params_merge` implementation if it is not supported.
                return impl_tokens;
            }

            impl_tokens.extend(impl_params_merge_for(scope_struct, params_scope));

            impl_tokens
        },
    )
}

fn impl_params_merge_for(
    scope_struct: &ScopeStruct,
    params_scope: ParamsScope,
) -> proc_macro2::TokenStream {
    let scope = scope_struct.scope();
    let scope_builder_name = &scope_struct.item_struct().ident;

    let (params_keys_assoc_type, params_selection_assoc_type, params_selection_struct) =
        params_selection_types_some(params_scope);

    let param_key_impl_known_predicates = param_key_impl::known_predicates(scope, params_scope);

    let params_merge_method_name = params_scope.params_merge_method_name();
    let params_deserialize_method_name = params_scope.params_deserialize_method_name();
    let p_keys_key_maybe_key = params_scope.p_keys_key_maybe_key();
    let params_k_type_param = params_scope.params_k_type_param();
    let params_type_reg_method_name = params_scope.params_type_reg_method_name();
    let params_file_name = params_scope.params_file_name();
    let params_file_type = params_scope.params_file_type();
    let params_selection_name = params_scope.params_selection_name();

    let doc_summary = {
        let params_scope_str = params_scope.to_str();
        format!(
            "Merges {params_scope_str} params provided by the caller with the {params_scope_str} params on disk."
        )
    };

    quote! {
        impl<'ctx, 'key, CmdCtxBuilderTypeParamsT, #params_k_type_param> crate::ctx::CmdCtxBuilder<
            'ctx,
            CmdCtxBuilderTypeParamsT,
            #scope_builder_name<CmdCtxBuilderTypeParamsT>
        >
        where
            CmdCtxBuilderTypeParamsT: crate::ctx::CmdCtxBuilderTypeParams<
                // WorkspaceParamsSelection = WorkspaceParamsSome<<ParamsKeys::WorkspaceParamsKMaybe as KeyMaybe>::Key>,
                #params_selection_assoc_type = crate::scopes::type_params::#params_selection_struct<#params_k_type_param>,
            >,
            <CmdCtxBuilderTypeParamsT as crate::ctx::CmdCtxBuilderTypeParams>::ParamsKeys:
            // ParamsKeys<WorkspaceParamsKMaybe = KeyKnown<WorkspaceParamsK>>
            peace_rt_model::params::ParamsKeys<#params_keys_assoc_type = peace_rt_model::params::KeyKnown<#params_k_type_param>>,

            // WorkspaceParamsK:
            //     Clone + std::fmt::Debug + Eq + std::hash::Hash + serde::de::DeserializeOwned + serde::Serialize + Send + Sync + Unpin + 'static,
            #param_key_impl_known_predicates
        {
            #[doc = #doc_summary]
            // async fn workspace_params_merge
            async fn #params_merge_method_name(
                &mut self,
                // workspace_params_file: &peace_resources::internal::WorkspaceParamsFile,
                #params_file_name: &peace_resources::internal::#params_file_type,
            ) -> Result<(), peace_rt_model::Error> {
                let storage = self.workspace.storage();
                let params_deserialized = peace_rt_model::WorkspaceInitializer::#params_deserialize_method_name::<
                    // <ParamsKeys::WorkspaceParamsKMaybe as KeyMaybe>::Key,
                    #p_keys_key_maybe_key
                >(
                    storage,
                    // self.scope_builder.params_type_regs_builder.workspace_params_type_reg(),
                    self.scope_builder.params_type_regs_builder.#params_type_reg_method_name(),
                    #params_file_name,
                )
                .await?;
                match params_deserialized {
                    Some(params_deserialized) => {
                        // Merge `params` on top of `params_deserialized`.
                        // or, copy `params_deserialized` to `params` where
                        // there isn't a value.

                        let params = &mut self.scope_builder.#params_selection_name.0;
                        if params.is_empty() {
                            *params = params_deserialized;
                        } else {
                            params_deserialized
                                .into_inner()
                                .into_inner()
                                .into_iter()
                                .for_each(|(key, param)| {
                                    if !params.contains_key(&key) {
                                        params.insert_raw(key, param);
                                    }
                                });
                        }

                    }
                    None => {}
                }

                Ok(())
            }
        }
    }
}
