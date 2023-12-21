import { parseFilePath } from "../utils";
import fs from "fs";
import type { StepInfo } from "./type";
import { getCommentsInBlock } from "../block";

const REGEX_PERMISSION_BLOCK =
  /Permission {\s*key: ([^,]+),\s*level: ([^,]+),\s*targets: ([^,]+),\s*[^}]+/gm;

function parser(
  type: "key" | "level" | "targets",
  { raw, text }: { text: string; raw: boolean },
) {
  if (!raw) {
    return text;
  }
  switch (type) {
    case "key":
      return text.split("::")[1];
    case "level":
      if (text.includes("judge_perm_level")) {
        return `根据目标路径决定`;
      } else {
        return text.split("::")[1];
      }
    case "targets":
      return text.split(".")[1];
  }
}

// 分割 Permission 申明，返回原始申明语句
function splitPermissions(file: string) {
  const filePath = parseFilePath(file);
  if (!fs.existsSync(filePath)) {
    throw new Error(`Error:Failed to open file '${filePath}'`);
  }
  const text = fs.readFileSync(filePath).toString();
  const lines = text.split("\n");

  // 找出 Generalizable 实现的代码块
  let startIndex = -1;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.startsWith("impl Generalizable for Step")) {
      startIndex = i;
      break;
    }
  }
  if (startIndex === -1) {
    throw new Error(`Error:Can't find Generalizable impl block in '${file}'`);
  }
  let endIndex = -1;
  for (let i = startIndex; i < lines.length; i++) {
    const line = lines[i];
    if (line.trimEnd() === "}") {
      endIndex = i;
      break;
    }
  }
  if (endIndex === -1) {
    throw new Error(`Error:Can't find Generalizable impl block in '${file}'`);
  }
  const implBlockText = lines.slice(startIndex, endIndex).join("\n");

  // 匹配 Permission 结构体
  let m: string[] | null = implBlockText.match(REGEX_PERMISSION_BLOCK);
  if (!m) {
    const comments = getCommentsInBlock({
      file,
      startsWith: "impl Generalizable for Step",
    }).extra;
    if (!comments) return [];
    m = comments
      .split("key: ")
      .filter(Boolean)
      .map((t) => {
        const normalLines = `key: ${t}`
          .split("\n")
          .filter(Boolean)
          .map((line) => `//@ ${line}`)
          .join("\n");
        return `\n${normalLines}`;
      });
  }
  return m.map((text) => {
    const sp = text.split("\n");
    const [key, level, targets, scene] = sp.slice(1).map((_t) => {
      let t = _t.trim();
      if (!t) return undefined;
      let raw = true;
      if (t.startsWith("//@")) {
        t = t.replace("//@", "").trim();
        raw = false;
      }
      if (t.endsWith(",")) {
        t = t.slice(0, -1);
      }

      const [type, text] = t.split(": ").map((t) => t.trim());
      return parser(type as "key" | "level" | "targets", {
        text,
        raw,
      });
    });
    return {
      key,
      level,
      targets,
      scene,
    };
  });
}

export function parsePermission(
  file: string,
): StepInfo["extra"]["permissions"] {
  const splitRes = splitPermissions(file);
  console.log(splitRes);

  return [];
}