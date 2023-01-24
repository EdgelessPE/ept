use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct WorkflowNode {
    pub name:String,
    pub step:String
}