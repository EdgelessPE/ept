use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Step {
    StepExecute(StepExecute),
    StepLink(StepLink),
    StepLog(StepLog),
    StepPath(StepPath),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLink {
    pub source_file: String,
    pub target_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepExecute {
    pub command: String,
    pub pwd: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepPath {
    pub record: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLog {
    pub level: String,
    pub msg: String,
}
