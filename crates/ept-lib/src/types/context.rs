use crate::types::cfg::Cfg;
use crate::types::interaction::{InteractionProvider, NoInteraction};
use crate::types::mixed_fs::MixedFS;
use crate::types::package::GlobalPackage;
use crate::utils::test::_default_test_cfg;
use crate::{log, p2s};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Child;
use std::sync::Arc;

/// 通用上下文结构体，包含配置和交互提供者
#[derive(Clone, Debug)]
pub struct RuntimeContext {
    /// 配置信息
    pub cfg: Cfg,
    /// 交互提供者
    pub interaction_provider: Arc<dyn InteractionProvider>,
}

impl RuntimeContext {
    /// 创建新的 RuntimeContext
    pub fn new(cfg: Cfg, interaction_provider: Arc<dyn InteractionProvider>) -> Self {
        Self {
            cfg,
            interaction_provider,
        }
    }

    /// 设置交互提供者（链式调用）
    pub fn with_interaction_provider<T: InteractionProvider + 'static>(
        mut self,
        provider: T,
    ) -> Self {
        self.interaction_provider = Arc::new(provider);
        self
    }

    /// 获取交互提供者
    pub fn interaction(&self) -> &dyn InteractionProvider {
        self.interaction_provider.as_ref()
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self {
            cfg: Cfg::default(),
            interaction_provider: Arc::new(NoInteraction),
        }
    }
}

/// 用于校验的上下文
pub struct VerifiableCtx<'a> {
    pub mixed_fs: &'a MixedFS,
    pub runtime_ctx: &'a RuntimeContext,
}

/// 步骤验证上下文
pub struct VerifyStepCtx<'a> {
    pub mixed_fs: &'a MixedFS,
    pub runtime_ctx: &'a RuntimeContext,
    pub is_expand_flow: bool,
}

/// 缓存操作上下文
// （是否启用缓存，源文件，Option<(缓存目录, 缓存 key)>）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CacheCtx(
    pub bool,
    pub std::path::PathBuf,
    pub Option<(std::path::PathBuf, String)>,
);

/// 工作流执行上下文
pub struct WorkflowContext<'a> {
    pub located: String,
    pub pkg: GlobalPackage,
    pub async_execution_handlers: Vec<(String, Child, bool)>, // 命令，handler，是否被抛弃
    pub exit_code: i32,
    pub runtime_ctx: &'a RuntimeContext,
}

impl WorkflowContext {
    pub fn _demo() -> Self {
        let runtime_ctx: &'static RuntimeContext = Box::leak(Box::new(_default_test_cfg()));
        Self {
            pkg: GlobalPackage::_demo(),
            located: p2s!(std::env::current_dir().unwrap()),
            async_execution_handlers: Vec::new(),
            exit_code: 0,
            runtime_ctx
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

impl<'a> VerifyStepCtx<'a> {
    pub fn _demo() -> Self {
        use crate::utils::test::_default_test_cfg;
        
        let runtime_ctx: _default_test_cfg();
        Self {
            mixed_fs: &MixedFS::new(""),
            runtime_ctx,
            is_expand_flow: false,
        }
    }
}
