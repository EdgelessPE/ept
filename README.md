<div align="center">
  <a href="https://ept.edgeless.top" target="_blank">
    <img alt="Nep Logo" width="200" src="logo/nep.png"/>
  </a>
</div>
<div align="center">
  <h1>ept</h1>
</div>

<div align="center">

（WIP）新一代 Windows 包管理解决方案 - 基于 Nep 规范打造

[![codecov](https://codecov.io/github/EdgelessPE/ept/graph/badge.svg?token=KF7Z1SSF3Q)](https://codecov.io/github/EdgelessPE/ept)

</div>

## 特性
* 🚀 使用 Rust 构建，编译后的体积小于 20MB 且无需任何运行时或动态链接库依赖，性能卓越
* 🔒 采用哈希算法 BLAKE3，配合 Ed25519 数字签名算法提供安全且极为快速的签名体验
* 📦 采用 Zstandard 压缩算法，实现高效的数据压缩和解压缩能力
* 🛠️ 完善的工作流设计，优雅的描述包的安装、更新、卸载等过程；支持从工作流自动生成反向工作流、权限信息、装箱单等信息
* 📝 完善的元信息管理能力，支持标签、权限控制等能力；支持识别程序自更新，支持识别注册表入口以获取主程序路径和卸载命令；支持安装包版和便携版软件包，支持可拓展软件包，支持自定义包类型偏好
* 🤖 生态链丰富，拥有完善的 CI/CD 流程，使用机器人自动构建并通过自动化质量保障系统确保包的质量


## 单元测试
* （可选）在根目录中创建 `eptrc.toml` 文件并指定 `local.base` 用于隔离测试安装环境
* 使用 `scoop install miniserve` 或 `cargo install --locked miniserve` 安装 [miniserve](https://github.com/svenstaro/miniserve)
* 执行 `pnpm rs:ut`
* 如需要查看单测覆盖率，请使用 `cargo install cargo-tarpaulin` 安装 [tarpaulin](https://github.com/xd009642/tarpaulin) 后执行 `pnpm rs:ut:html`

## 构建
使用 `cargo build` 构建测试版本的可执行文件，该文件运行时会默认启用 Debug 模式。

如果需要构建生产环境的版本，请执行 `pnpm rs:build`，这会调用 [vc-ltl](https://crates.io/crates/vc-ltl) 构建一个无需 VC 运行库的生产版本可执行文件。
