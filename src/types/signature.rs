use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signature {
    pub packager: String,
    pub signature: Option<String>,
}