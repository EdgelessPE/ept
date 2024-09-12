# Context
During the execution of a workflow, an execution context is generated, which usually contains information such as built-in variables and built-in functions.

## Built-in Variables
In the [conditions](./3-conditions) statement field, you can call built-in variables by their names, for example:
```toml
if = 'Arch=="X64"'
```

In regular string fields (fields outside of [conditions](./3-conditions) statements are usually string fields), you can use `${}` statements to call built-in variables. This syntax is similar to [template literals](https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Template_literals) in JS. For example, `${ProgramFiles_X64}` represents the path of the `Program Files` directory, so you can write it like this:
```toml
[move_shortcut]
step = "Move"
from = "./lib.dll"
to = "${ProgramFiles_X64}/Microsoft/VSCode/"
```
When executing this step, assuming the system drive letter is `C:`, `lib.dll` will be moved to `C:/Program Files/Microsoft/VSCode/lib.dll`.

:::warning
If you want to use template string syntax in a conditional statement, make sure the expression is within a string (`""`):
```toml
# This syntax is useful when you need to concatenate strings
if = '"${SystemDrive}/User${ExitCode}"=="C:/User0"'

# This syntax is verbose when you don't need to concatenate strings
if = '"${Arch}"=="X64"'
```
:::

You can find the complete definition of built-in variables in [Definitions and APIs](/nep/definition/2-context).

## Built-in Functions
Built-in functions are usually used in conjunction with [conditions](./3-conditions), for example:
```toml
[kill_code]
step = "Kill"
if = 'IsAlive("code.exe")'
target = "code.exe"
```
This means if the `code.exe` process is still running, kill that process.

:::info
Note that built-in functions are typically simple functions with a single string input and a boolean output; if you need to use complex functions, consider executing a script.
:::

You can find the complete definition of built-in functions in [Definitions and APIs](/nep/definition/2-context).
