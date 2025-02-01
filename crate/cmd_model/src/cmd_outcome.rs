use futures::Future;
use indexmap::IndexMap;
use peace_item_model::ItemId;

use crate::{CmdBlockDesc, ItemStreamOutcome};

/// Outcome of a [`CmdExecution`].
///
/// The variants indicate whether execution was successful, interrupted, or
/// errored when processing an item.
///
/// [`CmdExecution`]: https://docs.rs/peace_cmd_rt/latest/peace_cmd_rt/struct.CmdExecution.html
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CmdOutcome<T, E> {
    /// Execution completed successfully.
    Complete {
        /// The outcome value.
        value: T,
        /// Descriptors of the `CmdBlock`s that were processed.
        ///
        /// This includes all `CmdBlock`s that were included in the
        /// `CmdExecution`.
        cmd_blocks_processed: Vec<CmdBlockDesc>,
    },
    /// Execution ended due to an interruption during command block execution.
    BlockInterrupted {
        /// The stream outcome of the interrupted command block.
        item_stream_outcome: ItemStreamOutcome<T>,
        /// Descriptors of the `CmdBlock`s that were processed.
        ///
        /// This does not include the `CmdBlock` that was interrupted.
        cmd_blocks_processed: Vec<CmdBlockDesc>,
        /// Descriptors of the `CmdBlock`s that were not processed.
        ///
        /// The first block in this list is the one that was interrupted.
        cmd_blocks_not_processed: Vec<CmdBlockDesc>,
    },
    /// Execution ended due to an interruption between command blocks.
    ExecutionInterrupted {
        /// The outcome value.
        value: Option<T>,
        /// Descriptors of the `CmdBlock`s that were processed.
        cmd_blocks_processed: Vec<CmdBlockDesc>,
        /// Descriptors of the `CmdBlock`s that were not processed.
        cmd_blocks_not_processed: Vec<CmdBlockDesc>,
    },
    /// Execution ended due to one or more item errors.
    ///
    /// It is also possible for the stream to be interrupted when an error
    /// occurs, so the value is wrapped in a `ItemStreamOutcome`.
    ItemError {
        /// The outcome value.
        item_stream_outcome: ItemStreamOutcome<T>,
        /// Descriptors of the `CmdBlock`s that were processed.
        ///
        /// This does not include the `CmdBlock` that erred.
        cmd_blocks_processed: Vec<CmdBlockDesc>,
        /// Descriptors of the `CmdBlock`s that were not processed.
        ///
        /// The first block in this list is the one that erred.
        cmd_blocks_not_processed: Vec<CmdBlockDesc>,
        /// Item error(s) from the last command block's execution.
        errors: IndexMap<ItemId, E>,
    },
}

impl<T, E> CmdOutcome<T, E> {
    pub fn value(&self) -> Option<&T> {
        match self {
            CmdOutcome::Complete {
                value,
                cmd_blocks_processed: _,
            } => Some(value),
            CmdOutcome::BlockInterrupted {
                item_stream_outcome,
                cmd_blocks_processed: _,
                cmd_blocks_not_processed: _,
            } => Some(item_stream_outcome.value()),
            CmdOutcome::ExecutionInterrupted {
                value,
                cmd_blocks_processed: _,
                cmd_blocks_not_processed: _,
            } => value.as_ref(),
            CmdOutcome::ItemError {
                item_stream_outcome,
                cmd_blocks_processed: _,
                cmd_blocks_not_processed: _,
                errors: _,
            } => Some(item_stream_outcome.value()),
        }
    }

    /// Returns whether the command completed successfully.
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete { .. })
    }

    /// Returns whether the command completed successfully.
    pub fn is_interrupted(&self) -> bool {
        matches!(
            self,
            Self::BlockInterrupted { .. } | Self::ExecutionInterrupted { .. }
        )
    }

    /// Returns whether the command encountered item errors during execution.
    pub fn is_err(&self) -> bool {
        matches!(self, Self::ItemError { .. })
    }

    /// Maps the inner value to another, maintaining any collected errors.
    pub fn map<F, U>(self, f: F) -> CmdOutcome<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Complete {
                value: t,
                cmd_blocks_processed,
            } => {
                let u = f(t);
                CmdOutcome::Complete {
                    value: u,
                    cmd_blocks_processed,
                }
            }
            Self::BlockInterrupted {
                item_stream_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            } => {
                let item_stream_outcome = item_stream_outcome.map(f);
                CmdOutcome::BlockInterrupted {
                    item_stream_outcome,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                }
            }
            Self::ExecutionInterrupted {
                value: t,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            } => {
                let u = t.map(f);
                CmdOutcome::ExecutionInterrupted {
                    value: u,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                }
            }
            Self::ItemError {
                item_stream_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
                errors,
            } => {
                let item_stream_outcome = item_stream_outcome.map(f);
                CmdOutcome::ItemError {
                    item_stream_outcome,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                    errors,
                }
            }
        }
    }

    /// Maps the inner value to another asynchronously, maintaining any
    /// collected errors.
    pub async fn map_async<'f, F, Fut, U>(self, f: F) -> CmdOutcome<U, E>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = U> + 'f,
    {
        match self {
            Self::Complete {
                value: t,
                cmd_blocks_processed,
            } => {
                let u = f(t).await;
                CmdOutcome::Complete {
                    value: u,
                    cmd_blocks_processed,
                }
            }
            Self::BlockInterrupted {
                item_stream_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            } => {
                let (item_stream_outcome, value) = item_stream_outcome.replace(());
                let value = f(value).await;
                let (item_stream_outcome, ()) = item_stream_outcome.replace(value);
                CmdOutcome::BlockInterrupted {
                    item_stream_outcome,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                }
            }
            Self::ExecutionInterrupted {
                value: t,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            } => {
                let u = match t {
                    Some(t) => Some(f(t).await),
                    None => None,
                };
                CmdOutcome::ExecutionInterrupted {
                    value: u,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                }
            }
            Self::ItemError {
                item_stream_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
                errors,
            } => {
                let (item_stream_outcome, value) = item_stream_outcome.replace(());
                let value = f(value).await;
                let (item_stream_outcome, ()) = item_stream_outcome.replace(value);
                CmdOutcome::ItemError {
                    item_stream_outcome,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                    errors,
                }
            }
        }
    }
}

impl<T, E> CmdOutcome<Result<T, E>, E> {
    /// Transposes a `CmdOutcome<Result<T, E>, E>` to a `Result<CmdOutcome<T,
    /// E>, E>`.
    pub fn transpose(self) -> Result<CmdOutcome<T, E>, E> {
        match self {
            Self::Complete {
                value,
                cmd_blocks_processed,
            } => match value {
                Ok(value) => Ok(CmdOutcome::Complete {
                    value,
                    cmd_blocks_processed,
                }),
                Err(e) => Err(e),
            },
            Self::BlockInterrupted {
                item_stream_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            } => {
                let (item_stream_outcome, value) = item_stream_outcome.replace(());
                match value {
                    Ok(value) => {
                        let (item_stream_outcome, ()) = item_stream_outcome.replace(value);
                        Ok(CmdOutcome::BlockInterrupted {
                            item_stream_outcome,
                            cmd_blocks_processed,
                            cmd_blocks_not_processed,
                        })
                    }
                    Err(e) => Err(e),
                }
            }
            Self::ExecutionInterrupted {
                value,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            } => match value.transpose() {
                Ok(value) => Ok(CmdOutcome::ExecutionInterrupted {
                    value,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                }),
                Err(e) => Err(e),
            },
            Self::ItemError {
                item_stream_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
                errors,
            } => {
                let (item_stream_outcome, value) = item_stream_outcome.replace(());
                match value {
                    Ok(value) => {
                        let (item_stream_outcome, ()) = item_stream_outcome.replace(value);
                        Ok(CmdOutcome::ItemError {
                            item_stream_outcome,
                            cmd_blocks_processed,
                            cmd_blocks_not_processed,
                            errors,
                        })
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}
