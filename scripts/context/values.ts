import fs from "fs";
import { parseFilePath } from "../utils";
import { PermissionLevel } from "../type";

export function parseInnerValues(
  filePath: string
): { name: string; level: PermissionLevel }[] {
  const text = fs.readFileSync(parseFilePath(filePath)).toString();
  const m = text.match(/\{"\$\{(\w+)\}",\w+\(\),PermissionLevel::(\w+)\}/g);

  if (!m?.length) return [];
  return m.map((line) => {
    const m = line.match(/\{"\$\{(\w+)\}",\w+\(\),PermissionLevel::(\w+)\}/);
    if (m) {
      return {
        name: m[1],
        level: m[2] as PermissionLevel,
      };
    } else {
      throw new Error(`Error:Failed to parse value declaration line '${line}'`);
    }
  });
}
