use serde::{Deserialize, Serialize};

use super::{steps::Step, verifiable::Verifiable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkflowHeader {
    pub name: String,
    pub step: String,
    pub c_if: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkflowNode {
    pub header: WorkflowHeader,
    pub body: Step,
}

impl Verifiable for WorkflowNode {
    fn verify_self(&self) -> anyhow::Result<()> {
        self.header.verify_self()?;
        self.body.verify_self()
    }
}