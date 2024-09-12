# Reverse Workflow

To simplify the repetitive labor of packaging authors in writing workflows, a corresponding reverse workflow is automatically generated when writing the installation workflow (`setup.toml`). For example, in the installation workflow, this step of creating a shortcut is included:
```toml
[create_shortcut]
step = "Link"
source_file = "./code.exe"
```

So even if the packager does not provide an uninstallation workflow (`remove.toml`), the reverse workflow will automatically generate a step to delete this shortcut during uninstallation.

You can find the complete reverse workflow definition in [Definitions and APIs](/nep/definition/4-steps).
