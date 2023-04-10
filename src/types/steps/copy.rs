use anyhow::{Result,anyhow, Ok};
use serde::{Deserialize, Serialize};
use crate::{types::{permissions::Generalizable,permissions::Permission,workflow::WorkflowContext,mixed_fs::MixedFS, verifiable::Verifiable}, utils::{common_wild_match_verify}, executor::{values_validator_path, judge_perm_level}};
use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepCopy{
    pub from:String,
    pub to:String,
    pub overwrite:Option<bool>
}

impl TStep for StepCopy {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        Ok(0)
    }
    fn reverse_run(self, cx: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.remove(&self.from);
        fs.add(&self.to);
        Vec::new()
    }
    fn interpret<F>(self, interpreter: F) -> Self
        where
            F: Fn(String) -> String {
        Self { from: interpreter(self.from), to: interpreter(self.to), overwrite: self.overwrite }
    }
}

impl Generalizable for StepCopy{
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let mut res=Vec::new();
        res.push(Permission{
            key:"fs_read".to_string(),
            level:judge_perm_level(&self.from)?,
            targets:vec![self.from.clone()]
        });
        res.push(Permission{
            key:"fs_write".to_string(),
            level:judge_perm_level(&self.to)?,
            targets:vec![self.to.clone()]
        });

        Ok(res)
    }
}

impl Verifiable for StepCopy{
    fn verify_self(&self, located: &String) -> Result<()> {
        values_validator_path(&self.from)?;
        values_validator_path(&self.to)?;
        common_wild_match_verify(&self.from,&self.to,located)
    }
}