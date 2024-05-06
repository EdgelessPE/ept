# ept
[![codecov](https://codecov.io/github/EdgelessPE/ept/graph/badge.svg?token=KF7Z1SSF3Q)](https://codecov.io/github/EdgelessPE/ept)

新一代 Windows 包管理解决方案 - 基于 Nep 规范打造

## 单元测试
* （可选）在根目录中创建 `config.toml` 文件并指定 `local.base` 用于隔离测试安装环境
* 执行 `pnpm rust:ut`
* 如需要查看单测覆盖率，请安装 [tarpaulin](https://github.com/xd009642/tarpaulin) 后执行 `pnpm rust:ut:coverage`