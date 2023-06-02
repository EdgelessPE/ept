use anyhow::{Result,anyhow, Ok};
use serde::{Deserialize, Serialize};
use crate::{types::{permissions::Generalizable,permissions::Permission,workflow::WorkflowContext,mixed_fs::MixedFS, verifiable::Verifiable}, utils::{common_wild_match_verify, contains_wild_match, try_recycle, parse_wild_match}, executor::{values_validator_path, judge_perm_level}, p2s};
use super::{TStep, copy::parse_target_for_copy};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepMove{
    pub from:String,
    pub to:String,
    pub overwrite:Option<bool>
}

fn mv(from:&String,to:&String,located:&String,overwrite:bool,wild_match_mode:bool)->Result<()>{
    let (to_path,_)=parse_target_for_copy(from, to,located,wild_match_mode)?;
    if to_path.exists(){
        if overwrite{
            try_recycle(&to_path)?;
        }else{
            // 如果不覆盖则不需要移动
            return Ok(());
        }
    }
    std::fs::rename(from, &to_path).map_err(|e|anyhow!("Error:Failed to move file from '{from}' to '{to_str}' : {err}",err=e.to_string(),to_str=p2s!(to_path)))
}

impl TStep for StepMove {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        let overwrite=self.overwrite.unwrap_or(false);
        if contains_wild_match(&self.from){
            for from in parse_wild_match(self.from, &cx.located)?{
                mv(&p2s!(from), &self.to,&cx.located, overwrite,true)?;
            }
        }else{
            mv(&self.from, &self.to,&cx.located, overwrite,false)?;
        }

        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        fs.remove(&self.from);
        fs.add(&self.to,&self.from);
        Vec::new()
    }
    fn interpret<F>(self, interpreter: F) -> Self
        where
            F: Fn(String) -> String {
        Self { from: interpreter(self.from), to: interpreter(self.to), overwrite: self.overwrite }
    }
}

impl Generalizable for StepMove{
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        let mut res=Vec::new();
        res.push(Permission{
            key:"fs_write".to_string(),
            level:judge_perm_level(&self.from)?,
            targets:vec![self.from.clone(),self.to.clone()]
        });

        Ok(res)
    }
}

impl Verifiable for StepMove{
    fn verify_self(&self, located: &String) -> Result<()> {
        values_validator_path(&self.from)?;
        values_validator_path(&self.to)?;
        common_wild_match_verify(&self.from,&self.to,located)
    }
}

#[test]
fn test_copy(){
    use std::path::Path;
    use crate::types::package::GlobalPackage;
    use std::fs::remove_dir_all;
    use fs_extra::dir::CopyOptions;
    envmnt::set("DEBUG", "true");
    let mut cx=WorkflowContext { located: String::from("D:/Desktop/Projects/EdgelessPE/ept"), pkg: GlobalPackage::_demo() };
    remove_dir_all("test").unwrap();

    // 准备源
    let opt=CopyOptions::new();
    fs_extra::dir::copy("src","test/src",&opt);
    fs_extra::dir::copy("keys","test/src/keys",&opt);

    // 文件-文件
    StepMove{
        from: "test/src/types/author.rs".to_string(),
        to: "test/1.rs".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/1.rs").exists());
    assert!(!Path::new("test/src/types/author.rs").exists());

    // 文件-不存在目录
    StepMove{
        from: "test/src/types/extended_semver.rs".to_string(),
        to: "test/ca/".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/ca/extended_semver.rs").exists());
    assert!(!Path::new("test/src/types/extended_semver.rs").exists());

    // 文件-已存在目录
    StepMove{
        from: "test/src/types/info.rs".to_string(),
        to: "test/ca".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/ca/info.rs").exists());
    assert!(!Path::new("test/src/types/info.rs").exists());

    // 目录-不存在目录
    StepMove{
        from: "test/src/entrances".to_string(),
        to: "test/entry1".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/entry1/utils/mod.rs").exists());
    assert!(!Path::new("test/src/entrances").exists());

    // 目录-不存在目录
    StepMove{
        from: "test/src/ca".to_string(),
        to: "test/entry2/".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/entry2/mod.rs").exists());
    assert!(!Path::new("test/src/ca").exists());

    // 目录-已存在目录
    StepMove{
        from: "test/src/executor".to_string(),
        to: "test/entry2/entrances".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/entry2/entrances/README.md").exists());
    assert!(!Path::new("test/src/executor").exists());


    // 通配符文件-不存在目录
    StepMove{
        from: "test/src/*.rs".to_string(),
        to: "test/main".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/main/main.rs").exists());
    assert!(!Path::new("test/src/main.rs").exists());

    // 通配符目录-目录
    StepMove{
        from: "test/src/key?".to_string(),
        to: "test/keys".to_string(),
        overwrite: None,
    }.run(&mut cx).unwrap();
    assert!(Path::new("test/keys/keys/public.pem").exists());
    assert!(!Path::new("test/src/keys/public.pem").exists());

}