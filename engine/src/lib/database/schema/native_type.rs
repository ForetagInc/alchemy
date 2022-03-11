use serde::{Deserialize, Serialize};

/// The native types for Arango to store
#[derive(Serialize, Deserialize, PartialEq, Default, GraphQLEnum)]
pub enum SchemaNativeType {
    #[default]
    String,
    Integer,
    Boolean,
}

impl SchemaNativeType {
    pub fn as_str(&self) -> String {
        match self {
            SchemaNativeType::String => String::from("string"),
            SchemaNativeType::Integer => String::from("integer"),
            SchemaNativeType::Boolean => String::from("boolean"),
        }
    }
}
