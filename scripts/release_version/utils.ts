import minimist from "minimist";
import { readFile, writeFile } from "node:fs/promises";

const BUMP_TYPE = ["major", "minor", "patch"] as const;
type BumpType = (typeof BUMP_TYPE)[number];
const args = minimist(process.argv.slice(2));

export function getBumpType(): BumpType {
  if (BUMP_TYPE.includes(args.type)) {
    return args.type;
  }
  throw new Error(
    args.type
      ? `Unknown bump type : ${args.type}`
      : "Bump type expected in arg '--type'",
  );
}

export function needGitTag() {
  return !!args.tag;
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
