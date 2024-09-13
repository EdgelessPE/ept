# Conditions
The `if` condition field allows you to control whether a step needs to be executed in the workflow based on the [context](./2-context.md).

For example, when used in conjunction with the built-in variable [ExitCode](/nep/definition/2-context.html#exitcode) (note that using this variable requires disabling strict mode in `package.toml`), you can decide whether to execute the current step based on the exit code of the previous step:
```toml
[call_installer]
step = "Execute"
command = "./installer.exe /S"

[warning_on_failure]
step = "Log"
if = 'ExitCode!=0' # Condition statement
msg = "Warning: Failed to execute installer!"
```
In this example, if the `call_installer` step fails, the value of `ExitCode` will be set to a non-zero value (let's assume it's set to 1), then when the `warning_on_failure` step is executed, the value of `ExitCode` will be 1, the condition statement evaluates to true, and thus the warning "Warning: Failed to execute installer!" will be printed.

It should be noted that the value of `ExitCode` is always the **result of the execution of the previous step**, so the third step (log_2) in the following example will not be executed:
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
# At this point, ExitCode's value is the exit code of step log_1, and log_1 was executed normally without errors
# Therefore, the value of ExitCode is 0, the condition statement evaluates to false, and this step will not be executed
if = "ExitCode==1"
msg = "You can't see this message, since 'ExitCode==0'"
```
