import type { Top } from "../type";
import type { ValueInfo } from "./type";

// 渲染单一字段
function valueFieldRenderer(
  info: ValueInfo,
  { titleLevel }: { titleLevel: number },
): string {
  const demoText = info.demo
    ? `\n* 示例值：\`${info.demo}\` \n* 示例：\`if = '${info.name}=="${info.demo}"'\``
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
