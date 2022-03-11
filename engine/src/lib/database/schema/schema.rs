use super::Rule;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseSchema {
    pub message: String,
    pub level: String,
    pub rule: Rule,
}
