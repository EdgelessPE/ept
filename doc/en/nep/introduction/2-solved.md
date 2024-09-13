# What Problems Does Nep Solve
We are aware that there are already several package management solutions for the Windows platform, but these solutions all have more or less some flaws. Nep has the following features to address some of the pain points of existing solutions:

> Reference: [Chocolatey](https://chocolatey.org/), [Scoop](https://scoop.sh/), [Winget](https://github.com/microsoft/winget-cli)

## Lighter Runtime Dependency Requirements
Current solutions cannot do without the support of "giants" such as .Net, PowerShell, Git, NuGet, and cannot effectively cope with lightweight scenarios.

The ept package management tool that comes with Nep is implemented in Rust, with a compiled single file size of less than 20MB (the size is close to 6MB after zip compression), and can run without any dependency libraries.

## More Comprehensive Package Solution
The manifest strategy used by some solutions is not a true "package" in the sense, so it cannot effectively meet the package management needs of offline scenarios.

Nep itself is a well-designed package specification, so it can be easily used in offline or private deployment scenarios. At the same time, we have also considered the advantages of the manifest strategy in accelerating static resource downloads, and will provide a package similar to the manifest form in the future to more reasonably use the static acceleration resources of upstream manufacturers to achieve faster package distribution.

## Faster Resource Link Strategy Design
In scenarios such as decompression, integrity verification, and signing, Nep has chosen a more modern solution that can complete resource management functions more quickly and securely.

Nep uses [Zstandard](https://github.com/facebook/zstd) + tar as the decompression solution for packages, which has higher IO throughput and higher compression ratio compared to other decompression strategies. It also has a certain "readability without specific tools" - you can use compression software such as Bandizip or 7-Zip-zstd to complete unpacking and viewing operations.

Nep uses [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) as the encryption hash algorithm for packages, which has several times the computational performance improvement and better security compared to other encryption algorithms.

Nep uses the [Ed25519](https://ed25519.cr.yp.to/) elliptic curve asymmetric encryption algorithm as the signature generation algorithm for packages, which has higher security and shorter key storage length compared to other asymmetric signature generation algorithms.
