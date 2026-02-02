use crate::types::cfg::Cfg;
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::utils::test::_default_test_cfg;
use crate::{log, p2s};
use anyhow::{anyhow, Result};
use std::process::Child;

/// 工作流执行上下文
pub struct WorkflowContext {
    pub located: String,
    pub pkg: GlobalPackage,
    pub async_execution_handlers: Vec<(String, Child, bool)>, // 命令，handler，是否被抛弃
    pub exit_code: i32,
    pub cfg: Cfg,
}

/// 用于校验的上下文
pub struct VerifiableCtx<'a> {
    pub mixed_fs: &'a MixedFS,
    pub cfg: &'a Cfg,
}

/// 步骤验证上下文
pub struct VerifyStepCtx {
    pub mixed_fs: MixedFS,
    pub cfg: Cfg,
    pub is_expand_flow: bool,
}

/// 缓存操作上下文
// （是否启用缓存，源文件，Option<(缓存目录, 缓存 key)>）
#[derive(Debug)]
pub struct CacheCtx(
    pub bool,
    pub std::path::PathBuf,
    pub Option<(std::path::PathBuf, String)>,
);

impl WorkflowContext {
    pub fn _demo() -> Self {
        Self::new(
            _default_test_cfg(),
            &p2s!(std::env::current_dir().unwrap()),
            GlobalPackage::_demo(),
        )
    }

    pub fn new(cfg: Cfg, located: &str, pkg: GlobalPackage) -> Self {
        Self {
            pkg,
            located: located.to_owned(),
            async_execution_handlers: Vec::new(),
            exit_code: 0,
            cfg,
        }
    }

    pub fn finish(self) -> Result<i32> {
        use crate::utils::term::read_console;

        log!("Debug:Finish context");

        // 等待异步 handlers
        for (cmd, mut handler, abandon) in self.async_execution_handlers {
            if abandon {
                if let Err(e) = handler.kill() {
                    log!("Warning(Execute):Failed to kill async abandoned command '{cmd}' : {e}");
                } else {
                    log!("Info(Execute):Killed async abandoned command '{cmd}'");
                }
            } else {
                let output = handler.wait_with_output().map_err(|e| {
                    anyhow!("Error(Execute):Failed to wait on async command '{cmd}' : {e}")
                })?;
                // 处理退出码
                match output.status.code() {
                    Some(val) => {
                        if val == 0 {
                            log!("Info(Execute):Async command '{cmd}' output :");
                            println!("{output}", output = read_console(output.stdout));
                        } else {
                            log!("Error(Execute):Async command '{cmd}' failed, output :");
                            println!("{output}", output = read_console(output.stdout));
                        }
                    }
                    None => {
                        log!("Error(Execute):Async command '{cmd}' terminated by signal");
                    }
                }
            }
        }

        Ok(self.exit_code)
    }
}

impl VerifyStepCtx {
    pub fn _demo() -> Self {
        Self {
            mixed_fs: MixedFS::new(""),
            cfg: _default_test_cfg(),
            is_expand_flow: false,
        }
    }
}
