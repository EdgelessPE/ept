# 添加步骤
步骤名称应当尽可能为单一动词并以大写开头，或是符合大写驼峰（CamelCase），为此申明的结构体名称为`Step + NAME`

此处假设新增步骤的名称为`StepCustom`

1. 新建文件`Custom.rs`，并在`mod.rs`中申明为模块
2. 申明结构体：
    ```rust
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct StepCustom {
        pub FIELD: TYPE,
        // ...
    }
    ```
3. 为`StepCustom`实现`TStep`特性
4. 编写单元测试
5. 在`mod.rs`的`def_enum_step!`宏调用中注册步骤`StepCustom`