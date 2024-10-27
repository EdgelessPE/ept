import cp from "child_process";
import { getCurrentVersion, sleep } from "./utils";
import rcedit from "rcedit";
import { existsSync } from "node:fs";
import core from "@actions/core";

const IS_USE_CERT = false;

async function main() {
  // 检查证书存在
  if (IS_USE_CERT && !existsSync("scripts/release_version/cert.pfx")) {
    throw new Error(
      "Certificate not found, please put 'cert.pfx' in 'scripts/release_version'",
    );
  }

  const targetVersion = await getCurrentVersion();
  core.setOutput("version", targetVersion);

  console.log(`Info: Target version : ${targetVersion}`);
  const binPath = "target/release/ept.exe";

  // 编译 Rust 项目
  console.log("Info: Compiling...");
  cp.execSync("npm run rs:build");
  await sleep(1000);

  // 修改编译产物的版本号
  console.log("Info: Modifying release version...");
  await rcedit(binPath, {
    "product-version": targetVersion,
    "file-version": targetVersion,
    "version-string": {
      FileDescription: "Edgeless Package Tool",
      ProductName: "ept",
      LegalCopyright: `Copyright (c) ${new Date().getFullYear()} Cno. MIT Licensed project of EdgelessPE`,
    },
  });

  // 签名
  if (IS_USE_CERT) {
    console.log("Info: Signing...");
    cp.execSync(
      `signtool sign /f "scripts/release_version/cert.pfx" /p 114514 /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ${binPath}`,
    );
  }

  console.log(
    `Success: New executable file generated at '${binPath}', version ${targetVersion}`,
  );
}

main().then(() => process.exit(0));
