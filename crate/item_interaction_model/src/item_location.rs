use std::path::Path;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::ItemLocationType;

/// One layer of where a resource is located.
///
/// These will be merged into the same node based on their variant and name.
///
/// For example, if two different items provide the following
/// `ItemLocation`s:
///
/// Item 1:
///
/// 1. `ItemLocation::Group("cloud")`
/// 2. `ItemLocation::Host("app.domain.com")`
/// 3. `ItemLocation::Path("/path/to/a_file")`
///
/// Item 2:
///
/// 1. `ItemLocation::Host("app.domain.com")`
/// 2. `ItemLocation::Path("/path/to/another_file")`
///
/// Then the resultant node hierarchy will be:
///
/// ```yaml
/// cloud:
///   app.domain.com:
///     "/path/to/a_file": {}
///     "/path/to/another_file": {}
/// ```
///
/// # Implementors
///
/// Item implementors should endeavour to use the same name for each
/// `ItemLocation`, as that is how the Peace framework determines if two
/// `ItemLocation`s are the same.
///
/// # Design
///
/// When designing this, another design that was considered is using an enum
/// like the following:
///
/// ```rust,ignore
/// #[derive(Debug)]
/// enum ItemLocation {
///     Host(ItemLocationHost),
///     Url(Url),
/// }
///
/// struct ItemLocationHost {
///     host: Host<String>,
///     port: Option<u16>,
/// }
///
/// impl ItemLocation {
///     fn from_url(url: &Url) -> Self {
///         Self::Url(url.clone())
///     }
/// }
///
/// impl From<&Url> for ItemLocationHost {
///     type Error = ();
///
///     fn from(url: &Url) -> Result<Self, ()> {
///         url.host()
///             .map(|host| {
///                 let host = host.to_owned();
///                 let port = url.map(Url::port_or_known_default);
///                 Self { host, port }
///             })
///             .ok_or(())
///     }
/// }
/// ```
///
/// However, the purpose of `ItemLocation` is primarily for rendering, and
/// providing accurate variants for each kind of resource location causes
/// additional burden on:
///
/// * framework maintainers to maintain those variants
/// * item implementors to select the correct variant for accuracy
/// * item implementors to select a variant consistent with other item
///   implementors
///
/// A less accurate model with a limited number of [`ItemLocationType`]s
/// balances the modelling accuracy, rendering, and maintenance burden.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct ItemLocation {
    /// The type of the resource location.
    pub r#type: ItemLocationType,
    /// The name of the resource location.
    pub name: String,
}

impl ItemLocation {
    /// The string used for an unknown host.
    pub const HOST_UNKNOWN: &'static str = "unknown";
    /// The string used for localhost.
    pub const LOCALHOST: &'static str = "💻 localhost";

    /// Returns a new `ItemLocation`.
    ///
    /// See also:
    ///
    /// * [`ItemLocation::group`]
    /// * [`ItemLocation::host`]
    /// * [`ItemLocation::localhost`]
    /// * [`ItemLocation::path`]
    pub fn new(r#type: ItemLocationType, name: String) -> Self {
        Self { r#type, name }
    }

    /// Returns `ItemLocation::new(name, ItemLocationType::Group)`.
    pub fn group(name: String) -> Self {
        Self {
            r#type: ItemLocationType::Group,
            name,
        }
    }

    /// Returns `ItemLocation::new(name, ItemLocationType::Host)`.
    pub fn host(name: String) -> Self {
        Self {
            r#type: ItemLocationType::Host,
            name,
        }
    }

    /// Returns `ItemLocation::new("unknown".to_string(),
    /// ItemLocationType::Host)`.
    pub fn host_unknown() -> Self {
        Self {
            r#type: ItemLocationType::Host,
            name: Self::HOST_UNKNOWN.to_string(),
        }
    }

    /// Returns `ItemLocation::new(name, ItemLocationType::Host)`.
    ///
    /// This is "lossy" in the sense that if the URL doesn't have a [`Host`],
    /// this will return localhost, as URLs without a host may be unix sockets,
    /// or data URLs.
    ///
    /// [`Host`]: url::Host
    pub fn host_from_url(url: &Url) -> Self {
        url.host_str()
            .map(|host_str| Self {
                r#type: ItemLocationType::Host,
                name: format!("🌐 {host_str}"),
            })
            .unwrap_or_else(Self::localhost)
    }

    /// Returns `ItemLocation::host("localhost".to_string())`.
    pub fn localhost() -> Self {
        Self {
            r#type: ItemLocationType::Host,
            name: Self::LOCALHOST.to_string(),
        }
    }

    /// Returns `ItemLocation::new(name, ItemLocationType::Path)`.
    ///
    /// See also [`ItemLocation::path_lossy`].
    ///
    /// [`ItemLocation::path_lossy`]: Self::path_lossy
    pub fn path(name: String) -> Self {
        Self {
            r#type: ItemLocationType::Path,
            name,
        }
    }

    /// Returns `ItemLocation::new(name, ItemLocationType::Path)`, using the
    /// lossy conversion from the given path.
    pub fn path_lossy(name: &Path) -> Self {
        Self {
            // For some reason, calling `to_string_lossy()` on the path itself doesn't return the
            // replacement character, and breaks the
            // `item_interaction_model::item_location::path_lossy` test.
            //
            // The rust source code on 1.80.0 stable uses `String::from_utf8_lossy` internally:
            // <https://doc.rust-lang.org/src/std/sys/os_str/bytes.rs.html#271>
            //
            // ```rust
            // name.to_string_lossy().to_string()
            // ```
            name: String::from_utf8_lossy(name.as_os_str().as_encoded_bytes()).to_string(),
            r#type: ItemLocationType::Path,
        }
    }

    /// Returns the name of the resource location.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the type of the resource location.
    pub fn r#type(&self) -> ItemLocationType {
        self.r#type
    }
}
