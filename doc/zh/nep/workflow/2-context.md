# 上下文
在一条工作流的执行过程中会产生执行上下文，通常包含内置变量、内置函数等信息。

## 内置变量
在[条件](./3-conditions)语句字段中可以通过变量名称来调用内置变量，例如：
```toml
if = 'Arch=="X64"'
```

在常规的字符串字段中（[条件](./3-conditions)语句之外的字段通常都为字符串字段），你可以使用`${}`语句来调用内置变量，这种写法类似于 JS 中的[模板字符串](https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Template_literals)。例如`${ProgramFiles_X64}`表示`Program Files`目录的路径，因此你可以这样写：
```toml
[move_shortcut]
step = "Move"
from = "./lib.dll"
to = "${ProgramFiles_X64}/Microsoft/VSCode/"
```
在执行该步骤时，假设系统盘盘符为`C:`，那么`lib.dll`就会被移动到`C:/Program Files/Microsoft/VSCode/lib.dll`。

:::warning
如果想在条件语句中使用模板字符串写法，需要确保表达式位于字符串（`""`）内：
```toml
# 需要拼接字符串时这种写法比较有用
if = '"${SystemDrive}/User${ExitCode}"=="C:/User0"'

# 不需要拼接字符串时，这种写法就比较啰嗦
if = '"${Arch}"=="X64"'
```
:::

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
需要注意，内置函数通常都是输入为单个字符串、输出为布尔型的简单函数；若需要使用复杂函数请考虑使用[`Execute`](/nep/definition/4-steps/execute.html)步骤执行一个脚本
:::

你可以在[定义与API](/nep/definition/2-context)中找到完整的内置函数定义。