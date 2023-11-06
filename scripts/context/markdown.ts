import type { Top } from "../type";
import type { FnValue, ValueInfo } from "./type";

// 渲染内置变量的单一字段
function valueFieldRenderer(
  info: ValueInfo,
  { titleLevel }: { titleLevel: number },
): string {
  const q = typeof info.demoValue === "string" ? '"' : "";
  const demoText =
    info.demoValue !== undefined
      ? `\n* 示例值：\`${info.demoValue}\` \n* 示例：
    \`\`\`toml
    ${
      info.demo ? "# 非条件字段中使用\n    " + info.demo + "\n\n    " : ""
    }# 条件字段中使用\n    if = '${info.name}==${q}${info.demoValue}${q}'
    \`\`\``
      : "";
  return `${"#".repeat(titleLevel)} ${info.name}
${info.wiki}
* 权限等级：[\`${
    info.level
  }\`](/nep/definition/3-permissions#${info.level.toLowerCase()})${demoText}`;
}

export function valuesRenderer(
  top: Top,
  infos: ValueInfo[],
  { titleLevel }: { titleLevel: number },
) {
  return `${"#".repeat(titleLevel)} ${top.title}
${top.description ?? ""}
${infos
  .map((info) => valueFieldRenderer(info, { titleLevel: titleLevel + 1 }))
  .join("\n")}`;
}

// 渲染内置函数的单一字段
function functionFieldRenderer(
  info: FnValue,
  { titleLevel }: { titleLevel: number },
): string {
  const permissionLevel =
    info.permission.level === "JUDGE_WITH_PATH"
      ? "根据输入路径决定"
      : `[\`${info.permission.level}\`](/nep/definition/3-permissions#${info.permission.level})`;
  return `${"#".repeat(titleLevel)} ${info.name}
${info.wiki ?? ""}
* 入参校验：${info.validationRules ?? ""}
* 示例：\`${info.demo ?? ""}\`
* 权限：
  * 类型：[\`${info.permission.key}\`](/nep/definition/3-permissions#${
    info.permission.key
  })
  * 等级：${permissionLevel}`;
}

export function functionRenderer(
  top: Top,
  infos: FnValue[],
  { titleLevel }: { titleLevel: number },
) {
  return `${"#".repeat(titleLevel)} ${top.title}
${top.description ?? ""}
${infos
  .map((info) => functionFieldRenderer(info, { titleLevel: titleLevel + 1 }))
  .join("\n")}`;
}
