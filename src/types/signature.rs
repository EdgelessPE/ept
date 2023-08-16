use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signature {
    pub package: SignatureNode,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignatureNode {
    pub raw_name_stem: String,
    pub signer: String,
    pub signature: Option<String>,
}
