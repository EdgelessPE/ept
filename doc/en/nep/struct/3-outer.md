# Outsourcing
Outsourcing includes the in-package and some other information that needs to be provided for ept to use, such as signature files, QA test results, etc.; these contents can be archived using tar to obtain the out-package.

The general directory structure of the out-package is as follows:
```
│  {PACKAGE_NAME}_{VERSION}_{FIRST_AUTHOR}.tar.zst          # In-package
│  signature.toml                                           # Signature file
```

## Signature File
The signature file is named `signature.toml`, which stores the author's and signature information for the in-package and other files.

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
