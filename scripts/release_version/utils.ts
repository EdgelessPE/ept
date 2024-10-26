import readline from "node:readline";
import minimist from "minimist";
import { readFile, writeFile } from "node:fs/promises";
import { runGitCliff } from "git-cliff";
import { SemVer } from "semver";

const BUMP_TYPE = ["major", "minor", "patch"] as const;
const args = minimist(process.argv.slice(2));

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export async function ask(prompt: string): Promise<string> {
  if (args.dev) {
    return "d";
  }
  if (args.confirm) {
    return "y";
  }
  return new Promise((resolve) => {
    rl.question(prompt, (answer) => {
      resolve(answer.toLowerCase());
    });
  });
}

export async function getTargetVersion(curVersion: string): Promise<string> {
  // 手动指定步进版本号
  if (BUMP_TYPE.includes(args.type)) {
    const instance = new SemVer(curVersion);
    instance.inc(args.type);
    return instance.toString();
  }

  // 自动判断版本号
  const res = await runGitCliff(
    {
      bumpedVersion: true,
    },
    { stdio: undefined },
  );
  return res.stdout.trim();
}

export async function modifyVersion(
  file: "package.json" | "Cargo.toml",
  fromVersion: string,
  toVersion: string,
) {
  const text = (await readFile(file)).toString();
  const fromLine =
    file === "package.json"
      ? `"version": "${fromVersion}",`
      : `version = "${fromVersion}"`;
  const toLine =
    file === "package.json"
      ? `"version": "${toVersion}",`
      : `version = "${toVersion}"`;
  if (!text.includes(fromLine)) {
    throw new Error(`Fatal: Version line '${fromLine}' not found in ${file}`);
  }
  const nextText = text.replace(fromLine, toLine);
  await writeFile(file, nextText);
}

export async function sleep(ms: number) {
  return new Promise((res) => setTimeout(res, ms));
}

const INSERT_TAG = "<!-- INSERT_HERE -->";
export async function genChangeLog(targetVersion: string, isDev: boolean) {
  // 生成新版本的变更日志
  const { stdout } = await runGitCliff(
    {
      tag: targetVersion,
      unreleased: true,
      strip: "all",
    },
    {
      stdio: undefined,
    },
  );
  console.log(stdout);

  // 将其插入到 CHANGELOG 的对应位置
  if (stdout.trim()) {
    const text = (await readFile("CHANGELOG.md")).toString();
    const nextText = text.replace(INSERT_TAG, `${INSERT_TAG}\n\n${stdout}`);
    await writeFile("CHANGELOG.md", nextText);
  } else {
    if (isDev) console.log("Warning: No change log generated");
    else throw new Error("Error: No change log generated");
  }
}
