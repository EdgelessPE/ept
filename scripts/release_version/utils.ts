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
  return await new Promise((resolve) => {
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
      bump: "auto",
    },
    { stdio: undefined },
  );
  // console.log(res.stdout);
  const ver = res.stdout?.match(/## \[(\d+\.\d+\.\d+)\]/)?.[1];
  if (ver) {
    return ver;
  } else {
    throw new Error(`Failed to parse auto bump version from git cliff`);
  }
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

export async function genChangeLog() {
  await runGitCliff({
    output: "CHANGELOG.md",
  });
}
