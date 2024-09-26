import { readFile } from "node:fs/promises";
import TOML from "smol-toml";
import cp from "child_process";
import { getBumpType, modifyVersion, needGitTag, sleep } from "./utils";
import { SemVer } from "semver";
import rcedit from "rcedit";

async function main() {
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

  // 解析版本号
  const instance = new SemVer(packageVersion);

  // 解析下一版本号
  const bumpType = getBumpType();
  instance.inc(bumpType);
  const targetVersion = instance.toString();

  console.log(
    `Info: Bumping version from '${packageVersion}' to '${targetVersion}'...`,
  );

  // 修改版本号
  await modifyVersion("package.json", packageVersion, targetVersion);
  await modifyVersion("Cargo.toml", packageVersion, targetVersion);

  // 打 git tag
  if (needGitTag()) {
    console.log("Info: Tagging...");
    cp.execSync(`git tag ${targetVersion}`);
  }

  // 编译 Rust 项目
  console.log("Info: Compiling...");
  cp.execSync(`npm run rs:build`);

  // 修改编译产物的版本号
  console.log("Info: Modifying release version...");
  await sleep(1000);
  await rcedit("target/release/ept.exe", {
    "product-version": targetVersion,
    "file-version": targetVersion,
    "version-string": {
      FileDescription: "Edgeless Package Tool",
      ProductName: "ept",
      LegalCopyright: `Copyright (c) ${new Date().getFullYear()} Cno. MIT Licensed project of EdgelessPE`,
    },
  });
}

main().then();
