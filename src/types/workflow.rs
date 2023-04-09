use super::{
    package::GlobalPackage, permissions::Generalizable, steps::Step, verifiable::Verifiable,
};
use crate::types::permissions::Permission;
use anyhow::Result;
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

impl Generalizable for WorkflowNode {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let mut perm = Vec::new();
        perm.append(&mut self.header.generalize_permissions()?);
        perm.append(&mut self.body.generalize_permissions()?);

        Ok(perm)
    }
}

impl Verifiable for WorkflowNode {
    fn verify_self(&self, located: &String) -> anyhow::Result<()> {
        self.header.verify_self(located)?;
        self.body.verify_self(located)
    }
}

pub struct WorkflowContext {
    pub located: String,
    pub pkg: GlobalPackage,
}
