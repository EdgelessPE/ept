#[macro_use]
extern crate lazy_static;
extern crate tar;
#[macro_use]
extern crate tantivy;

pub mod ca;
pub mod compression;
pub mod entrances;
pub mod executor;
pub mod parsers;
pub mod signature;
#[macro_use]
pub mod types;
#[macro_use]
pub mod utils;

// Re-export commonly used types
pub use types::{
    cfg::Cfg,
    cli::{Action, ActionConfig, Args},
};

// Import entrance functions
use entrances::{
    auto_mirror_update_all, clean,
    config::{config_get, config_init, config_list, config_set, config_which},
    info, install_using_package, install_using_parsed, list, meta, mirror_add, mirror_list,
    mirror_remove, mirror_update, mirror_update_all, pack, search, uninstall, update_all,
    update_using_parsed, upgrade,
};

/// EptInstance 结构体，封装 Cfg 并提供所有 entrances 函数作为方法
pub struct EptInstance {
    runtime_ctx: RuntimeContext,
}

impl EptInstance {
    /// 创建新的 EptInstance 实例
    pub fn new(runtime_ctx: RuntimeContext) -> Self {
        Self { runtime_ctx }
    }

    /// 获取内部 RuntimeContext 的引用
    pub fn runtime_ctx(&self) -> &RuntimeContext {
        &self.runtime_ctx
    }

    /// 获取内部 RuntimeContext 的可变引用
    pub fn runtime_ctx_mut(&mut self) -> &mut RuntimeContext {
        &mut self.runtime_ctx
    }

    /// 自动镜像更新
    pub fn auto_mirror_update_all(&self) -> anyhow::Result<bool> {
        auto_mirror_update_all(&self.runtime_ctx)
    }

    /// 清理临时文件和无效目录
    pub fn clean(&self) -> anyhow::Result<usize> {
        clean(&self.runtime_ctx)
    }

    /// 获取配置值
    pub fn config_get(&self, table: &str, key: &str) -> anyhow::Result<String> {
        config_get(&self.runtime_ctx, table, key)
    }

    /// 初始化配置文件
    pub fn config_init(&self) -> anyhow::Result<String> {
        config_init(&self.runtime_ctx)
    }

    /// 列出所有配置
    pub fn config_list(&self) -> anyhow::Result<String> {
        config_list(&self.runtime_ctx)
    }

    /// 设置配置值
    pub fn config_set(&self, table: &str, key: &str, value: &str) -> anyhow::Result<()> {
        config_set(&self.runtime_ctx, table, key, value)
    }

    /// 配置所在位置
    pub fn config_which(&self) -> anyhow::Result<String> {
        config_which()
    }

    /// 获取包信息
    pub fn info(
        &self,
        target_input: types::matcher::PackageInputEnum,
        verify_signature: bool,
    ) -> anyhow::Result<(types::info::Info, Option<std::path::PathBuf>)> {
        info(&self.runtime_ctx, target_input, verify_signature)
    }

    /// 使用包文件安装
    pub fn install_using_package(
        &self,
        source_file: &str,
        verify_signature: bool,
    ) -> anyhow::Result<(String, String)> {
        install_using_package(&self.runtime_ctx, source_file, verify_signature)
    }

    /// 使用解析后的输入安装
    pub fn install_using_parsed(
        &self,
        parsed: Vec<utils::parse_inputs::ParseInputResEnum>,
        verify_signature: bool,
    ) -> anyhow::Result<Vec<(String, String)>> {
        install_using_parsed(&self.runtime_ctx, parsed, verify_signature)
    }

    /// 列出已安装的包
    pub fn list(&self) -> anyhow::Result<Vec<types::info::Info>> {
        list(&self.runtime_ctx)
    }

    /// 获取包的元数据
    pub fn meta(
        &self,
        input: types::matcher::PackageInputEnum,
        verify_signature: bool,
    ) -> anyhow::Result<types::meta::MetaResult> {
        meta(&self.runtime_ctx, input, verify_signature)
    }

    /// 添加镜像源
    pub fn mirror_add(
        &self,
        url: &str,
        should_match_name: Option<String>,
    ) -> anyhow::Result<String> {
        mirror_add(&self.runtime_ctx, url, should_match_name)
    }

    /// 列出所有镜像源
    pub fn mirror_list(&self) -> anyhow::Result<Vec<types::mirror::MirrorInfo>> {
        mirror_list(&self.runtime_ctx)
    }

    /// 移除镜像源
    pub fn mirror_remove(&self, name: &str) -> anyhow::Result<()> {
        mirror_remove(&self.runtime_ctx, name)
    }

    /// 更新指定镜像源
    pub fn mirror_update(&self, name: &str) -> anyhow::Result<String> {
        mirror_update(&self.runtime_ctx, name)
    }

    /// 更新所有镜像源
    pub fn mirror_update_all(&self) -> anyhow::Result<Vec<String>> {
        mirror_update_all(&self.runtime_ctx)
    }

    /// 打包目录为 nep 文件
    pub fn pack(
        &self,
        source_dir: &str,
        into_file: Option<String>,
        need_sign: bool,
    ) -> anyhow::Result<String> {
        pack(&self.runtime_ctx, source_dir, into_file, need_sign)
    }

    /// 搜索包
    pub fn search(
        &self,
        text: &str,
        is_regex: bool,
    ) -> anyhow::Result<Vec<types::mirror::SearchResult>> {
        search(&self.runtime_ctx, text, is_regex)
    }

    /// 卸载包
    pub fn uninstall(
        &self,
        scope: Option<String>,
        package_name: &str,
    ) -> anyhow::Result<(String, String)> {
        uninstall(&self.runtime_ctx, scope, package_name)
    }

    /// 更新所有包
    pub fn update_all(&self, verify_signature: bool) -> anyhow::Result<(i32, i32)> {
        update_all(&self.runtime_ctx, verify_signature)
    }

    /// 使用解析后的输入更新
    pub fn update_using_parsed(
        &self,
        parsed: Vec<utils::parse_inputs::ParseInputResEnum>,
        verify_signature: bool,
    ) -> anyhow::Result<Vec<types::info::UpdateInfo>> {
        update_using_parsed(&self.runtime_ctx, parsed, verify_signature)
    }

    /// 升级 ept 工具链
    pub fn upgrade(&self, dry_run: bool, need_exit_process: bool) -> anyhow::Result<String> {
        upgrade(&self.runtime_ctx, dry_run, need_exit_process)
    }
}

// Re-export utility functions
use crate::types::context::RuntimeContext;
pub use utils::{
    flags::{get_flag, set_flag, Flag},
    launch_clean,
};
