use std::fmt;

use peace_item_model::ItemId;
use serde::{Deserialize, Serialize};

use crate::{FieldNameAndType, ValueResolutionMode};

/// Collects information about how a value is resolved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueResolutionCtx {
    /// When resolving `Value`s, whether to look up `Current<T>` or
    /// `Goal<T>`.
    value_resolution_mode: ValueResolutionMode,
    /// ID of the item whose params are being resolved.
    item_id: ItemId,
    /// Name of the `Item::Params` type.
    params_type_name: String,
    /// Hierarchy of fields traversed to resolve this value.
    resolution_chain: Vec<FieldNameAndType>,
}

impl ValueResolutionCtx {
    pub fn new(
        value_resolution_mode: ValueResolutionMode,
        item_id: ItemId,
        params_type_name: String,
    ) -> Self {
        Self {
            value_resolution_mode,
            item_id,
            params_type_name,
            resolution_chain: Vec::new(),
        }
    }

    pub fn value_resolution_mode(&self) -> ValueResolutionMode {
        self.value_resolution_mode
    }

    pub fn item_id(&self) -> &ItemId {
        &self.item_id
    }

    pub fn params_type_name(&self) -> &str {
        &self.params_type_name
    }

    pub fn resolution_chain(&self) -> &[FieldNameAndType] {
        self.resolution_chain.as_ref()
    }

    /// Appends a field name and type to the resolution chain.
    pub fn push(&mut self, field_name_and_type: FieldNameAndType) {
        self.resolution_chain.push(field_name_and_type);
    }

    /// Removes a field name and type from the resolution chain.
    pub fn pop(&mut self) {
        self.resolution_chain.pop();
    }
}

impl fmt::Display for ValueResolutionCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params_type_name = self.params_type_name();
        write!(f, "{params_type_name} {{")?;

        if let Some((last, chain)) = self.resolution_chain().split_last() {
            writeln!(f)?;

            chain
                .iter()
                .enumerate()
                .try_for_each(|(indentation, field_name_and_type)| {
                    let indentation = indentation + 1;
                    (0..indentation).try_for_each(|_| write!(f, "    "))?;

                    let field_name = field_name_and_type.field_name();
                    let type_name = field_name_and_type.type_name();
                    writeln!(f, "{field_name}: {type_name} {{")
                })?;

            // Don't add opening `{` for the actual field.
            let indentation = self.resolution_chain().len();
            (0..indentation).try_for_each(|_| write!(f, "    "))?;

            let field_name = last.field_name();
            let type_name = last.type_name();
            writeln!(f, "{field_name}: {type_name},")?;

            (0..indentation).try_for_each(|_| write!(f, "    "))?;
            writeln!(f, "..")?;
        }

        (0..self.resolution_chain().len())
            .rev()
            .skip(1)
            .try_for_each(|indentation| {
                let indentation = indentation + 1;
                (0..indentation).try_for_each(|_| write!(f, "    "))?;
                writeln!(f, "}},")?;

                (0..indentation).try_for_each(|_| write!(f, "    "))?;
                writeln!(f, "..")
            })?;

        write!(f, "}}")
    }
}
