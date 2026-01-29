# 添加内置变量或内置函数
内置变量或内置函数的名称应当符合大写驼峰（CamelCase）

## 内置变量
1. 编辑 `values.rs` 中的 `define_values!` 宏调用
2. 在 `fn test_collect_values` 添加单元测试

## 内置函数
注意：目前的设计仅支持添加输入 String 返回 bool 的简单函数，如果需要使用复杂函数应考虑重构

1. 在 `functions` 目录中新建文件（文件名应当符合 snake_case）并申明一个空结构体，然后为其实现 `EvalFunction` 特性
2. 在 `functions/mod.rs` 中导入并调用 `def_eval_functions!` 宏注册
3. 在 `src/types/workflow.rs` 的 `test_header_perm()` 和 `test_header_valid()` 中添加权限和入参校验单测案例
4. 在 `src/executor/mod.rs` 的 `test_condition_eval()` 中添加执行单测案例