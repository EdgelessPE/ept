use anyhow::Result;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, TS)]
#[ts(export)]
pub enum PermissionLevel {
    /// 普通权限
    Normal,
    /// 重要权限
    Important,
    /// 敏感权限
    Sensitive,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, TS, EnumString,
)]
#[allow(non_camel_case_types)]
pub enum PermissionKey {
    /// 创建/删除 PATH 入口，指向某个文件
    path_entrances,
    /// 直接向 PATH 变量中添加/删除目录
    path_dirs,
    /// 在桌面创建快捷方式
    link_desktop,
    /// 在开始菜单创建快捷方式
    link_startmenu,
    /// 执行软件安装器
    execute_installer,
    /// 执行未知的自定义命令
    execute_custom,
    /// 读文件系统
    fs_read,
    /// 写文件系统
    fs_write,
    /// 查询某个进程是否正在运行
    process_query,
    /// 查询某个 Nep 包是否已被安装
    nep_installed,
    /// 弹出 Toast 通知消息
    notify_toast,
    /// 杀死进程
    process_kill,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export)]
pub struct Permission {
    pub key: PermissionKey,
    pub level: PermissionLevel,
    pub targets: Vec<String>,
}

pub trait Generalizable {
    fn generalize_permissions(&self) -> Result<Vec<Permission>>;
}
