use super::TStep;
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::types::permissions::{Generalizable, Permission, PermissionLevel};
use crate::utils::env::env_desktop;
use crate::{log, p2s, types::verifiable::Verifiable, utils::parse_relative_path};
use anyhow::{anyhow, Result};
use mslnk::ShellLink;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs::remove_file, path::Path};

lazy_static! {
    static ref TARGET_RE: Regex = Regex::new(r"^(([^/]+)/)?([^/]+)$").unwrap();
}

fn parse_target_name(name: &String) -> Result<(Option<String>, String)> {
    let sp: Vec<&str> = name.split("/").collect();
    let length = sp.len();
    if length > 2 {
        Err(anyhow!("Error:Invalid filed 'target_name' : '{}'", name))
    } else if length == 2 {
        Ok((
            Some(sp.get(0).unwrap().to_string()),
            sp.get(1).unwrap().to_string(),
        ))
    } else {
        Ok((None, sp.get(0).unwrap().to_string()))
    }
}

#[test]
fn test_parse_target_name() {
    let r1 = parse_target_name(&"Microsoft/Visual Studio Code".to_string()).unwrap();
    assert_eq!(
        r1,
        (
            Some("Microsoft".to_string()),
            "Visual Studio Code".to_string()
        )
    );

    let r2 = parse_target_name(&"尼..普".to_string()).unwrap();
    assert_eq!(r2, (None, "尼..普".to_string()));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepLink {
    pub source_file: String,
    pub target_name: String,
}

impl TStep for StepLink {
    fn run(self, located: &String, _: &GlobalPackage) -> anyhow::Result<i32> {
        // 获取用户桌面位置
        let desktop = env_desktop();

        // 解析源文件绝对路径
        let abs_clear_source_path =
            parse_relative_path(&p2s!(Path::new(located).join(&self.source_file)))?;
        // println!("{:?}",&abs_clear_source_path);
        let abs_clear_source = p2s!(abs_clear_source_path);

        // 创建实例
        let sl = ShellLink::new(&abs_clear_source)
            .map_err(|_| anyhow!("Error(Link):Can't find source file '{}'", &abs_clear_source))?;

        // 创建快捷方式
        let target = format!("{}/{}.lnk", desktop, &self.target_name);
        sl.create_lnk(&target).map_err(|err| {
            anyhow!(
                "Error(Link):Can't create link {}->{} : {}",
                &abs_clear_source,
                &target,
                err.to_string()
            )
        })?;
        log!("Info(Link):Added shortcut '{}'", target);
        Ok(0)
    }
    fn reverse_run(self, _: &String, _: &GlobalPackage) -> Result<()> {
        // 获取用户桌面位置
        let desktop = env_desktop();

        // 解析快捷方式路径
        let target = format!("{}/{}.lnk", desktop, &self.target_name);

        // 尝试删除
        let target_path = Path::new(&target);
        if target_path.exists() {
            remove_file(target_path)?;
            log!("Info(Link):Removed shortcut '{}'", target);
        }
        Ok(())
    }
    fn get_manifest(&self, _fs: &mut MixedFS) -> Vec<String> {
        vec![self.source_file.to_owned()]
    }
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            source_file: interpreter(self.source_file),
            target_name: interpreter(self.target_name),
        }
    }
}

impl Verifiable for StepLink {
    fn verify_self(&self) -> Result<()> {
        if !TARGET_RE.is_match(&self.target_name) {
            return Err(anyhow!(
                "Error(Link):Invalid field 'target_name', expect 'NAME' or 'FOLDER/NAME', got '{}'",
                &self.target_name
            ));
        }
        if self.target_name.contains("..") {
            return Err(anyhow!(
                "Error(Link):Invalid field 'target_name' : shouldn't contain '..'"
            ));
        }
        Ok(())
    }
}

impl Generalizable for StepLink {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: "link_desktop".to_string(),
            level: PermissionLevel::Normal,
            targets: vec![self.target_name.to_owned()],
        }])
    }
}

#[test]
fn test_link() {
    let step = StepLink {
        source_file: String::from("./Code.exe"),
        target_name: String::from("MS/VSC"),
    };
    step.verify_self().unwrap();
    // step.run(&String::from("./apps/VSCode"),&pkg)
    // .unwrap();
}
