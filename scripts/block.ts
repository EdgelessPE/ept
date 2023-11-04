import fs from "fs";
import { parseFilePath } from "./utils";
import type { CommonFieldInfo } from "./type";

// 优雅地在 md 行之间加入换行符
function gracefulJoinMarkdown(lines: string[]): string {
  let insideCodeBlock = false;
  let finalText = "";

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.startsWith("```") || line.startsWith(":::")) {
      insideCodeBlock = !insideCodeBlock;
    }
    if (i === lines.length - 1) {
      finalText = finalText + line;
      break;
    }
    if (insideCodeBlock) {
      finalText = finalText + `${line}\n`;
    } else {
      finalText = finalText + `${line}\n\n`;
    }
  }

  return finalText;
}

// 匹配用花括号包裹的代码块，返回块内的所有行
function matchBlock(startsWith: string, text: string): string[] {
  const lines = text.split("\n");
  // 检索结构体申明开始行号
  let startLineIndex = -1;
  lines.find((line, index) => {
    if (line.startsWith(`${startsWith} {`)) {
      startLineIndex = index;
      return true;
    } else {
      return false;
    }
  });
  if (startLineIndex === -1) return [];

  // 向下检索申明结束行号
  let endLineIndex = -1;
  for (let i = startLineIndex; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line === "}") {
      endLineIndex = i;
      break;
    }
  }
  if (endLineIndex === -1) return [];

  return lines.slice(startLineIndex + 1, endLineIndex);
}

// 注册已知的注释信息
const COMMENTS_DEFINITION: Array<{
  // 字段名称
  key: keyof Omit<CommonFieldInfo, "declaration">;
  // 注释起始标识
  startsWith: string[];
  // 把栈渲染为字符串
  renderer: (stack: string[]) => string | undefined;
}> = [
  {
    key: "wiki",
    startsWith: ["/// ", "//- "],
    renderer: (stack) =>
      stack.length > 0 ? gracefulJoinMarkdown(stack) : undefined,
  },
  {
    key: "demo",
    startsWith: ["//# ", "//#"],
    renderer: (stack) => {
      if (stack.length > 0 && stack[0].startsWith("```")) {
        stack = stack.map((line) => `  ${line}`);
        stack.unshift("");
      }
      return stack.length > 0 ? stack.join("\n") : undefined;
    },
  },
  {
    key: "extra",
    startsWith: ["//@ "],
    renderer: (stack) => (stack.length > 0 ? stack.join("\n") : undefined),
  },
];

// 匹配代码块并解析注释
export function splitBlock({
  file,
  startsWith,
}: {
  file: string;
  startsWith: string;
}): CommonFieldInfo[] {
  const filePath = parseFilePath(file);
  if (!fs.existsSync(filePath)) {
    throw new Error(`Error:Failed to open file '${filePath}'`);
  }
  const text = fs.readFileSync(filePath).toString();
  const lines = matchBlock(startsWith, text);
  if (lines.length === 0) {
    throw new Error(
      `Error:Failed to find block starts with '${startsWith}' in '${filePath}'`,
    );
  }

  // 构建注释信息
  const stackRegister: Array<{
    prefix: string;
    key: string;
  }> = [];
  const stackMap: Record<string, string[]> = {};
  for (const node of COMMENTS_DEFINITION) {
    stackMap[node.key] = [];
    for (const prefix of node.startsWith) {
      stackRegister.push({
        prefix,
        key: node.key,
      });
    }
  }

  const result: CommonFieldInfo[] = [];

  const clearLines = lines.map((line) => line.trim());
  for (const line of clearLines) {
    // 依次检查是否匹配注册过的注释前缀
    for (const { prefix, key } of stackRegister) {
      if (line.startsWith(prefix)) {
        stackMap[key].push(line.slice(prefix.length));
        break;
      }
    }

    // 忽略普通或其他特殊注释
    if (line.startsWith("//")) continue;

    // 走到这个位置说明匹配到申明语句了

    // 使用栈构造字段
    const r: CommonFieldInfo = { declaration: line };
    for (const { key, renderer } of COMMENTS_DEFINITION) {
      r[key] = renderer(stackMap[key]);
      stackMap[key] = [];
    }
    result.push(r);
  }

  // 结束后如果有栈不为空，则建一个空白申明语句
  let isStackClear = true;
  for (const { key } of COMMENTS_DEFINITION) {
    if (stackMap[key].length) {
      isStackClear = false;
      break;
    }
  }
  if (!isStackClear) {
    const r: CommonFieldInfo = { declaration: "" };
    for (const { key, renderer } of COMMENTS_DEFINITION) {
      r[key] = renderer(stackMap[key]);
      stackMap[key] = [];
    }
    result.push(r);
  }

  return result;
}

// 获取一个代码块中的注释信息
type PureCommentInfo = Omit<CommonFieldInfo, "declaration">;
export function getCommentsInBlock({
  file,
  startsWith,
}: {
  file: string;
  startsWith: string;
}): PureCommentInfo {
  const filePath = parseFilePath(file);
  if (!fs.existsSync(filePath)) {
    throw new Error(`Error:Failed to open file '${filePath}'`);
  }
  const text = fs.readFileSync(filePath).toString();

  // 筛选出 block 内的所有注释
  const lines: string[] = [];
  let insideBlock = false;
  let braceCount = 0;
  for (const _line of text.split("\n")) {
    const line = _line.trim();
    if (line.startsWith(startsWith) && line.endsWith("{")) {
      insideBlock = true;
      continue;
    }
    if (insideBlock) {
      if (line.endsWith("{")) {
        braceCount++;
      }
      if (line.startsWith("//")) {
        lines.push(line);
      }
      if (line === "}") {
        braceCount--;
        if (braceCount <= 0) {
          insideBlock = false;
          break;
        }
      }
    }
  }

  // 构造 key map
  const keyMap: Record<string, keyof PureCommentInfo> = {};
  const startsWithArr: string[] = [];
  for (const node of COMMENTS_DEFINITION) {
    for (const st of node.startsWith) {
      keyMap[st] = node.key;
      startsWithArr.push(st);
    }
  }

  // 合并注释
  const cateComments: PureCommentInfo = {};
  for (const line of lines) {
    for (const st of startsWithArr) {
      if (line.startsWith(st)) {
        const key = keyMap[st];
        const content = line.slice(st.length);
        if (cateComments[key]) {
          cateComments[key] = `${cateComments[key]}\n${content}`;
        } else {
          cateComments[key] = content;
        }
        break;
      }
    }
  }

  return cateComments;
}
