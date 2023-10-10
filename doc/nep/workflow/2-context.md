# 上下文
在一条工作流的执行过程中会产生执行上下文，通常包含内置变量、内置函数等信息。

## 内置变量
在字符串中，你可以在合适的位置使用`${}`语句来调用内置变量，例如`${ProgramFiles_X64}`表示`Program Files`目录的路径，因此你可以写出这样的工作流：
```toml
[move_shortcut]
step = "Move"
from = "./lib.dll"
to = "${ProgramFiles_X64}/Microsoft/VSCode/"
```
在执行该步骤时，假设系统盘盘符为`C:`，那么`lib.dll`就会被移动到`C:/Program Files/Microsoft/VSCode/lib.dll`。

在[条件](./3-conditions)语句中可以直接通过变量名称来调用内置变量，而不是使用`${}`包裹。因此下面三个条件语句是等效的：
```toml
# 写法1，使用转义符
if = "\"${Arch}\"==\"X64\""

# 写法2，使用单引号来避免为双引号添加转义符
if = '"${Arch}"=="X64"'

# 写法3，去掉双引号和花括号
if = 'Arch=="X64"'
```
显然写法 3 更加简洁清晰，这也是我们推荐的条件语句写法。

你可以在[定义与API](/nep/definition/2-context)中找到完整的内置变量定义。

## 内置函数
内置函数通常会配合[条件](./3-conditions)使用，例如：
```toml
[kill_code]
step = "Kill"
if = 'IsAlive("code.exe")'
target = "code.exe"
```
表示如果`code.exe`进程仍在运行，则杀死该进程。

:::info
需要注意，内置函数通常都是输入为单个字符串、输出为布尔型的简单函数；若需要使用复杂函数请考虑执行一个脚本
:::

你可以在[定义与API](/nep/definition/2-context)中找到完整的内置函数定义。