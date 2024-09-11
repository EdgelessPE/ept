# Basic Workflow Introduction

Workflows are used to perform a series of specific "steps" to complete specific tasks, such as installation, updates, or uninstallation.

An example installation workflow might look like this:
```toml
[create_shortcut]
step = "Link"
source_file = "Code.exe"
target_name = "Visual Studio Code"

[add_path]
step = "Path"
record = "Code.exe"
operation = "Add"
```
Semantically, it can be roughly seen that this workflow performs the following two steps:
* Add a shortcut for `Code.exe`, with the shortcut name `Visual Studio Code`
* Add `Code.exe` to the environment variables

We will explain the composition of the workflow in detail below.

## Steps
A workflow is composed of several steps, for example:
```toml
[copy_config]
step = "Copy"
from = "./VSCode/_config"
to = "./Users/Config"
```
is an independent step, which is used to copy folders.

Each step is an independent toml table and must contain the `step` field to indicate what kind of operation the step needs to perform; the value of the `step` field must be one of the several step types defined in [Definitions and APIs](/nep/definition/4-steps).

Below the `step` field are the fields unique to the `Copy` step—`from` and `to`, indicating where to copy files from and where to copy them to. For different steps, you can view [Definitions and APIs](/nep/definition/4-steps) to understand their respective unique fields.

The key of the step usually uses [snake_case](https://en.wikipedia.org/wiki/Snake_case) to indicate the actual meaning of the operation the step wants to perform, for example, the example above indicates that this step wants to **copy the configuration folder**. Since keys cannot contain spaces, special characters, or Chinese characters, if you want to express the meaning of the operation more clearly, you can add a `name` field to the step:
```toml
[copy_config]
name = "Pre-set VSCode Configuration"
step = "Copy"
from = "./VSCode/_config"
to = "./Users/Config"
``` {/*steps*/}
