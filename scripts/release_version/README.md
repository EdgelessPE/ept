# 发版脚本
此目录下存放发布新版本所使用的脚本

## 准备证书
在当前目录下使用管理员身份执行 PowerShell 脚本 `./scripts/gen-cert.ps1`，脚本会生成一个自签名的证书 `cert.pfx`，密码 `114514`。

## 发版流程
在项目根目录执行 `pnpm rs:release --type patch` 即可发布一个 patch 位更新的新版本（可以将 `patch` 替换为 `major` 或 `minor`），新版的 `ept.exe` 会存放在 `target/release` 目录下。

脚本执行结束后将 git tag 和文件更新推送到仓库，并将 `target/release/ept.exe` 上传到 GitHub Release 等位置即可完成一次发版。

## 干运行
在调试时可以追加参数 `--dry`，此时脚本不会将修改后的版本号写入文件且不会打 git tag。