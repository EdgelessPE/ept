# Nep 解决了什么问题
我们清楚目前已经有多款针对 Windows 平台的类包管理解决方案，但是这些方案都或多或少的存在一些缺陷。Nep 致力于解决以下问题：

> 引用：[Chocolatey](https://chocolatey.org/) [Scoop](https://scoop.sh/) [Winget](https://github.com/microsoft/winget-cli)

## 沉重的运行时负担
当前的多种方案离不开 .Net、PowerShell、Git、NuGet 等“庞然大物”的支持，从而无法有效应对轻量化的场景。

Nep 配套的 ept 包管理工具
## 离线不友好的装箱单方案
部分方案使用的装箱单无法有效应对离线场景的包管理需求。
## 不完善的解决方案
