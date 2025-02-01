use std::{marker::PhantomData, path::Path};

use peace_flow_model::FlowId;
use peace_flow_rt::ItemGraph;
use peace_item_model::ItemId;
use peace_resource_rt::{
    paths::{StatesCurrentFile, StatesGoalFile},
    states::{
        ts::{CurrentStored, GoalStored},
        States, StatesCurrentStored, StatesGoalStored,
    },
    type_reg::untagged::{BoxDtDisplay, TypeMapOpt, TypeReg},
};
use peace_rt_model::Storage;
use peace_rt_model_core::Error;

/// Reads and writes [`StatesCurrentStored`] and [`StatesGoalStored`] to and
/// from storage.
pub struct StatesSerializer<E>(PhantomData<E>);

impl<E> StatesSerializer<E>
where
    E: std::error::Error + From<Error> + Send + 'static,
{
    /// Returns the [`StatesCurrentStored`] of all [`Item`]s if it exists on
    /// disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `states`: States to serialize.
    /// * `states_file_path`: Path to save the serialized states to.
    ///
    /// [`Item`]: peace_cfg::Item
    pub async fn serialize<TS>(
        storage: &Storage,
        item_graph: &ItemGraph<E>,
        states: &States<TS>,
        states_file_path: &Path,
    ) -> Result<(), E>
    where
        TS: Send + Sync,
    {
        let states_serde = item_graph.states_serde::<serde_yaml::Value, _>(states);
        storage
            .serialized_write(
                #[cfg(not(target_arch = "wasm32"))]
                "StatesSerializer::serialize".to_string(),
                states_file_path,
                &states_serde,
                Error::StatesSerialize,
            )
            .await?;

        Ok(())
    }

    /// Returns the [`StatesCurrentStored`] of all [`Item`]s if it exists on
    /// disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `states_type_reg`: Type registry with functions to deserialize each
    ///   item state.
    /// * `states_current_file`: `StatesCurrentFile` to deserialize.
    ///
    /// [`Item`]: peace_cfg::Item
    pub async fn deserialize_stored(
        flow_id: &FlowId,
        storage: &Storage,
        states_type_reg: &TypeReg<ItemId, BoxDtDisplay>,
        states_current_file: &StatesCurrentFile,
    ) -> Result<StatesCurrentStored, E> {
        let states = Self::deserialize_internal::<CurrentStored>(
            #[cfg(not(target_arch = "wasm32"))]
            "StatesSerializer::deserialize_stored".to_string(),
            flow_id,
            storage,
            states_type_reg,
            states_current_file,
        )
        .await?;

        states.ok_or_else(|| E::from(Error::StatesCurrentDiscoverRequired))
    }

    /// Returns the [`StatesGoalStored`] of all [`Item`]s if it exists on disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `states_type_reg`: Type registry with functions to deserialize each
    ///   item state.
    /// * `states_goal_file`: `StatesGoalFile` to deserialize.
    ///
    /// [`Item`]: peace_cfg::Item
    pub async fn deserialize_goal(
        flow_id: &FlowId,
        storage: &Storage,
        states_type_reg: &TypeReg<ItemId, BoxDtDisplay>,
        states_goal_file: &StatesGoalFile,
    ) -> Result<StatesGoalStored, E> {
        let states = Self::deserialize_internal::<GoalStored>(
            #[cfg(not(target_arch = "wasm32"))]
            "StatesSerializer::deserialize_goal".to_string(),
            flow_id,
            storage,
            states_type_reg,
            states_goal_file,
        )
        .await?;

        states.ok_or_else(|| E::from(Error::StatesGoalDiscoverRequired))
    }

    /// Returns the [`StatesCurrentStored`] of all [`Item`]s if it exists on
    /// disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `states_type_reg`: Type registry with functions to deserialize each
    ///   item state.
    /// * `states_current_file`: `StatesCurrentFile` to deserialize.
    ///
    /// [`Item`]: peace_cfg::Item
    pub async fn deserialize_stored_opt(
        flow_id: &FlowId,
        storage: &Storage,
        states_type_reg: &TypeReg<ItemId, BoxDtDisplay>,
        states_current_file: &StatesCurrentFile,
    ) -> Result<Option<StatesCurrentStored>, E> {
        Self::deserialize_internal(
            #[cfg(not(target_arch = "wasm32"))]
            "StatesSerializer::deserialize_stored_opt".to_string(),
            flow_id,
            storage,
            states_type_reg,
            states_current_file,
        )
        .await
    }

    /// Returns the [`States`] of all [`Item`]s if it exists on disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `states_type_reg`: Type registry with functions to deserialize each
    ///   item state.
    /// * `states_current_file`: `StatesCurrentFile` to deserialize.
    ///
    /// # Type Parameters
    ///
    /// * `TS`: The states type state to use, such as [`ts::Current`] or
    ///   [`ts::CurrentStored`].
    ///
    /// [`Item`]: peace_cfg::Item
    /// [`ts::Current`]: peace_resource_rt::states::ts::Current
    /// [`ts::CurrentStored`]: peace_resource_rt::states::ts::CurrentStored
    #[cfg(not(target_arch = "wasm32"))]
    async fn deserialize_internal<TS>(
        thread_name: String,
        flow_id: &FlowId,
        storage: &Storage,
        states_type_reg: &TypeReg<ItemId, BoxDtDisplay>,
        states_file_path: &Path,
    ) -> Result<Option<States<TS>>, E>
    where
        TS: Send + Sync,
    {
        let states_opt = storage
            .serialized_typemap_read_opt(thread_name, states_type_reg, states_file_path, |error| {
                #[cfg(not(feature = "error_reporting"))]
                {
                    Error::StatesDeserialize {
                        flow_id: flow_id.clone(),
                        error,
                    }
                }
                #[cfg(feature = "error_reporting")]
                {
                    use miette::NamedSource;
                    use yaml_error_context_hack::ErrorAndContext;

                    let file_contents = std::fs::read_to_string(states_file_path).unwrap();

                    let ErrorAndContext {
                        error_span,
                        error_message,
                        context_span,
                    } = ErrorAndContext::new(&file_contents, &error);
                    let states_file_source =
                        NamedSource::new(states_file_path.to_string_lossy(), file_contents);

                    Error::StatesDeserialize {
                        flow_id: flow_id.clone(),
                        states_file_source,
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
                    .map(States::from)
            })?;

        Ok(states_opt)
    }

    /// Returns the [`States`] of all [`Item`]s if it exists on disk.
    ///
    /// # Parameters:
    ///
    /// * `storage`: `Storage` to read from.
    /// * `states_type_reg`: Type registry with functions to deserialize each
    ///   item state.
    /// * `states_current_file`: `StatesCurrentFile` to deserialize.
    ///
    /// # Type Parameters
    ///
    /// * `TS`: The states type state to use, such as [`ts::Current`] or
    ///   [`ts::CurrentStored`].
    ///
    /// [`Item`]: peace_cfg::Item
    /// [`ts::Current`]: peace_resource_rt::states::ts::Current
    /// [`ts::CurrentStored`]: peace_resource_rt::states::ts::CurrentStored
    #[cfg(target_arch = "wasm32")]
    async fn deserialize_internal<TS>(
        flow_id: &FlowId,
        storage: &Storage,
        states_type_reg: &TypeReg<ItemId, BoxDtDisplay>,
        states_file_path: &Path,
    ) -> Result<Option<States<TS>>, E>
    where
        TS: Send + Sync,
    {
        let states_opt = storage
            .serialized_typemap_read_opt(states_type_reg, states_file_path, |error| {
                #[cfg(not(feature = "error_reporting"))]
                {
                    Error::StatesDeserialize {
                        flow_id: flow_id.clone(),
                        error,
                    }
                }
                #[cfg(feature = "error_reporting")]
                {
                    use miette::NamedSource;
                    use yaml_error_context_hack::ErrorAndContext;

                    let file_contents = std::fs::read_to_string(states_file_path).unwrap();

                    let ErrorAndContext {
                        error_span,
                        error_message,
                        context_span,
                    } = ErrorAndContext::new(&file_contents, &error);
                    let states_file_source =
                        NamedSource::new(states_file_path.to_string_lossy(), file_contents);

                    Error::StatesDeserialize {
                        flow_id: flow_id.clone(),
                        states_file_source,
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
                    .map(States::from)
            })?;

        Ok(states_opt)
    }
}
