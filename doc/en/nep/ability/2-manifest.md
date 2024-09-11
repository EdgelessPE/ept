# 装箱单

Nep 支持通过总结安装工作流中的步骤以生成装箱单，在打包前对包内必须携带的内容进行检查。

例如对于如下示例步骤：

```toml
[add_path]
name = "Add PATH"
step = "Path"
record = "Code.exe"
operation = "Add"
```

会生成如下装箱单：

```
["Code.exe"]
```

假设包名为`VSCode`，则在打包时必须包含文件`VSCode/Code.exe`。

你可以在[定义与API](/nep/definition/4-steps)中找到完整的装箱单生成依据。