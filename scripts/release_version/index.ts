import { readFile } from "node:fs/promises";
import TOML from "smol-toml";
import cp from "child_process";
import {
  getTargetVersion,
  modifyVersion,
  ask,
  sleep,
  genChangeLog,
} from "./utils";
import rcedit from "rcedit";
import { existsSync } from "node:fs";

async function main() {
  // 检查证书存在
  if (!existsSync("scripts/release_version/cert.pfx")) {
    throw new Error(
      "Certificate not found, please put 'cert.pfx' in 'scripts/release_version'",
    );
  }

  // 读取版本号，并判断 Rust 和 Node 版本号一致
  const packageText = (await readFile("package.json")).toString();
  const cargoText = (await readFile("Cargo.toml")).toString();

  const packageVersion = JSON.parse(packageText).version;
  const cargoVersion = (
    TOML.parse(cargoText) as { package: { version: string } }
  ).package.version;

  if (packageVersion !== cargoVersion) {
    throw new Error(
      `Version mismatch in 'package.json'(${packageVersion}) and 'Cargo.toml'(${cargoVersion})`,
    );
  }

  const targetVersion = await getTargetVersion(packageVersion);
  console.log(`Info: Target version : ${targetVersion}`);
  if (targetVersion === packageVersion) {
    throw new Error(`No commits find since last tag '${packageVersion}'`);
  }

  // 确认执行
  const res = await ask(
    `Info: Ready to bumping version from '${packageVersion}' to '${targetVersion}', press 'd' to run with develop mode without modifying file or adding git tag (y/d/n) : `,
  );
  if (res !== "y" && res !== "d") {
    throw new Error("Operation canceled by user");
  }
  const isDev = res === "d";
  const binPath = isDev ? "target/debug/ept.exe" : "target/release/ept.exe";

  // 编译 Rust 项目
  console.log("Info: Compiling...");
  cp.execSync(isDev ? "cargo build" : "npm run rs:build");
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
  console.log("Info: Signing...");
  cp.execSync(
    `signtool sign /f "scripts/release_version/cert.pfx" /p 114514 /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ${binPath}`,
  );

  // 生成 Changelog
  console.log("Info: Generating changelog...");
  await genChangeLog(targetVersion, isDev);

  // 修改版本号
  if (!isDev) {
    await modifyVersion("package.json", packageVersion, targetVersion);
    await modifyVersion("Cargo.toml", packageVersion, targetVersion);
  }

  // 提交 git 变更并打 tag
  if (!isDev) {
    console.log("Info: Committing and tagging...");
    cp.execSync(`git add -all`);
    cp.execSync(`git commit -m "release: ${targetVersion}"`);
    cp.execSync(`git tag ${targetVersion}`);
  }

  console.log(
    `Success: New executable file generated at '${binPath}', version ${targetVersion}`,
  );
}

main().then(() => process.exit(0));
