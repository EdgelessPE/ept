import {
  ask,
  genChangeLog,
  getCurrentVersion,
  getTargetVersion,
  modifyVersion,
} from "./utils";
import cp from "child_process";

async function bump_version() {
  const packageVersion = await getCurrentVersion();
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

  // 生成 Changelog
  console.log("Info: Generating changelog...");
  await genChangeLog(targetVersion, isDev);

  // 修改版本号
  if (!isDev) {
    await modifyVersion("package.json", packageVersion, targetVersion);
    await modifyVersion("Cargo.toml", packageVersion, targetVersion);
    cp.execSync(`cargo update -w`);
  }

  // 提交 git 变更并打 tag
  if (!isDev) {
    console.log("Info: Committing and tagging...");
    cp.execSync(`git add --all`);
    cp.execSync(`git commit -m "release: ${targetVersion}"`);
    cp.execSync(`git tag v${targetVersion}`);
  }

  console.log(`Success: Bumped version to '${targetVersion}'`);
}

bump_version().then(() => process.exit(0));
