use crate::{
    p2s,
    utils::{get_exe_version, is_url, parse_relative_path_with_located}, verify_enum,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{mixed_fs::MixedFS, verifiable::Verifiable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Software {
    pub scope: String,
    pub upstream: String,
    pub category: String,
    pub arch: Option<String>,
    pub language: String,
    pub main_program: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Verifiable for Software {
    fn verify_self(&self, located: &String) -> Result<()> {
        let err_wrapper = |e: anyhow::Error| {
            anyhow!(
                "Error:Failed to verify table 'software' in 'package.toml' : {e}"
            )
        };

        // 检查 arch 枚举
        if let Some(arch)=&self.arch{
            verify_enum!("arch",arch,"X64"|"X86"|"ARM64")?;
        }

        // 上游必须是 URL
        if !is_url(&self.upstream) {
            return Err(err_wrapper(anyhow!(
                "upstream should be a valid url, got '{text}'",
                text = self.upstream
            )));
        }

        // 主程序应该存在且可以读取版本号
        if let Some(main_program) = &self.main_program {
            let mixed_fs = MixedFS::new(located.clone());
            if !mixed_fs.exists(main_program) {
                return Err(err_wrapper(anyhow!(
                    "given main program '{main_program}' doesn't exist"
                )));
            }

            // 对于相对路径的主程序，尝试进行读取
            let mp_path = parse_relative_path_with_located(main_program, located);
            if mp_path.exists() {
                if let Err(e) = get_exe_version(p2s!(mp_path)) {
                    return Err(err_wrapper(anyhow!(
                        "failed to get main program ('{main_program}') file version : {e}"
                    )));
                }
            }
        }

        Ok(())
    }
}

#[test]
fn test_verify_software(){
    use crate::types::package::GlobalPackage;
    let located="".to_string();
    let base=GlobalPackage::_demo().software.unwrap();
    assert!(base.verify_self(&located).is_ok());

    let mut s1=base.clone();
    s1.arch=Some("X32".to_string());
    assert!(s1.verify_self(&located).is_err());
}