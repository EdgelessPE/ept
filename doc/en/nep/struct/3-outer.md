# Outer-package
An Outer-package includes an Inner-package and some other information needed to be provided to ept for use, such as signature files, QA test results, etc.; archiving these contents using tar will yield an Outer-package.

The general directory structure of an Outer-package is as follows:
```
│  {PACKAGE_NAME}_{VERSION}_{FIRST_AUTHOR}.tar.zst          # Inner-package
│  signature.toml                                           # Signature file
```
## Signature File
The signature file is named `signature.toml`, which stores the author and signature information for the Inner-package and other files.

A signature file might look like this:
```toml
# Signature information for the in-package
[package]
# Signer
signer = '{SIGNER}'
# Digest
signature = '{SIGNATURE}'
```

You can find the complete definition of the signature file fields in [Definition and API](/nep/definition/1-package).
