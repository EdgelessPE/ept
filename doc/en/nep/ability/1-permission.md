# Permission Control

Nep provides the ability to summarize and control permissions for packages, and supports grading permissions of different sensitivities.

For example, for the following example steps:

```toml
[copy_dll]
step = "Copy"
from = "./lib"
to = "${ProgramFiles_X86}/Microsoft/32.dll"
```

The following raw permission information will be generated:

```
Permission {
    key: "fs_write",
    level: Sensitive,
    targets: [
        "${ProgramFiles_X86}/Microsoft/32.dll",
    ],
}
```
The above raw data can be translated as: It requires a sensitive permission to write to the file system, with the target being `${ProgramFiles_X86}/Microsoft/32.dll`.

You can find the complete permission definition in [Definition and API](/nep/definition/3-permissions).
