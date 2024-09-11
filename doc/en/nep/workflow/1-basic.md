# 基础工作流介绍

工作流用于执行一系列特定的“步骤”来完成特定的工作，例如安装、更新或卸载工作。

一个示例的安装工作流可能是这样的：
```toml
[create_shortcut]
step = "Link"
source_file = "Code.exe"
target_name = "Visual Studio Code"

[add_path]
step = "Path"
record = "Code.exe"
operation = "Add"
```
通过语义可以大致看出，这个工作流分别执行了以下两个步骤：
* 为`Code.exe`添加快捷方式，快捷方式名为`Visual Studio Code`
* 将`Code.exe`添加到环境变量

我们会在下面详细解释工作流的构成。

## 步骤
工作流由数个步骤构成，例如：
```toml
[copy_config]
step = "Copy"
from = "./VSCode/_config"
to = "./Users/Config"
```
就是一个独立的步骤，这个步骤用于复制文件夹。

每个步骤都是一个独立的 toml 表，且必须包含`step`字段用于标注该步骤需要执行怎样的操作；`step`字段的值必须是[定义与API](/nep/definition/4-steps)中定义的数个步骤类型之一。

在`step`字段的下方则是`Copy`步骤独有的字段——`from`和`to`，表示了从什么位置复制文件到什么位置。对于不同的步骤，你可以通过查看[定义与API](/nep/definition/4-steps)来了解他们各自的独有字段。

步骤的键通常使用 [snake_case](https://zh.wikipedia.org/wiki/%E8%9B%87%E5%BD%A2%E5%91%BD%E5%90%8D%E6%B3%95) 标识了该步骤希望进行操作的实际意义，例如上面的示例表明这个步骤希望**复制配置文件夹**。由于键不能包含空格、特殊字符或中文，因此如果想更清晰的表达操作意义可以为步骤添加`name`字段：
```toml
[copy_config]
name = "预设 VSCode 配置"
step = "Copy"
from = "./VSCode/_config"
to = "./Users/Config"
```
