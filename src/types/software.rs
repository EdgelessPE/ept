use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Software {
    pub scope: String,
    pub upstream: String,
    pub category: String,
    pub main_program: Option<String>,
}
