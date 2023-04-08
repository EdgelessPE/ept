# 添加内置变量或内置函数
内置变量或内置函数的名称应当符合大写驼峰（CamelCase）

## 内置变量
1. 编辑`values.rs`中的`define_values!`宏调用
2. 在`fn test_collect_values`添加单元测试

## 内置函数
注意：目前的设计仅支持添加输入 String 返回 bool 的简单函数，如果需要使用复杂函数应考虑重构

1. 编辑`functions.rs`中：
    * `fn capture_function_info`的`info_arr`变量，提供函数定义信息
    * `fn functions_decorator`函数体，提供函数执行器闭包
    * `fn generalize_permissions`的`match`语句，提供权限信息
    * （可选）`fn verify_self`的`match`语句，提供自定义参数校验
2. 在`fn test_header_perm`和`fn test_header_valid`添加单元测试，测试权限匹配和校验规则是否正确
3. 在`mod.rs`的`fn test_workflow_executor`添加单元测试，测试执行器运行是否正确