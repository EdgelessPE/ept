# ept
[![codecov](https://codecov.io/github/EdgelessPE/ept/graph/badge.svg?token=KF7Z1SSF3Q)](https://codecov.io/github/EdgelessPE/ept)

（WIP）新一代 Windows 包管理解决方案 - 基于 Nep 规范打造

## 构建
在测试环境中可以使用 `cargo build` 构建测试版本的可执行文件，该文件运行时会默认启用 Debug 模式。

如果需要构建生产环境的版本，请执行 `pnpm rust:build`，这会调用 [vc-ltl](https://crates.io/crates/vc-ltl) 构建一个无需 VC 运行库的生产版本可执行文件。

## 单元测试
* （可选）在根目录中创建 `eptrc.toml` 文件并指定 `local.base` 用于隔离测试安装环境
* 使用 `scoop install miniserve` 或 `cargo install --locked miniserve` 安装 [miniserve](https://github.com/svenstaro/miniserve)
* 执行 `pnpm rust:ut`
* 如需要查看单测覆盖率，请使用 `cargo install cargo-tarpaulin` 安装 [tarpaulin](https://github.com/xd009642/tarpaulin) 后执行 `pnpm rust:ut:html`
