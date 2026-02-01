# AGENTS_zh.md - EdgelessPE/ept 项目编码规范

这是 Edgeless Package Tool (ept) 的 Rust CLI 项目编码指南。

## 构建/检查/测试命令

```bash
# 构建项目
cargo build
cargo build --release

# 运行所有测试
cargo test -- --test-threads 1

# 运行特定测试（使用 --bin ept 指定二进制测试）
cargo test --bin ept -- <test_name>
cargo test --bin ept -- types::author::test_author_eq

# 运行特定模块的测试
cargo test --bin ept -- --test-threads 1 types::
cargo test --bin ept -- --test-threads 1 utils::

# 代码格式化和 Lint
pnpm rs:lint

# 检查编译而不构建
cargo check
```

## 代码风格规范

### 导入排序规则
1. 外部 crate 导入（如 `use anyhow::Result;`）
2. 标准库导入（如 `use std::path::PathBuf;`）
3. 内部模块导入使用 `crate::`（如 `use crate::types::package::Package;`）
4. 父模块导入（如 `use super::TStep;`）

### 命名规范
- **结构体/枚举**: PascalCase（如 `Package`, `StepCopy`, `ActionConfig`）
- **函数/方法**: snake_case（如 `verify_self()`, `parse_inputs()`）
- **常量**: UPPER_SNAKE_CASE（如 `FILE_NAME`）
- **模块**: snake_case（如 `package.rs`, `extended_semver.rs`）
- **泛型参数**: 单个大写字母（如 `T`, `F`）

### 函数参数规范
- **配置参数位置**: 如果函数需要 `cfg: &Cfg` 入参，则该入参始终位于**第一位**（类似 `self` 的约定）
  ```rust
  // ✅ 正确
  pub fn install(cfg: &Cfg, source: &str, verify: bool) -> Result<()>
  pub fn workflow_executor(cfg: Cfg, flow: Vec<WorkflowNode>, located: String, pkg: GlobalPackage)
  
  // ❌ 错误
  pub fn install(source: &str, verify: bool, cfg: &Cfg) -> Result<()>
  ```
- **配置来源**: 如果函数中需要使用 `cfg`，则必须从祖先处获取（通过参数传递），而不是调用 `Cfg::default()`
  ```rust
  // ✅ 正确 - 从祖先处获取
  pub fn some_function(cfg: &Cfg, ...) -> Result<()> {
      let path = get_path_mirror(cfg)?;
      ...
  }
  
  // ❌ 错误 - 在函数内部创建默认配置
  pub fn some_function(...) -> Result<()> {
      let cfg = Cfg::default();
      let path = get_path_mirror(&cfg)?;
      ...
  }
  ```

### 类型与 Trait
- 拥有的字符串使用 `String`，函数参数中的字符串切片使用 `&str`
- 优先使用 `Option<T>` 而非可空值
- 使用 `Result<T, E>` 处理可能失败的操作（E 通常为 `anyhow::Error`）
- 为数据结构派生常用 trait：`#[derive(Debug, Clone, PartialEq)]`
- 为配置/数据类型使用 `#[derive(Serialize, Deserialize)]`

### 错误处理
- 大多数操作使用 `anyhow::Result<T>`
- 创建描述性错误消息：`anyhow!("Error(Context):Failed to {action} : {e}")`
- 使用 `map_err()` 包装错误并添加上下文
- 使用 `?` 操作符进行错误传播和提前返回

### 注释与文档
- 使用 `///` 编写公共 API 文档
- 使用 `//` 编写行内实现注释
- **所有注释和文档必须使用简体中文**
- 字段注释格式：`//# example value` 或 `// description`
- 领域特定文档可以使用中文

### 测试
- 测试写在源文件内，使用 `#[test]` 属性
- 集成风格测试使用 `test_<module>_corelation` 命名
- 通过 `pub fn _demo()` 方法提供演示/测试数据
- 测试工具函数放在 `src/utils/test.rs`
- **在单测函数中创建 `Cfg` 结构体时，必须使用 `crate::utils::test::_default_test_cfg()`，而不是 `Cfg::default()`**
  ```rust
  // ✅ 正确
  let cfg = crate::utils::test::_default_test_cfg();
  
  // ❌ 错误
  let cfg = Cfg::default();
  ```

### 宏
- 使用 `p2s!()` 宏进行路径到字符串的转换
- 使用 `log!()` 宏进行格式化日志输出（如果需要打印日志，请使用 `log!` 宏而不是 `println!` 或其他宏）
- 使用 `verify_enum!()` 进行枚举值验证

### Windows 特定代码
- 项目目标平台为 Windows
- 使用 `#[cfg(windows)]` 编写 Windows 特定功能
- 使用 `std::path::Path` 进行跨平台路径处理

### Serde 序列化
- 使用 `#[serde(rename = "...")]` 映射字段名
- 使用 `#[serde(default)]` 为可选字段设置默认值
- TOML 使用 `features = ["preserve_order"]` 保持顺序

## 项目结构

```
src/
├── main.rs              # CLI 入口点和命令路由
├── types/               # 核心数据类型和结构
│   ├── package.rs       # 包元数据结构
│   ├── software.rs      # 软件特定元数据
│   ├── workflow.rs      # 工作流定义
│   ├── steps/           # 工作流步骤实现
│   └── ...
├── utils/               # 工具函数和辅助函数
│   ├── fs.rs            # 文件系统操作
│   ├── path.rs          # 路径处理工具
│   └── ...
├── entrances/           # 命令实现
│   ├── install.rs       # 安装命令
│   ├── uninstall.rs     # 卸载命令
│   └── ...
├── executor/            # 工作流执行引擎
├── parsers/             # 配置解析
├── signature/           # 加密签名
└── compression/         # 归档处理
```

## 常见模式

### 定义新步骤类型

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StepNewType {
    pub field: String,
    pub optional_field: Option<bool>,
}

impl TStep for StepNewType {
    fn run(self, cx: &mut WorkflowContext) -> Result<i32> {
        // 实现
        Ok(0)
    }
    fn reverse_run(self, _: &mut WorkflowContext) -> Result<()> {
        Ok(())
    }
    fn get_manifest(&self, fs: &mut MixedFS) -> Vec<String> {
        Vec::new()
    }
    fn verify_step(&self, ctx: &VerifyStepCtx) -> Result<()> {
        Ok(())
    }
}

impl Interpretable for StepNewType {
    fn interpret<F>(self, interpreter: F) -> Self
    where
        F: Fn(String) -> String,
    {
        Self {
            field: interpreter(self.field),
            optional_field: self.optional_field,
        }
    }
}

impl Generalizable for StepNewType {
    fn generalize_permissions(&self) -> Result<Vec<Permission>> {
        Ok(vec![Permission {
            key: PermissionKey::fs_write,
            level: PermissionLevel::Normal,
            targets: vec![self.field.clone()],
        }])
    }
}
```

### 错误消息格式

始终在错误前添加上下文：
- `Error(ModuleName):Description : {details}`
- `Warning(ModuleName):Description`
- `Info(ModuleName):Description`

示例：
```rust
anyhow!("Error(Copy):Failed to copy file from '{from}' to '{to}' : {e}")
anyhow!("Error(Execute):Command '{cmd}' execution failed : {err}")
```

### 路径处理

始终使用 `p2s!()` 宏进行路径到字符串的转换：
```rust
let path_str = p2s!(path);
```

使用 `format_path()` 规范化路径（将反斜杠替换为正斜杠）。

### 添加测试

测试应该内联在源文件末尾：
```rust
#[test]
fn test_module_feature() {
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Debug, true);
    // 测试实现
}

#[test]
fn test_module_corelation() {
    // 模块集成风格测试
}
```

## 外部依赖

本项目使用的关键 crate：
- `anyhow` - 错误处理
- `serde` + `toml` - 配置序列化
- `clap` - CLI 参数解析
- `reqwest` - HTTP 请求
- `tar` + `zstd` - 归档压缩
- `blake3` + `ed25519-compact` - 加密操作
- `winapi` + `winreg` - Windows 特定 API
- `tantivy` - 全文搜索
- `sysinfo` - 系统信息
