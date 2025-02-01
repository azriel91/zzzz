use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Unique identifier for an [`Item`], `Cow<'static, str>` newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// The following are all examples of valid `ItemId`s:
///
/// ```rust
/// # use peace_item_model::{item_id, ItemId};
/// #
/// let _snake = item_id!("snake_case");
/// let _camel = item_id!("camelCase");
/// let _pascal = item_id!("PascalCase");
/// ```
///
/// # Design Note
///
/// TODO: Experiment with upgrades.
///
/// For backward compatibility and migrating items from old IDs to new IDs, e.g.
/// when they were deployed with an old version of the automation software,
/// there needs to be a way to:
///
/// * Read state using the old ID.
/// * Either clean up that state, or migrate that state into an Item with the
///   new ID.
///
/// [`Item`]: https://docs.rs/peace_cfg/latest/peace_cfg/trait.Item.html
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemId(Cow<'static, str>);

peace_core::id_newtype!(ItemId, ItemIdInvalidFmt, item_id, code_inline);
