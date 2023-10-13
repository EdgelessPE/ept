import fs from "fs";
import path from "path";
import { Result, Ok, Err } from "ts-results";
import { FieldInfo } from "./type";

interface FileInfo {
  file: string;
  structName: string;
}

// 读取 Rust 中的某个 struct，分析出所有字段信息
export function parseStruct({
  file,
  structName,
}: FileInfo): Result<FieldInfo[], string> {
  // 解析路径
  if (file.startsWith("@/")) {
    file = file.replace("@/", path.join(__dirname, "../src/"));
  }
  // 打开文件
  if (!fs.existsSync(file)) {
    return new Err(`Error:Failed to open file '${file}'`);
  }
  const text = fs.readFileSync(file).toString();

  // 匹配结构体
  const m = text.match(new RegExp(`pub struct ${structName} {[^}]+}`, "gm"));
  if (!m.length) {
    return new Err(
      `Error:Failed to find struct '${structName}' in file '${file}'`
    );
  }

  // 清理数据并按行分割
  const clearMatches = m[0]
    .split("\n")
    .slice(1, -1)
    .map((line) => {
      let r = line.trim();
      if (r.startsWith("pub ")) {
        r = r.slice(4);
      }
      return r;
    });

  // 解析
  const result:FieldInfo[]=[]
  let stack: string[] = [];
  for (const line of clearMatches) {
    // 将 wiki 注释推入栈
    if (line.startsWith("/// ")) {
      stack.push(line.slice(4));
    }

    // 忽略普通或其他特殊注释
    if (line.startsWith("//")) continue;

    // 解析字段名和类型
    const m = line.match(/(\w+):\s?([\w<>()]+)/);
    if (m) {
      const [, name, rawType] = m;
      const type: FieldInfo["type"] =
        rawType.startsWith("Option<") && rawType.endsWith(">")
          ? {
              identifier: rawType.slice(7, -1),
              optional: true,
            }
          : {
              identifier: rawType,
              optional: false,
            };
      result.push({
        name,
        type,
        wiki:stack.join('\n\n')
      })
      stack=[]
    } else {
      return new Err(
        `Error:Failed to parse line '${line}' as valid rust field declaration`
      );
    }
  }

  return new Ok(result);
}
