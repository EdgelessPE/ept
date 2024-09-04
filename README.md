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
* 🚀 使用 Rust 构建，执行迅速，性能卓越
* ✨ 体积小于 20MB 的单文件应用，无需任何运行时依赖
* 🔒 摘要采用最新的哈希算法 BLAKE3，提供安全且极为快速的摘要计算体验
* 🌐 数字签名采用 Ed25519 算法，兼顾高性能和高安全性，保障包的完整性
* 📦 采用 Zstandard 压缩算法，实现高效的数据压缩和解压缩能力
* 🛠️ 完善的工作流设计，优雅的描述包的安装、更新、卸载等过程
* 🤖 生态链丰富，拥有完善的 CI/CD 流程，使用机器人自动构建并通过自动化质量保障系统确保包的质量


## 单元测试
* （可选）在根目录中创建 `eptrc.toml` 文件并指定 `local.base` 用于隔离测试安装环境
* 使用 `scoop install miniserve` 或 `cargo install --locked miniserve` 安装 [miniserve](https://github.com/svenstaro/miniserve)
* 执行 `pnpm rust:ut`
* 如需要查看单测覆盖率，请使用 `cargo install cargo-tarpaulin` 安装 [tarpaulin](https://github.com/xd009642/tarpaulin) 后执行 `pnpm rust:ut:html`

## 构建
使用 `cargo build` 构建测试版本的可执行文件，该文件运行时会默认启用 Debug 模式。

如果需要构建生产环境的版本，请执行 `pnpm rust:build`，这会调用 [vc-ltl](https://crates.io/crates/vc-ltl) 构建一个无需 VC 运行库的生产版本可执行文件。
