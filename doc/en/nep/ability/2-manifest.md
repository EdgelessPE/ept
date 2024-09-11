# Bill of Materials

Nep supports generating a bill of materials by summarizing the steps in the installation workflow, checking the content that must be carried in the package before packaging.

For example, for the following example steps:

```toml
[add_path]
name = "Add PATH"
step = "Path"
record = "Code.exe"
operation = "Add"
```

The following bill of materials will be generated:

```
["Code.exe"]
```

Assuming the package name is `VSCode`, the file `VSCode/Code.exe` must be included when packaging.

You can find the complete basis for generating the bill of materials in [Definition and API](/nep/definition/4-steps). {/*examples*/}
