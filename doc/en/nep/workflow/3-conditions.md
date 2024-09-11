# 条件
通过条件字段`if`上的条件语句可以在工作流中根据[上下文](./2-context.md)控制某条步骤是否需要被执行。

最常见的情景是配合内置变量 [ExitCode](/nep/definition/2-context.html#exitcode) 使用，根据上一条步骤的退出码决定是否执行当前步骤：
```toml
[call_installer]
step = "Execute"
command = "./installer.exe /S"

[warning_on_failure]
step = "Log"
if = 'ExitCode!=0' # 条件语句
msg = "Warning: Failed to execute installer!"
```
在这个示例中，如果`call_installer`步骤执行失败则会将`ExitCode`的值配置为非 0 的一个值（这里假设配置为 1），那么当执行到步骤`warning_on_failure`时`ExitCode`的值就为 1，条件语句的执行结果为真，因此会打印警告`Warning: Failed to execute installer!`。

需要注意`ExitCode`的值始终为**上一条步骤的执行结果**，因此下面这个示例中的第三个步骤（`log_2`）不会被执行：
```toml
[throw_error]
step = "Execute"
command = "exit 1"

[log_1]
step = "Log"
if = "ExitCode==1"
msg = "You can see this message, since 'ExitCode==1'"

[log_2]
step = "Log"
# 此时 ExitCode 的值为步骤 log_1 的退出码，而 log_1 正常执行了没有报错
# 因此 ExitCode 的值为 0，条件语句执行结果为假，这条步骤不会被执行
if = "ExitCode==1"
msg = "You can't see this message, since 'ExitCode==0'"
```