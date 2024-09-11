import { type FileInfo } from "./type";
import fs from "fs";
import { parseFilePath } from "../utils";

function clearMatchLine(line: string) {
  const sp = line
    .slice(13, -1)
    .split(",")
    .map((t) => t.trim());
  const filedName = sp[0].slice(1, -1);
  const values = sp[2].split("|").map((t) => t.trim().slice(1, -1));

  return {
    filedName,
    values,
  };
}

// 读取一个文件中所有的枚举定义
export function parseEnumDefinitions({ file }: FileInfo) {
  const text = fs.readFileSync(parseFilePath(file)).toString();
  const m = text.match(
    /verify_enum!\("(\w+)",\s?([\w.&]+),\s?"(\S+)"(\s?\|\s?"(\S+)")*\)/g,
  );

  const map: Record<string, string[]> = {};
  for (const { filedName, values } of m?.map(clearMatchLine) ?? []) {
    map[filedName] = values;
  }
  return map;
}
