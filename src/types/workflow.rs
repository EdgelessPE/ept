use serde::{Deserialize, Serialize};

use super::steps::Step;

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
