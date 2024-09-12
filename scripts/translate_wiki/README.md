# 文档翻译同步脚本
此目录下存放使用大模型同步并翻译文档的脚本，以便从中文文档向其他语言的文档同步

## 准备
* 在当前目录下执行 `cp .chatgpt-md-translator.example .chatgpt-md-translator`
* 编辑 `.chatgpt-md-translator`中的 `OPENAI_API_KEY`，将其替换为你的 Kimi API Key；如果你希望使用其他的 OpenAI 兼容供应商请修改 `API_ENDPOINT` 字段

## 使用
* 编辑 Rust 代码中的文档注释，然后重新生成文档；或手动对非自动生成的中文文档进行编辑
* 执行命令 `pnpm doc:translate`
* 对翻译结果手动润色，包括手动润色和根据 git diff 挑选更佳的翻译结果
* 【已安装 git hook 时可选】执行一次 `pnpm doc:translate --check`，脚本会自动为润色后的英文文档生成重新 md5

## Git Hook
在项目根目录执行 `pnpm install` 时会自动安装 Git Hook。之后进行 git commit 操作时 lint-staged 会调用 `pnpm doc:translate --check` 对文档是否同步进行检查，如果没有同步国际化则无法通过提交；如果手动润色后没有更新英文文档的 md5，则会自动更新并添加 `store.json` 到 git 暂存区一起提交