# 反向工作流

为了简化打包者编写工作流的重复劳动量，在编写安装工作流（`setup.toml`）时会自动生成对应的反向工作流。例如在安装工作流中包含了这个创建快捷方式的步骤：
```toml
[create_shortcut]
step = "Link"
source_file = "./code.exe"
```

那么即使打包者未提供卸载工作流（`remove.toml`），在卸载时反向工作流也会自动生成一个将这个快捷方式删除的步骤。

你可以在[定义与API](/nep/definition/4-steps/0-general)中找到完整的反向工作流定义。