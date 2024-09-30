# 发版脚本
此目录下存放发布新版本所使用的脚本

## 准备证书
在当前目录下使用管理员身份执行 PowerShell 脚本 `./scripts/gen_cert.ps1`，脚本会生成一个自签名的证书 `cert.pfx`，密码 `114514`。

## 发版流程
在项目根目录执行 `pnpm rs:release` 即可发布一个新版本，新版的 `ept.exe` 会存放在 `target/release` 目录下。

脚本执行结束后将 git tag 和文件更新推送到仓库，并将 `target/release/ept.exe` 上传到 GitHub Release 等位置即可完成一次发版。

## 控制版本号步进
默认状态下脚本会自动根据上一次的 git tag 和历史的 git commit messages 自动步进版本号，追加参数 `--type patch` 或 `--type minor` 或 `--type major` 即可控制版本号步进。

## 跳过控制台交互
追加参数 `--dev`，等效于确认调试运行；追加参数 `--confirm`，等效于确认完全执行。
