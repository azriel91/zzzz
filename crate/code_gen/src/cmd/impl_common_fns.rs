use quote::quote;

use crate::cmd::{CmdCtxBuilderTypeBuilder, FlowCount, ImplHeaderBuilder, ScopeStruct};

/// Generates functions for the command context builder that are not constrained
/// by type parameters.
pub fn impl_common_fns(scope_struct: &ScopeStruct) -> proc_macro2::TokenStream {
    let scope = scope_struct.scope();
    let scope_builder_name = &scope_struct.item_struct().ident;

    let return_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone()).build();

    let mut common_fns = quote! {
        /// Sets the interrupt receiver and strategy so `CmdExecution`s can be interrupted.
        pub fn with_interruptibility(
            mut self,
            interruptibility: interruptible::Interruptibility<'static>,
        ) -> #return_type {
            let crate::ctx::CmdCtxBuilder {
                output,
                interruptibility: _,
                workspace,
                resources,
                scope_builder,
            } = self;

            crate::ctx::CmdCtxBuilder {
                output,
                interruptibility,
                workspace,
                resources,
                scope_builder,
            }
        }

        /// Sets the interrupt receiver and strategy so `CmdExecution`s can be interrupted.
        pub fn with_resource<R>(
            mut self,
            resource: R,
        ) -> Self
        where R: peace_resource_rt::Resource
        {
            self.resources.insert(resource);
            self
        }
    };

    if scope.flow_count() == FlowCount::One {
        common_fns.extend(quote! {
            /// Sets an item's parameters.
            ///
            /// Note: this **must** be called for each item in the flow.
            pub fn with_item_params<I>(
                mut self,
                item_id: peace_item_model::ItemId,
                params_spec: <I::Params<'_> as peace_params::Params>::Spec,
            ) -> Self
            where
                I: peace_cfg::Item,
                AppError: From<I::Error>,
            {
                self.scope_builder.params_specs_provided.insert(item_id, params_spec);
                self
            }
        });
    };

    let builder_type = CmdCtxBuilderTypeBuilder::new(scope_builder_name.clone()).build();
    let impl_header = ImplHeaderBuilder::new(builder_type).build();

    quote! {
        #impl_header
        {
            #common_fns
        }
    }
}
