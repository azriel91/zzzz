#[cfg(feature = "error_reporting")]
use peace::miette;

/// Error while running a workspace test.
///
/// Error type for tests that need an error for their item graphs and
/// command contexts.
#[cfg_attr(feature = "error_reporting", derive(peace::miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum PeaceTestError {
    /// Flow ID is invalid.
    #[error("Flow ID is invalid.")]
    FlowIdInvalidFmt(
        #[source]
        #[from]
        peace::flow_model::FlowIdInvalidFmt<'static>,
    ),

    /// Failed to initialize tempdir.
    #[error("Failed to initialize tempdir.")]
    TempDir(
        #[source]
        #[from]
        std::io::Error,
    ),

    /// A VecCopy item error occurred.
    #[error("A VecCopy item error occurred.")]
    VecCopy(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::VecCopyError,
    ),
    /// A Mock item error occurred.
    #[error("A Mock item error occurred.")]
    Mock(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::mock_item::MockItemError,
    ),

    /// A Blank item error occurred.
    #[error("A Blank item error occurred.")]
    Blank(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace_items::blank::BlankError,
    ),

    /// A `peace` runtime error occurred.
    #[error("A `peace` runtime error occurred.")]
    PeaceRt(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace::rt_model::Error,
    ),
}
