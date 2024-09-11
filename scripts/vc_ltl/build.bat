@setlocal
@echo off

rem Find the path of the Microsoft Visual C++ 2017 or later
set VisualStudioInstallerFolder="%ProgramFiles(x86)%\Microsoft Visual Studio\Installer"
if %PROCESSOR_ARCHITECTURE%==x86 set VisualStudioInstallerFolder="%ProgramFiles%\Microsoft Visual Studio\Installer"
pushd %VisualStudioInstallerFolder%
for /f "usebackq tokens=*" %%i in (`vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
  set VisualStudioInstallDir=%%i
)
popd

rem Load the Microsoft Visual C++ 2017 build environment
call "%VisualStudioInstallDir%\VC\Auxiliary\Build\vcvarsall.bat" x86_amd64

rem Load the VC-LTL configuration
call "%~dp0\VC-LTL helper for nmake.cmd"

rem We use static CRT because we don't want to depend on VC-LTL CRT redistribute
set RUSTFLAGS=--codegen target-feature=+crt-static

rem Build Rust project via cargo
cargo build --release

@endlocal