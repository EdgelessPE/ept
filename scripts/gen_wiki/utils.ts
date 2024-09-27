import { fileURLToPath } from "node:url";
import path from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export function parseFilePath(rawPath: string) {
  if (rawPath.startsWith("@/")) {
    rawPath = rawPath.replace("@/", path.join(__dirname, "../../src/"));
  }
  return rawPath;
}

const DECL_REGEX = /(\w+):\s?([\w<>()]+)/;

// 输入类型申明行，返回解析结果
export function parseTypeDeclaration(decl: string):
  | {
      identifier: string;
      optional: boolean;
      name: string;
    }
  | undefined {
  const m = decl.match(DECL_REGEX);
  if (!m) return undefined;
  const [, rawName, rawType] = m;
  const name = rawName.startsWith("c_") ? rawName.slice(2) : rawName;
  return rawType.startsWith("Option<") && rawType.endsWith(">")
    ? {
        identifier: rawType.slice(7, -1),
        optional: true,
        name,
      }
    : {
        identifier: rawType,
        optional: false,
        name,
      };
}
