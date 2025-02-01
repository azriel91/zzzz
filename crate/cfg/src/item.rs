use std::fmt::{Debug, Display};

use async_trait::async_trait;
use dyn_clone::DynClone;
use peace_data::Data;
use peace_item_model::ItemId;
use peace_params::{Params, ParamsSpec};
use peace_resource_rt::{resources::ts::Empty, Resources};
use serde::{de::DeserializeOwned, Serialize};

use crate::{ApplyCheck, FnCtx};

/// Defines all of the data and logic to manage an item.
///
/// The item may be simple or complex, ranging from:
///
/// * File download.
/// * Application installation.
/// * Server launching / initialization.
/// * Multiple cloud resource management.
///
/// The lifecycle functions include:
///
/// 1. Status discovery.
/// 2. Execution.
/// 3. Backup.
/// 4. Restoration.
/// 5. Clean up / deletion.
///
/// Since the latter four functions are write-operations, their specification
/// includes a dry run function.
///
/// # Logical IDs vs Physical IDs
///
/// A logical ID is defined by code, and does not change. A physical ID is one
/// generated during execution, which may be random or computed.
///
/// ## Examples
///
/// The following are examples of logical IDs and corresponding physical
/// IDs:
///
/// * If the function creates a file, the ID *may* be the full file path, or it
///   may be the file name, assuming the file path may be deduced by the clean
///   up logic from [`Data`].
///
/// * If the function instantiates a virtual machine on a cloud platform, this
///   may be the ID of the instance so that it may be terminated.
///
/// | Logical ID               | Physical ID                            |
/// | ------------------------ | -------------------------------------- |
/// | `app.file_path`          | `/mnt/data/app.zip`                    |
/// | `app_server_instance_id` | `ef34a9a4-0c02-45a6-96ec-a4db06d4980c` |
/// | `app_server.address`     | `10.0.0.1`                             |
///
/// [`Data`]: crate::CleanOpSpec::Data
#[async_trait(?Send)]
pub trait Item: DynClone {
    /// Consumer provided error type.
    type Error: std::error::Error + Send + Sync;

    /// Summary of the managed item's state.
    ///
    /// **For an extensive explanation of state, and how to define it, please
    /// see the [state concept] as well as the [`State`] type.**
    ///
    /// This type is used to represent the current state of the item (if it
    /// exists), the goal state of the item (what is intended to exist), and
    /// is used in the *diff* calculation -- what is the difference between the
    /// current and goal states.
    ///
    /// # Examples
    ///
    /// * A file's state may be its path, and a hash of its contents.
    /// * A server's state may be its operating system, CPU and memory capacity,
    ///   IP address, and ID.
    ///
    /// [state concept]: https://peace.mk/book/technical_concepts/state.html
    /// [`State`]: crate::state::State
    #[cfg(not(feature = "output_progress"))]
    type State: Clone
        + Debug
        + Display
        + PartialEq
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + 'static;

    /// Summary of the managed item's state.
    ///
    /// **For an extensive explanation of state, and how to define it, please
    /// see the [state concept] as well as the [`State`] type.**
    ///
    /// This type is used to represent the current state of the item (if it
    /// exists), the goal state of the item (what is intended to exist), and
    /// is used in the *diff* calculation -- what is the difference between the
    /// current and goal states.
    ///
    /// # Examples
    ///
    /// * A file's state may be its path, and a hash of its contents.
    /// * A server's state may be its operating system, CPU and memory capacity,
    ///   IP address, and ID.
    ///
    /// [state concept]: https://peace.mk/book/technical_concepts/state.html
    /// [`State`]: crate::state::State
    #[cfg(feature = "output_progress")]
    type State: Clone
        + Debug
        + Display
        + PartialEq
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + 'static
        + crate::RefInto<peace_item_interaction_model::ItemLocationState>;

    /// Diff between the current and target [`State`]s.
    ///
    /// # Design Note
    ///
    /// Initially I thought the field-wise diff between two [`State`]s is
    /// suitable, but:
    ///
    /// * Externally controlled state may not be known ahead of time.
    /// * It isn't easy or necessarily goal to compare every single field.
    /// * `state.apply(diff) = state_goal` may not be meaningful for a field
    ///   level diff, and the `apply` may be a complex process.
    ///
    /// [`State`]: Self::State
    type StateDiff: Clone + Debug + Display + Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Parameters to use this item.
    ///
    /// Item consumers must provide for this item to work.
    ///
    /// # Examples
    ///
    /// * For a file download item:
    ///
    ///     - URL of the file.
    ///     - Credentials.
    ///
    /// * For a server launch item:
    ///
    ///     - Image ID.
    ///     - Server size.
    ///
    /// # Implementors
    ///
    /// Peace will automatically save and load these into `Resources` when a
    /// command context is built.
    type Params<'exec>: Params<Spec = ParamsSpec<Self::Params<'exec>>>
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + 'static;

    /// Data that the item accesses at runtime.
    ///
    /// These may be objects instantiated in `setup` for use during execution,
    /// or information calculated from previous items.
    type Data<'exec>: Data<'exec>;

    /// Returns the ID of this full spec.
    ///
    /// # Implementors
    ///
    /// The ID should be a unique value that does not change over the lifetime
    /// of the managed item.
    ///
    /// [`ItemId`]s must begin with a letter or underscore, and contain only
    /// letters, numbers, and underscores.  The [`item_id!`] macro provides
    /// a compile time check to ensure that these conditions are upheld.
    ///
    /// ```rust
    /// # use peace_item_model::{item_id, ItemId};
    /// const fn id() -> ItemId {
    ///     item_id!("my_item")
    /// }
    /// # fn main() { let _id = id(); }
    /// ```
    ///
    /// # Design Note
    ///
    /// This is an instance method as logic for an `Item` may be used for
    /// multiple tasks. For example, an `Item` implemented to download a
    /// file may be instantiated with different files to download, and each
    /// instance of the `Item` should have its own ID.
    ///
    /// [`item_id!`]: peace_static_check_macros::item_id
    fn id(&self) -> &ItemId;

    /// Inserts an instance of each data type in [`Resources`].
    ///
    /// # Implementors
    ///
    /// [`Resources`] is the map of any type, and an instance of each data type
    /// must be inserted into the map so that item functions can borrow the
    /// instance of that type.
    ///
    /// ## External Parameters
    ///
    /// If the item works with an external source for parameters, such as:
    ///
    /// * a version controlled package file that specifies dependency versions
    /// * (discouraged) a web service with project configuration
    ///
    /// then this is the function to include the logic to read those files.
    ///
    /// ## Fallibility
    ///
    /// The function signature allows for fallibility, to allow issues to be
    /// reported early, such as:
    ///
    /// * Credentials to SDK clients not present on the user's system.
    /// * Incompatible / invalid values specified in project configuration
    ///   files, or expected project configuration files don't exist.
    ///
    /// [`check`]: crate::ApplyFns::check
    /// [`apply`]: crate::ApplyFns::apply
    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), Self::Error>;

    /// Returns an example fully deployed state of the managed item.
    ///
    /// # Implementors
    ///
    /// This is *expected* to always return a value, as it is used to:
    ///
    /// * Display a diagram that shows the user what the item looks like when it
    ///   is fully deployed, without actually interacting with any external
    ///   state.
    ///
    /// As much as possible, use the values in the provided params and data.
    ///
    /// This function should **NOT** interact with any external services, or
    /// read from files that are part of the automation process, e.g.
    /// querying data from a web endpoint, or reading files that may be
    /// downloaded by a predecessor.
    ///
    /// ## Infallibility
    ///
    /// The signature is deliberately infallible to signal to implementors that
    /// calling an external service / read from a file is incorrect
    /// implementation for this method -- values in params / data may be example
    /// values from other items that may not resolve.
    ///
    /// ## Non-async
    ///
    /// Similar to infallibility, this signals to implementors that this
    /// function should be a cheap example state computation that is relatively
    /// realistic rather than determining an accurate value.
    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, data: Self::Data<'_>) -> Self::State;

    /// Returns the current state of the managed item, if possible.
    ///
    /// This should return `Ok(None)` if the state is not able to be queried,
    /// such as when failing to connect to a remote host, instead of returning
    /// an error.
    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, Self::Error>;

    /// Returns the current state of the managed item.
    ///
    /// This is *expected* to successfully discover the current state, so errors
    /// will be presented to the user.
    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, Self::Error>;

    /// Returns the goal state of the managed item, if possible.
    ///
    /// This should return `Ok(None)` if the state is not able to be queried,
    /// such as when failing to read a potentially non-existent file to
    /// determine its content hash, instead of returning an error.
    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, Self::Error>;

    /// Returns the goal state of the managed item.
    ///
    /// This is *expected* to successfully discover the goal state, so errors
    /// will be presented to the user.
    ///
    /// # Examples
    ///
    /// * For a file download item, the goal state could be the destination path
    ///   and a content hash.
    ///
    /// * For a web application service item, the goal state could be the web
    ///   service is running on the latest version.
    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, Self::Error>;

    /// Returns the difference between two states.
    ///
    /// # Implementors
    ///
    /// When this type is serialized, it should provide "just enough" /
    /// meaningful information to the user on what has changed. So instead of
    /// including the complete goal [`State`], it should only include the
    /// parts that changed.
    ///
    /// This function call is intended to be cheap and fast.
    ///
    /// # Examples
    ///
    /// * For a file download item, the difference could be the content hash
    ///   changes from `abcd` to `efgh`.
    ///
    /// * For a web application service item, the goal state could be the
    ///   application version changing from 1 to 2.
    async fn state_diff(
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
        state_a: &Self::State,
        state_b: &Self::State,
    ) -> Result<Self::StateDiff, Self::Error>;

    /// Returns the representation of a clean `State`.
    ///
    /// # Implementors
    ///
    /// This should return essentially the `None` concept of the item
    /// state. The diff between this and the current state will be shown to the
    /// user when they want to see what would be cleaned up by the clean
    /// command.
    async fn state_clean(
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Self::State, Self::Error>;

    /// Returns whether `apply` needs to be executed.
    ///
    /// If the current state is already in sync with the target state, then
    /// `apply` does not have to be executed.
    ///
    /// # Examples
    ///
    /// * For a file download item, if the destination file differs from the
    ///   file on the server, then the file needs to be downloaded.
    ///
    /// * For a web application service item, if the web service is running, but
    ///   reports a previous version, then the service may need to be restarted.
    ///
    /// # Implementors
    ///
    /// This function call is intended to be cheap and fast.
    ///
    /// # Parameters
    ///
    /// * `fn_ctx`: Context to send progress updates.
    /// * `params`: Parameters to the item.
    /// * `data`: Runtime data that the function reads from or writes to.
    /// * `state_current`: Current [`State`] of the managed item, returned from
    ///   [`state_current`].
    /// * `state_target`: Target [`State`] of the managed item, either
    ///   [`state_clean`] or [`state_goal`].
    /// * `state_diff`: Goal [`State`] of the managed item, returned from
    ///   [`state_diff`].
    ///
    /// [`state_clean`]: crate::Item::state_clean
    /// [`state_current`]: crate::Item::state_current
    /// [`state_goal`]: crate::Item::state_goal
    /// [`State`]: Self::State
    /// [`state_diff`]: crate::Item::state_diff
    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error>;

    /// Dry-run transform of the current state to the target state.
    ///
    /// This will only be called if [`check`] returns [`ExecRequired`].
    ///
    /// This should mirror the logic in [`apply`], with the following
    /// differences:
    ///
    /// 1. When state will actually be altered, this would skip the logic.
    ///
    /// 2. Where there would be IDs received from an external system, a
    ///   placeholder ID should still be inserted into the runtime data. This
    ///   should allow subsequent `Item`s that rely on this one to use those
    ///   placeholders in their logic.
    ///
    /// # Implementors
    ///
    /// This function call is intended to be read-only and relatively cheap.
    /// Values in `params` and `data` cannot be guaranteed to truly exist.
    /// [#196] tracks the work to resolve what this function's contract should
    /// be.
    ///
    /// # Parameters
    ///
    /// * `fn_ctx`: Context to send progress updates.
    /// * `params`: Parameters to the item.
    /// * `data`: Runtime data that the function reads from or writes to.
    /// * `state_current`: Current [`State`] of the managed item, returned from
    ///   [`state_current`].
    /// * `state_target`: Target [`State`] of the managed item, either
    ///   [`state_clean`] or [`state_goal`].
    /// * `state_diff`: Goal [`State`] of the managed item, returned from
    ///   [`state_diff`].
    ///
    /// [`check`]: Self::check
    /// [`ExecRequired`]: crate::ApplyCheck::ExecRequired
    /// [`state_clean`]: crate::Item::state_clean
    /// [`state_current`]: crate::Item::state_current
    /// [`state_goal`]: crate::Item::state_goal
    /// [`State`]: Self::State
    /// [`state_diff`]: crate::Item::state_diff
    /// [#196]: https://github.com/azriel91/peace/issues/196
    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error>;

    /// Transforms the current state to the target state.
    ///
    /// This will only be called if [`check`] returns [`ExecRequired`].
    ///
    /// # Parameters
    ///
    /// * `fn_ctx`: Context to send progress updates.
    /// * `params`: Parameters to the item.
    /// * `data`: Runtime data that the function reads from or writes to.
    /// * `state_current`: Current [`State`] of the managed item, returned from
    ///   [`state_current`].
    /// * `state_target`: Target [`State`] of the managed item, either
    ///   [`state_clean`] or [`state_goal`].
    /// * `state_diff`: Goal [`State`] of the managed item, returned from
    ///   [`state_diff`].
    ///
    /// [`check`]: Self::check
    /// [`ExecRequired`]: crate::ApplyCheck::ExecRequired
    /// [`state_clean`]: crate::Item::state_clean
    /// [`state_current`]: crate::Item::state_current
    /// [`state_goal`]: crate::Item::state_goal
    /// [`State`]: Self::State
    /// [`state_diff`]: crate::Item::state_diff
    async fn apply(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error>;

    /// Returns the physical resources that this item interacts with.
    ///
    /// # Examples
    ///
    /// ## File Download Item
    ///
    /// This may be from:
    ///
    /// * host server
    /// * URL
    ///
    /// to:
    ///
    /// * localhost
    /// * file system path
    ///
    ///
    /// ### Server Launch Item
    ///
    /// This may be from:
    ///
    /// * localhost
    ///
    /// to:
    ///
    /// * cloud provider
    /// * region
    /// * subnet
    /// * host
    ///
    ///
    /// # Implementors
    ///
    /// The returned list should be in order of least specific to most specific
    /// location.
    #[cfg(feature = "item_interactions")]
    fn interactions(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Vec<peace_item_interaction_model::ItemInteraction>;
}
