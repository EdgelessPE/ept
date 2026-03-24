<div align="center">
  <a href="https://ept.edgeless.top" target="_blank">
    <img alt="Nep Logo" width="200" src="../../logo/nep.png"/>
  </a>
</div>
<div align="center">
  <h1>ept</h1>
</div>

<div align="center">

[![crates.io](https://img.shields.io/crates/v/ept-lib)](https://crates.io/crates/ept-lib)
[![License: MIT](https://img.shields.io/crates/l/ept-lib)](https://opensource.org/licenses/MIT)

</div>

## 简介

`ept-lib` 是 [Edgeless Package Tool (ept)](https://github.com/EdgelessPE/ept) 的核心库，为 Windows 提供包管理、工作流执行、签名验证等功能。

## 功能特性

- **包管理**：支持包的安装、卸载、更新、搜索、元信息查看
- **工作流引擎**：完善的工作流设计，描述包的安装、更新、卸载等过程
- **签名验证**：采用 BLAKE3 哈希算法 + Ed25519 数字签名，提供安全快速的签名体验
- **高效压缩**：采用 Zstandard 压缩算法
- **镜像源管理**：支持添加、移除、更新镜像源
- **配置管理**：灵活的配置系统，支持自定义设置
- **Windows 集成**：支持注册表入口、快捷方式创建、系统通知等

## 快速开始

### 添加依赖

```toml
cargo add ept-lib
```

### 基本用法

```rust
use ept_lib::{Cfg, EptInstance, RuntimeContext};

let cfg = Cfg::new()?;
let ctx = RuntimeContext::from_cfg(&cfg)?;
let ept = EptInstance::new(ctx);

// 搜索包
let results = ept.search("vscode", false)?;

// 列出已安装的包
let installed = ept.list()?;

// 安装包
ept.install_using_package("path/to/package.nep", true)?;
```

## 主要 API

### EptInstance

封装所有操作的入口结构体：

| 方法 | 描述 |
|------|------|
| `new(runtime_ctx)` | 创建新实例 |
| `search(text, is_regex)` | 搜索包 |
| `list()` | 列出已安装的包 |
| `info(target, verify)` | 获取包信息 |
| `install_using_package(source, verify)` | 使用包文件安装 |
| `uninstall(scope, name)` | 卸载包 |
| `update_all(verify)` | 更新所有包 |
| `mirror_list()` | 列出镜像源 |
| `mirror_add(url, name)` | 添加镜像源 |
| `clean()` | 清理临时文件 |

### 配置类型

- `Cfg` - 包配置结构
- `RuntimeContext` - 运行时上下文
- `Info` - 包信息
- `MirrorInfo` - 镜像源信息

## 许可

本项目基于 [MIT](https://opensource.org/licenses/MIT) 许可开源。

## 相关链接

- [ept 主页](https://ept.edgeless.top)
- [GitHub 仓库](https://github.com/EdgelessPE/ept)
- [ crates.io](https://crates.io/crates/ept-lib)
