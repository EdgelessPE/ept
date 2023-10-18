import { splitBlock } from "../block";
import { type FnValue, type PermissionLevel } from "./type";
import fs from "fs";
import path from "path";
import { parseFilePath } from "../utils";

function parsePermission(file: string): FnValue["permission"] {
  const text = fs.readFileSync(parseFilePath(file)).toString();
  const extract = (reg: RegExp) => {
    const m = text.match(reg);
    if (!m) {
      throw new Error(`Error:Failed to match reg '${reg}' in file '${file}'`);
    }
    return m[0].replace(reg, "$1");
  };

  const key = extract(/key: "(\w+)".to_string/);

  const level: FnValue["permission"]["level"] = text.includes(
    "judge_perm_level(&arg)",
  )
    ? "JUDGE_WITH_PATH"
    : (extract(/PermissionLevel::(\w+),/) as PermissionLevel);

  return { key, level };
}

export function parseInnerFn(_dir: string): FnValue[] {
  const dir = parseFilePath(_dir);
  const files = fs.readdirSync(dir);
  const results: FnValue[] = [];
  for (const fileName of files) {
    if (fileName === "mod.rs") continue;
    const filePath = path.join(dir, fileName);
    const fnName = fileName
      .split(".")[0]
      .split("_")
      .map((t) => {
        const ns = t[0].toUpperCase();
        return `${ns}${t.slice(1)}`;
      })
      .join("");

    const blocks = splitBlock({
      file: filePath,
      startsWith: `pub struct ${fnName}`,
    });
    if (blocks.length !== 1 || blocks[0].declaration !== "") {
      console.error("blocks: ", blocks);
      throw new Error(`Error:Failed to parse fn file '${fileName}'`);
    }
    const { wiki, demo, extra } = blocks[0];
    results.push({
      name: fnName,
      wiki,
      demo,
      permission: parsePermission(filePath),
      validationRules: extra,
    });
  }

  return results;
}
