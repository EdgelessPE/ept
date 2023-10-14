import type { Top } from "../type";
import type { ValueInfo } from "./type";

// 渲染单一字段
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
${info.wiki} ${demoText}`;
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
