use std::path::{Path, PathBuf};

use anyhow::{Result,anyhow, Ok};
use fs_extra::dir::CopyOptions;
use serde::{Deserialize, Serialize};
use crate::{types::{permissions::Generalizable,permissions::Permission,workflow::WorkflowContext,mixed_fs::MixedFS, verifiable::Verifiable}, utils::{common_wild_match_verify, common_merge_wild_match, contains_wild_match, ensure_dir_exist, try_recycle, parse_wild_match}, executor::{values_validator_path, judge_perm_level}, p2s, log};
use super::TStep;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepCopy{
    pub from:String,
    pub to:String,
    pub overwrite:Option<bool>
}

// 入参不应包含通配符，返回 （指向父目录存在的目标路径，是否在拷贝文件）
pub fn parse_target_for_copy(from:&String,to:&String)->Result<(PathBuf,bool)>{
    let from_path=Path::new(from);
    let to_path=Path::new(to);

    // 如果 from 不存在直接报错
    if !from_path.exists(){
        return Err(anyhow!("Error:Field 'from' refers to a non-existent target : '{from}'"));
    }

    // 如果 from 是文件夹，则 to 直接视为文件夹
    if from_path.is_dir(){
        ensure_dir_exist(to_path.parent().unwrap())?;
        return Ok((to_path.to_path_buf(),false));
    }else{
        // 此时拷贝的内容为文件，需要确定 to 的性质然后决定是否需要拼接文件名

        // 如果 to 已存在则直接进行判断
        if to_path.exists(){
            if to_path.is_file(){
                return Ok((to_path.to_path_buf(),true));
            }else if to_path.is_dir(){
                let file_name=from_path.file_name().unwrap();
                return Ok((to_path.join(file_name).to_path_buf(),true));
            }else{
                return Err(anyhow!("Error:Field 'to' refers to a existing abnormal target : '{to}'"));
            }
        }
    
        // 从字面规则判断 to 是否为文件夹
        if to.ends_with("/"){
            // 此时 from 是文件，说明 to 指向的是父目录，因此进行拼接
            let file_name=from_path.file_name().unwrap();
            ensure_dir_exist(to_path)?;
            return Ok((to_path.join(file_name).to_path_buf(),true));
        }

        // 兜底，表示 to 是文件路径
        ensure_dir_exist(to_path.parent().unwrap())?;
        return Ok((to_path.to_path_buf(),true));
    }

}

fn copy(from:&String,to:&String,overwrite:bool)->Result<()>{
    let (to_path,is_copy_file)=parse_target_for_copy(from, to)?;
    if to_path.exists(){
        if overwrite{
            try_recycle(&to_path)?;
        }else{
            // 如果不覆盖则不需要复制
            return Ok(());
        }
    }
    if is_copy_file{
        std::fs::copy(from, to_path).map_err(|e|anyhow!("Error:Failed to copy file from '{from}' to '{to}' : {err}",err=e.to_string()))?;
    }else{
        fs_extra::dir::copy(from, to_path, &CopyOptions::new()).map_err(|e|anyhow!("Error:Failed to copy dir from '{from}' to '{to}' : {err}",err=e.to_string()))?;
    }

    Ok(())
}

impl TStep for StepCopy {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        let overwrite=self.overwrite.unwrap_or(false);
        if contains_wild_match(&self.from){
            for from in parse_wild_match(self.from, &cx.located)?{
                copy(&p2s!(from), &self.to, overwrite)?;
            }
        }else{
            copy(&self.from, &self.to, overwrite)?;
        }

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.remove(&self.from);
        fs.add(&common_merge_wild_match(&self.from, &self.to));
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