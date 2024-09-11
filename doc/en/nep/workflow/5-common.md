# Common Workflows
## Installation, Update, Uninstallation
In the Nep specification, there are three common workflows:
* Installation workflow (`setup.toml`)
* Update workflow (`update.toml`)
* Uninstallation workflow (`remove.toml`)

When installing a package, ept will call this package's installation workflow;

When uninstalling, ept will call this package's uninstallation workflow and the corresponding [reverse workflow](./4-reserve.md) of the installation workflow;

The situation during update is slightly more complex:
* If the first author of the two packages before and after the update is the same:
  * If the old package provides an uninstallation workflow, then execute the old package's uninstallation workflow
  * If the new package provides an update workflow, then execute the new package's update workflow
  * If the new package does not provide an update workflow and the old package's uninstallation workflow has been executed, then execute the new package's installation workflow
* If the first author of the two packages before and after the update is different:
  * First execute the old package's uninstallation workflow and the reverse workflow corresponding to the installation workflow, then execute the new package's installation workflow

## Expansion
In expandable packages, ept will call the expansion workflow before installation or update.
