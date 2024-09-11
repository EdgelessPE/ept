# 权限控制

Nep 提供了对包的权限概括和控制能力，并支持对不同敏感度的权限进行分级。

例如对于如下示例步骤：

```toml
[copy_dll]
step = "Copy"
from = "./lib"
to = "${ProgramFiles_X86}/Microsoft/32.dll"
```

会生成如下原始权限信息：

```
Permission {
    key: "fs_write",
    level: Sensitive,
    targets: [
        "${ProgramFiles_X86}/Microsoft/32.dll",
    ],
}
```
上面的原始数据可以被翻译为：需要一个写文件系统的敏感权限，写入目标为`${ProgramFiles_X86}/Microsoft/32.dll`。

你可以在[定义与API](/nep/definition/3-permissions)中找到完整的权限定义。