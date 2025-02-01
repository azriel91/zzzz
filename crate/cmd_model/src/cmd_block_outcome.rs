use fn_graph::StreamOutcome;
use indexmap::IndexMap;
use peace_item_model::ItemId;

use crate::{StreamOutcomeAndErrors, ValueAndStreamOutcome};

/// Outcome of running `CmdBlock::exec`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CmdBlockOutcome<T, E> {
    /// Single value returned by the command block.
    ///
    /// Relevant to command blocks that deal with a single atomic operation,
    /// e.g. loading a file.
    Single(T),
    /// A value returned per item.
    ///
    /// # Design Note
    ///
    /// When implementing this, the following structures were considered:
    ///
    /// * Having separate fields for `T` and `StreamOutcome<()>`.
    /// * Having a single `StreamOutcome<T>` field.
    ///
    /// The first design makes it easier to access the value, and the second
    /// design ensures that you cannot access the value and accidentally forget
    /// about the stream outcome.
    ///
    /// Because this is an enum variant, consumers are not likely to miss the
    /// stream outcome even if the first design is chosen.
    ///
    /// Having a `StreamOutcome<()>` separate from the value means consumers can
    /// choose to ignore the `StreamOutcome` more easily.
    ///
    /// However, the `CmdBlock::exec` return type is also affected by this --
    /// having consumers return a `StreamOutcome<T>` allows them to use the
    /// `FnGraph` streaming methods, without having to split the value out of
    /// the `StreamOutcome`.
    ItemWise {
        /// The values returned per item.
        stream_outcome: StreamOutcome<T>,
        /// Errors from the command execution.
        errors: IndexMap<ItemId, E>,
    },
}

impl<T, E> CmdBlockOutcome<T, E> {
    /// Returns a new `CmdBlockOutcome::ItemWise` with the given value, and no
    /// errors.
    pub fn new_item_wise(stream_outcome: StreamOutcome<T>) -> Self {
        Self::ItemWise {
            stream_outcome,
            errors: IndexMap::new(),
        }
    }

    /// Returns whether the command ran successfully.
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Single(_) => true,
            Self::ItemWise {
                stream_outcome: _,
                errors,
            } => errors.is_empty(),
        }
    }

    /// Returns this outcome's value if there are no errors, otherwise returns
    /// self.
    pub fn try_into_value(self) -> Result<ValueAndStreamOutcome<T>, StreamOutcomeAndErrors<T, E>> {
        match self {
            Self::Single(value) => Ok(ValueAndStreamOutcome {
                value,
                stream_outcome: None,
            }),
            Self::ItemWise {
                stream_outcome,
                errors,
            } => {
                if errors.is_empty() {
                    let (stream_outcome, value) = stream_outcome.replace(());
                    Ok(ValueAndStreamOutcome {
                        value,
                        stream_outcome: Some(stream_outcome),
                    })
                } else {
                    Err(StreamOutcomeAndErrors {
                        stream_outcome,
                        errors,
                    })
                }
            }
        }
    }

    /// Returns whether the command encountered any errors during execution.
    pub fn is_err(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::ItemWise {
                stream_outcome: _,
                errors,
            } => !errors.is_empty(),
        }
    }

    pub fn map<F, U>(self, f: F) -> CmdBlockOutcome<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Single(t) => {
                let u = f(t);
                CmdBlockOutcome::Single(u)
            }
            Self::ItemWise {
                stream_outcome,
                errors,
            } => {
                let stream_outcome = stream_outcome.map(f);
                CmdBlockOutcome::ItemWise {
                    stream_outcome,
                    errors,
                }
            }
        }
    }
}
