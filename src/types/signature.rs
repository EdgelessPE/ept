use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signature {
    pub package: SignatureNode,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignatureNode {
    pub signer: String,
    pub signature: Option<String>,
}
