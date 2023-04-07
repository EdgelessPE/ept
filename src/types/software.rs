use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use crate::{utils::{is_url, parse_relative_path_with_located, get_exe_version}, p2s};

use super::{verifiable::Verifiable, mixed_fs::MixedFS};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Software {
    pub scope: String,
    pub upstream: String,
    pub category: String,
    pub language: String,
    pub main_program: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Verifiable for Software {
    fn verify_self(&self,located:&String) -> Result<()> {
        let err_wrapper=|e:anyhow::Error|anyhow!("Error:Failed to verify table 'software' in 'package.toml' : {err}",err=e.to_string());
        
        // 上游必须是 URL
        if !is_url(&self.upstream){
            return Err(err_wrapper(anyhow!("upstream should be a valid url, got '{text}'",text=self.upstream)));
        }

        // 主程序应该存在且可以读取版本号
        if let Some(main_program) =&self.main_program{
            let mixed_fs=MixedFS::new();
            if !mixed_fs.exists(main_program, located){
                return Err(err_wrapper(anyhow!("given main program '{main_program}' doesn't exist")));
            }

            // 对于相对路径的主程序，尝试进行读取
            let mp_path=parse_relative_path_with_located(main_program, located);
            if mp_path.exists(){
                if let Err(e)=get_exe_version(p2s!(mp_path)){
                    return Err(err_wrapper(anyhow!("failed to get main program ('{main_program}') file version : {err}",err=e.to_string())));
                }
            }
        }

        Ok(())
    }
}