use fn_graph::StreamOutcomeState;
use peace_item_model::ItemId;

/// How a `Flow` stream operation ended and IDs that were processed.
///
/// Currently this is constructed by `ItemStreamOutcomeMapper`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStreamOutcome<T> {
    /// The value of the outcome.
    pub value: T,
    /// How a `Flow` stream operation ended.
    pub state: StreamOutcomeState,
    /// IDs of the items that were processed.
    pub item_ids_processed: Vec<ItemId>,
    /// IDs of the items that were not processed.
    pub item_ids_not_processed: Vec<ItemId>,
}

impl<T> ItemStreamOutcome<T> {
    /// Returns an `ItemStreamOutcome` that is `Finished<T>`.
    pub fn finished_with(value: T, item_ids_processed: Vec<ItemId>) -> Self {
        Self {
            value,
            state: StreamOutcomeState::Finished,
            item_ids_processed,
            item_ids_not_processed: Vec::new(),
        }
    }

    /// Maps this outcome's value to another.
    pub fn map<TNew>(self, f: impl FnOnce(T) -> TNew) -> ItemStreamOutcome<TNew> {
        let ItemStreamOutcome {
            value,
            state,
            item_ids_processed,
            item_ids_not_processed,
        } = self;

        let value = f(value);

        ItemStreamOutcome {
            value,
            state,
            item_ids_processed,
            item_ids_not_processed,
        }
    }

    /// Replaces the value from this outcome with another.
    pub fn replace<TNew>(self, value_new: TNew) -> (ItemStreamOutcome<TNew>, T) {
        let ItemStreamOutcome {
            value: value_existing,
            state,
            item_ids_processed,
            item_ids_not_processed,
        } = self;

        (
            ItemStreamOutcome {
                value: value_new,
                state,
                item_ids_processed,
                item_ids_not_processed,
            },
            value_existing,
        )
    }

    /// Replaces the value from this outcome with another, taking the current
    /// value as a parameter.
    pub fn replace_with<TNew, U>(
        self,
        f: impl FnOnce(T) -> (TNew, U),
    ) -> (ItemStreamOutcome<TNew>, U) {
        let ItemStreamOutcome {
            value,
            state,
            item_ids_processed,
            item_ids_not_processed,
        } = self;

        let (value, extracted) = f(value);

        (
            ItemStreamOutcome {
                value,
                state,
                item_ids_processed,
                item_ids_not_processed,
            },
            extracted,
        )
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn state(&self) -> StreamOutcomeState {
        self.state
    }

    pub fn item_ids_processed(&self) -> &[ItemId] {
        self.item_ids_processed.as_ref()
    }

    pub fn item_ids_not_processed(&self) -> &[ItemId] {
        self.item_ids_not_processed.as_ref()
    }
}

impl<T> Default for ItemStreamOutcome<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: T::default(),
            state: StreamOutcomeState::NotStarted,
            item_ids_processed: Vec::new(),
            item_ids_not_processed: Vec::new(),
        }
    }
}
