use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Software {
    pub scope: String,
    pub upstream: String,
    pub category: String,
    pub language: String,
    pub main_program: Option<String>,
    pub tags: Option<Vec<String>>,
}
