use serde::{Deserialize, Serialize};

/// A field name and its type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldNameAndType {
    /// Name of the field, e.g. `my_field`.
    field_name: String,
    /// Name of the type, e.g. `MyField`.
    type_name: String,
}

impl FieldNameAndType {
    pub fn new(field_name: String, type_name: String) -> Self {
        Self {
            field_name,
            type_name,
        }
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}
