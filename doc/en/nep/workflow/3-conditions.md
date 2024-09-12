# Conditions
The `if` condition field allows you to control whether a step needs to be executed in the workflow based on the [context](./2-context.md).

The most common scenario is to use it in conjunction with the built-in variable [ExitCode](/nep/definition/2-context.html#exitcode) to decide whether to execute the current step based on the exit code of the previous step:
```toml
[call_installer]
step = "Execute"
command = "./installer.exe /S"

[warning_on_failure]
step = "Log"
if = 'ExitCode!=0' # Condition statement
msg = "Warning: Failed to execute installer!"
```
In this example, if the `call_installer` step fails, the value of `ExitCode` will be set to a non-zero value (here it is assumed to be set to 1). Then, when the step `warning_on_failure` is executed, the value of `ExitCode` is 1, the result of the condition statement is true, and therefore the warning `Warning: Failed to execute installer!` is printed.

It should be noted that the value of `ExitCode` is always the **result of the execution of the previous step**. Therefore, the third step ( `log_2` ) in the following example will not be executed:
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
