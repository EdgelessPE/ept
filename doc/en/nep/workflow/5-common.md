# Common Workflows
## Installation, Update, Uninstallation
In the Nep specification, there are three common workflows:
* Installation workflow (`setup.toml`)
* Update workflow (`update.toml`)
* Uninstallation workflow (`remove.toml`)

When installing a package, ept will invoke the installation workflow of this package;

When uninstalling, ept will invoke the uninstallation workflow of this package and the corresponding [reverse workflow](./4-reserve.md) of the installation workflow;

The situation during update is slightly more complex:
* If the first author of the two packages before and after the update is the same:
  * If the old package provides an uninstallation workflow, then execute the uninstallation workflow of the old package
  * If the new package provides an update workflow, then execute the update workflow of the new package
  * If the new package does not provide an update workflow and the uninstallation workflow of the old package has been executed, then execute the installation workflow of the new package
* If the first author of the two packages before and after the update is different:
  * First, execute the uninstallation workflow and the corresponding reverse workflow of the installation workflow of the old package, then execute the installation workflow of the new package

## Expandable
In expandable packages, ept will invoke the expandable workflow (`expand.toml`) before installation or update.
