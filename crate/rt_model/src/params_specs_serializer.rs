use std::marker::PhantomData;

use peace_flow_model::FlowId;
use peace_params::ParamsSpecs;
use peace_profile_model::Profile;
use peace_resource_rt::{paths::ParamsSpecsFile, type_reg::untagged::TypeMapOpt};

use crate::{Error, ParamsSpecsTypeReg, Storage};

/// Reads and writes [`ParamsSpecs`] to and from storage.
pub struct ParamsSpecsSerializer<E>(PhantomData<E>);

impl<E> ParamsSpecsSerializer<E>
where
    E: std::error::Error + From<Error> + Send,
{
    /// Serializes the [`ParamsSpecs`] of all [`Item`]s to disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to write to.
    /// * `params_specs`: `ParamsSpecs` to serialize.
    /// * `params_specs_file`: Path to save the serialized params_specs to.
    ///
    /// [`Item`]: peace_cfg::Item
    pub async fn serialize(
        storage: &Storage,
        params_specs: &ParamsSpecs,
        params_specs_file: &ParamsSpecsFile,
    ) -> Result<(), E> {
        storage
            .serialized_write(
                #[cfg(not(target_arch = "wasm32"))]
                "ParamsSpecsSerializer::serialize".to_string(),
                params_specs_file,
                params_specs,
                Error::ParamsSpecsSerialize,
            )
            .await?;

        Ok(())
    }

    /// Returns the [`ParamsSpecs`] of all [`Item`]s if it exists on disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `params_specs_type_reg`: Type registry with functions to deserialize
    ///   each params spec.
    /// * `params_specs_file`: `ParamsSpecsFile` to deserialize.
    ///
    /// [`Item`]: peace_cfg::Item
    pub async fn deserialize_opt(
        profile: &Profile,
        flow_id: &FlowId,
        storage: &Storage,
        params_specs_type_reg: &ParamsSpecsTypeReg,
        params_specs_file: &ParamsSpecsFile,
    ) -> Result<Option<ParamsSpecs>, E> {
        Self::deserialize_internal(
            #[cfg(not(target_arch = "wasm32"))]
            "ParamsSpecsSerializer::deserialize_opt".to_string(),
            profile,
            flow_id,
            storage,
            params_specs_type_reg,
            params_specs_file,
        )
        .await
    }

    /// Returns the [`ParamsSpecs`] of all [`Item`]s if it exists on disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `params_specs_type_reg`: Type registry with functions to deserialize
    ///   each params spec.
    /// * `params_specs_file`: `ParamsSpecsFile` to deserialize.
    ///
    /// [`Item`]: peace_cfg::Item
    #[cfg(not(target_arch = "wasm32"))]
    async fn deserialize_internal(
        thread_name: String,
        profile: &Profile,
        flow_id: &FlowId,
        storage: &Storage,
        params_specs_type_reg: &ParamsSpecsTypeReg,
        params_specs_file: &ParamsSpecsFile,
    ) -> Result<Option<ParamsSpecs>, E> {
        let params_specs_opt = storage
            .serialized_typemap_read_opt(
                thread_name,
                params_specs_type_reg,
                params_specs_file,
                |error| {
                    #[cfg(not(feature = "error_reporting"))]
                    {
                        Error::ParamsSpecsDeserialize {
                            profile: profile.clone(),
                            flow_id: flow_id.clone(),
                            error,
                        }
                    }
                    #[cfg(feature = "error_reporting")]
                    {
                        use miette::NamedSource;
                        use yaml_error_context_hack::ErrorAndContext;

                        let file_contents = std::fs::read_to_string(params_specs_file).unwrap();

                        let ErrorAndContext {
                            error_span,
                            error_message,
                            context_span,
                        } = ErrorAndContext::new(&file_contents, &error);
                        let params_specs_file_source =
                            NamedSource::new(params_specs_file.to_string_lossy(), file_contents);

                        Error::ParamsSpecsDeserialize {
                            profile: profile.clone(),
                            flow_id: flow_id.clone(),
                            params_specs_file_source,
                            error_span,
                            error_message,
                            context_span,
                            error,
                        }
                    }
                },
            )
            .await
            .map(|type_map_opt| {
                type_map_opt
                    .map(TypeMapOpt::into_type_map)
                    .map(ParamsSpecs::from)
            })?;

        Ok(params_specs_opt)
    }

    /// Returns the [`ParamsSpecs`] of all [`Item`]s if it exists on disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `params_specs_type_reg`: Type registry with functions to deserialize
    ///   each params spec.
    /// * `params_specs_file`: `ParamsSpecsFile` to deserialize.
    ///
    /// [`Item`]: peace_cfg::Item
    #[cfg(target_arch = "wasm32")]
    async fn deserialize_internal(
        profile: &Profile,
        flow_id: &FlowId,
        storage: &Storage,
        params_specs_type_reg: &ParamsSpecsTypeReg,
        params_specs_file: &ParamsSpecsFile,
    ) -> Result<Option<ParamsSpecs>, E> {
        let params_specs_opt = storage
            .serialized_typemap_read_opt(params_specs_type_reg, params_specs_file, |error| {
                #[cfg(not(feature = "error_reporting"))]
                {
                    Error::ParamsSpecsDeserialize {
                        profile: profile.clone(),
                        flow_id: flow_id.clone(),
                        error,
                    }
                }
                #[cfg(feature = "error_reporting")]
                {
                    use miette::NamedSource;
                    use yaml_error_context_hack::ErrorAndContext;

                    let file_contents = std::fs::read_to_string(params_specs_file).unwrap();

                    let ErrorAndContext {
                        error_span,
                        error_message,
                        context_span,
                    } = ErrorAndContext::new(&file_contents, &error);
                    let params_specs_file_source =
                        NamedSource::new(params_specs_file.to_string_lossy(), file_contents);

                    Error::ParamsSpecsDeserialize {
                        profile: profile.clone(),
                        flow_id: flow_id.clone(),
                        params_specs_file_source,
                        error_span,
                        error_message,
                        context_span,
                        error,
                    }
                }
            })
            .await
            .map(|type_map_opt| {
                type_map_opt
                    .map(TypeMapOpt::into_type_map)
                    .map(ParamsSpecs::from)
            })?;

        Ok(params_specs_opt)
    }
}
