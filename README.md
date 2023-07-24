# ept
## 单元测试
* （可选）在根目录中创建 `config.toml` 文件并指定 `local.base` 用于隔离测试安装环境
* 执行 `cargo test -- --test-threads 1`
* 如需要查看单测覆盖率，请安装 [tarpaulin](https://github.com/xd009642/tarpaulin) 后执行 `cargo tarpaulin --exclude-files test/**`