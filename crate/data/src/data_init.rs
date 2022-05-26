use std::fmt::Debug;

use fn_graph::Resources;

/// Defines the logic to instantiate runtime data.
///
/// # Note for API Consumers
///
/// This should be implemented on types used in `OpSpec::Data`.
pub trait DataInit {
    /// Instantiates each of `Self`'s fields in the provided [`Resources`].
    ///
    /// This should be a sensible default.
    ///
    /// # Parameters
    ///
    /// * `resources`: `Any` map to insert the instance of each field into.
    fn init(resources: &mut Resources);
}

impl<T> DataInit for Option<T>
where
    T: Debug + Send + Sync + 'static,
{
    fn init(resources: &mut Resources) {
        resources.insert::<Option<T>>(None);
    }
}
