# 内包
内包包含了主要文件内容，例如对于软件包来说内包包含了程序运行的所有必须文件及描述文件、工作流等上下文；将这些内容使用 tar 归档并使用 Zstandard 压缩算法压缩即可获得内包。

内包内容的大致目录结构如下：
```
│  package.toml           # 包描述文件
│
├─{PACKAGE_NAME}          # 包内容目录
│       ...
│
└─workflows               # 工作流目录
        setup.toml
        remove.toml
```
## 包描述文件
包描述文件的文件名为 `package.toml`，存放对包中内容的必要描述信息，例如名称、版本号、作者、简介等。

一个包描述文件可能如下：
```toml
# nep 规范版本号
nep = "x.x"

[package]
# 包名
name = "{PACKAGE_NAME}"
# 包模板
template = "Software"
# 版本号
version = "x.x.x.x"
# 打包者/作者
authors = ["{AUTHOR}"]

[software]
# 发行域
scope = "{SCOPE}"
# 软件分类
category = "{CATEGORY}"
```
### 包信息表
包信息表在 `package.toml` 中的 `[package]` 表内，提供 Nep 包的通用信息。

你可以在[定义与API](/nep/definition/1-package)中找到完整的包信息表字段定义。
### 独占表
对于不同的包模板有各自不同的独占表字段，提供属于该模板的包的独有信息。例如对于软件包（`template = "Software"`）来说，其独占表为 `package.toml` 中的 `[software]` 表。

你可以在[定义与API](/nep/definition/1-package)中找到完整的独占表字段定义。
## 包内容目录
包内容目录存放了使用者所需要的文件，其目录名称与包信息表中的包名（`package.name`）保持一致，其中存放该软件的主程序及其运行所需要的其他文件。
## 工作流目录
工作流目录的目录名称为 `workflows`，包含了符合 Nep 工作流规范的工作流描述文件。

对于软件包来说，合法的工作流文件名称为：
* `setup.toml`
* `update.toml`
* `remove.toml`
* `expand.toml`

其中 `setup.toml` 是必需提供的。

你可以在[基础工作流介绍](/nep/workflow/1-basic)中进一步了解工作流的相关概念。