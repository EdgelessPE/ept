# 外包
外包包含了内包和一些其他需要提供给 ept 使用的信息，例如签名文件、QA测试结果等；将这些内容使用 tar 归档即可获得外包。

外包内容的大致目录结构如下：
```
│  {PACKAGE_NAME}_{VERSION}_{FIRST_AUTHOR}.tar.zst          # 内包
│  signature.toml                                           # 签名文件
```
## 签名文件
签名文件的文件名为 `signature.toml`，存放对内包等文件的作者、签名信息。

一个签名文件可能如下：
```toml
# 内包的签名信息
[package]
# 签名人
signer = '{SIGNER}'
# 摘要
signature = '{SIGNATURE}'
```

你可以在[定义与API](/nep/definition/1-package)中找到完整的签名文件字段定义。