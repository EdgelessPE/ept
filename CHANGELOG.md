# Changelog

<!-- INSERT_HERE -->

## [0.4.7] - 2026-03-25

### 🐛 Bug Fixes

- Spec target pkg


## [0.4.6] - 2026-03-25

### 🐛 Bug Fixes

- Package failed


## [0.4.5] - 2026-03-25

### 🐛 Bug Fixes

- 使用自定义工作流


## [0.4.4] - 2026-03-24

### 🐛 Bug Fixes

- Publish issue


## [0.4.3] - 2026-03-24


## [0.4.2] - 2026-03-24

### 🐛 Bug Fixes

- 重复发布问题


## [0.4.1] - 2026-03-24

### 🚀 Features

- 支持crates.io发版

### 🐛 Bug Fixes

- Ut

### 📚 Documentation

- 添加 README.md 跳转链接


## [0.4.0] - 2026-03-24

### 🚀 Features

- Info 支持以 toml 格式保存
- 初步支持传递解析临时目录
- 使用支持 zstd 的 Get 函数封装
- 直接删除 temp 目录，避免回收站过度膨胀
- 优化utils
- 全局变量解耦，支持实例化调用 (#455)

### 🐛 Bug Fixes

- 修复使用 url 更新时提示需要 offline
- 修复列出多个镜像时未换行
- *(doc)* 修复 Wait 步骤示例错误
- 修复 GitHub Actions 单测失败 (#415)
- Test imports
- Lint err
- Lib compile
- Tsc
- Ts

### 🚜 Refactor

- *(entrances)* Optimize code structure and centralize constants
- 简化测试代码中的字符串参数传递
- Rust Monorepo (#446)

### 📚 Documentation

- Update readme
- 增加中英文切换
- 使用 main 分支
- 使用 develop 分支
- 跟进英文文档
- 润色英文 README


## [0.3.1] - 2025-03-11

### 🚀 Features

- 默认关闭 Windows Terminal 状态控制
- 配置初始化支持覆盖检查
- 安装过的包在 install 中会被跳过
- 支持关闭 emoji 显示
- 优化安装、更新的解析逻辑，支持指定包时的前置检查
- 支持自动清理缓存
- Info 支持 URL 和本地路径
- Info 打印支持格式化打印
- 支持在配置中定义 offline
- 使用 Flag 控制缓存启用

### 🐛 Bug Fixes

- 删除多余空格
- 修复无法卸载不完整的包
- 修复无法从 url 读取 Info 的问题
- 修复缓存无法被禁用的问题

### 📚 Documentation

- 添加对 arch x86 架构的警告


## [0.3.0] - 2025-03-09

### 🚀 Features

- 支持 Windows Terminal 状态同步
- 支持简介作为搜索存储
- 支持 tags 搜索
- 支持二进制搜索
- 支持创建和读取 quick map
- 初步使用快查索引定位在线包
- 快查索引添加url模板字段
- 改造需要读pkg-software的部分为读快查索引
- 别名改为字符串类型
- 添加别名不能和名称重复的校验
- 别名支持被搜索和直接使用
- 将 scope 字段作为 package 表下面的通用字段
- 使用结构体而不是元组处理索引的 schema
- 初步支持现代化 Info 打印
- 初步支持版本号提示
- Info 函数支持替换目标
- 支持打印来源行
- 支持打印摘要
- Meta 支持查看 URL
- Info 支持输入 PackageInputEnum
- 安装、更新前支持获取并打印 Info
- 优化无法检查更新的打印
- 检查更新失败不会导致退出异常
- 搜索结果支持新版打印规范
- List 支持新版打印规范、List 时不会打印权限简报
- 替换镜像打印
- 改造 uninstall 为新版打印规范
- 安装前打印详细元信息
- 更新支持打印详细元信息
- 打印 mirror 时支持展示 root url
- 增加对镜像站协议版本号的校验
- Meta 支持直接查看镜像源中的 Meta 信息
- 增加 PackageInputEnum 的解析 Debug 输出
- 增加 Meta 单测
- Meta 提供的 package 不应该被解释
- 如果安装的包已经被安装则会询问用户是否走更新流程
- Info 支持分开查看目标、本地、在线的包信息
- 支持 Flag A

### 🐛 Bug Fixes

- 别名和包名应该判断小写重复
- 修复单侧失败
- 补齐退出码处理逻辑
- 修复没有来源时格式出错
- 修复 Debug 日志开头在 Windows Terminal 中看不见
- 修复单测问题
- 查询镜像 Meta 时启用分数排序
- Clippy
- 修复两处 Info 迁移错误
- 修复安装/更新本地包时版本展示不准确


## [0.2.3] - 2024-10-27

### 🐛 Bug Fixes

- 修复二进制安装出错导致npm install失败
- 修复空releases导致的索引构建报错
- 修复可拓展包无法校验不存在的主程序

### 🚜 Refactor

- 重构 Verifiable 特性，使用 MixedFS 替换 located

### 📚 Documentation

- Fix typo


## [0.2.2] - 2024-09-27

### 🚀 Features

- 实现基本功能