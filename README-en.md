<div align="center">
  <a href="https://ept.edgeless.top" target="_blank">
    <img alt="Nep Logo" width="200" src="logo/nep.png"/>
  </a>
</div>
<div align="center">
  <h1>ept</h1>
</div>

<div align="center">

(WIP) Next-generation Windows package management solution - built based on the Nep specification

[![codecov](https://codecov.io/github/EdgelessPE/ept/graph/badge.svg?token=KF7Z1SSF3Q)](https://codecov.io/github/EdgelessPE/ept)

</div>

## Features
* 🚀 Built with Rust, the compiled size is less than 20MB and does not require any runtime or dynamic library dependencies, offering excellent performance
* 🔒 Uses the BLAKE3 hashing algorithm, combined with the Ed25519 digital signature algorithm to provide a secure and extremely fast signing experience
* 📦 Utilizes the Zstandard compression algorithm, achieving efficient data compression and decompression capabilities
* 🛠️ Complete workflow design, elegantly describing the installation, update, and uninstallation processes of packages; supports automatic generation of reverse workflows, permission information, and packing lists from workflows
* 📝 Comprehensive metadata management capabilities, supporting tags, permission control, etc.; supports recognizing program self-updates, recognizing registry entries to obtain the main program path and uninstall commands; supports installation package and portable software packages, supports expandable software packages, supports custom package type preferences
* 🤖 Rich ecosystem, with a complete CI/CD process, using robots for automatic construction and ensuring package quality through an automated quality assurance system

## Unit Testing
* (Optional) Create an `eptrc.toml` file in the project root directory and specify `local.base` to isolate the test installation environment
* Install [miniserve](https://github.com/svenstaro/miniserve) with `scoop install miniserve` or `cargo install --locked miniserve`
* Execute `pnpm rs:ut`
* If you need to view the single test coverage, install [tarpaulin](https://github.com/xd009642/tarpaulin) with `cargo install cargo-tarpaulin` and then execute `pnpm rs:ut:html`

## Building
Use `cargo build` to build the test version of the executable file, which will default to Debug mode when running.

If you need to build a production version, execute `pnpm rs:build`, which will call [vc-ltl](https://crates.io/crates/vc-ltl) to build a production version executable file that does not require the VC runtime library.
