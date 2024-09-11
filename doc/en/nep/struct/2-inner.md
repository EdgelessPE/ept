# Inclusion
Inclusion contains the main file content, such as for a software package, the inclusion contains all the necessary files for program execution as well as description files, workflow, and other context; archiving these contents using tar and compressing with the Zstandard compression algorithm will yield the inclusion.

The general directory structure of the inclusion content is as follows:
```
│  package.toml           # Package description file
│
├─{PACKAGE_NAME}          # Package content directory
│       ...
│
└─workflows               # Workflow directory
        setup.toml
        remove.toml
```
## Package Description File
The package description file is named `package.toml`, storing necessary descriptive information about the contents of the package, such as name, version number, author, summary, etc.

A package description file might look like this:
```toml
# nep version number
nep = "x.x"

[package]
# Package name
name = "{PACKAGE_NAME}"
# Package template
template = "Software"
# Version number
version = "x.x.x.x"
# Packager/Author
authors = ["{AUTHOR}"]

[software]
# Distribution scope
scope = "{SCOPE}"
# Software category
category = "{CATEGORY}"
```
### Package Information Table
The package information table is within the `[package]` table in `package.toml`, providing general information for Nep packages.

You can find the complete package information table field definition in [Definition and API](/nep/definition/1-package).
### Exclusive Table
Different package templates have their own exclusive table fields, providing unique information for packages belonging to that template. For example, for software packages (`template = "Software"`), their exclusive table is the `[software]` table in `package.toml`.

You can find the complete exclusive table field definition in [Definition and API](/nep/definition/1-package).
## Package Content Directory
The package content directory stores the files needed by users, and its directory name should be determined according to the situation.

For software packages, the name of the package content directory is consistent with the package name in the package information table (`package.name`). For example, if a package has a filename of `VSCode_1.0.0.0_Microsoft`, then its package name is `VSCode`, and its package content directory is also `VSCode`, which contains the main program of VSCode and other files needed for its operation.
## Workflow Directory
The workflow directory is named `workflows`, containing workflow description files that comply with the Nep workflow specification.

For software packages, the valid workflow file names are:
* `setup.toml`
* `update.toml`
* `remove.toml`
* `expand.toml`

Among them, `setup.toml` must be provided.

You can further understand the concepts related to workflows in [Basic Workflow Introduction](/nep/workflow/1-basic).
