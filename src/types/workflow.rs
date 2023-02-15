use serde::{Deserialize, Serialize};

use super::{steps::{StepExecute,StepLink,StepLog,StepPath}, TStep};

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

impl TStep for Step {
    fn run(self,located:&String)->anyhow::Result<i32> {
        self.run(located)
    }
    fn reverse_run(self,located:&String)->anyhow::Result<()> {
        self.reverse_run(located)
    }
    fn get_manifest(&self)->Vec<String> {
        self.get_manifest()
    }
    fn interpret<F>(self,interpreter:F)->Self
        where F:Fn(String)->String {
        self.interpret(interpreter)
    }
}