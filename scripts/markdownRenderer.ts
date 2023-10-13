import { FieldInfo } from "./type";

// 渲染单一字段
function fieldRenderer(
  info: FieldInfo,
  { titleLevel }: { titleLevel: number }
): string {
  const clearWiki = info.wiki.replace(/</g, "\\<").replace(/>/g, "\\>");
  const enumText = info.type.enum ? info.type.enum.map((t) => `\`${t}\``).join(' ') : undefined;
  return `
${"#".repeat(titleLevel)} ${info.name}
${info.type.optional ? "<Tag>可选</Tag> " : ""}${clearWiki}
* 类型：\`${info.type.identifier}\`${enumText?"\n* 有效值："+enumText:""}`;
}

// 渲染一个结构
export function structRenderer(
  top: { title: string; description?: string },
  fields: FieldInfo[],
  { titleLevel }: { titleLevel: number }
) {
  const needImportTag = fields.find((item) => item.type.optional);
  const fieldsText = fields
    .map((item) => fieldRenderer(item, { titleLevel: titleLevel + 1 }))
    .join("\n");
  const title =
    titleLevel === 0 ? "" : `${"#".repeat(titleLevel)} ${top.title}`;

  return {
    text: `${title} ${
      top.description ? "\n" + top.description + "\n" : ""
    } ${fieldsText}`,
    needImportTag,
  };
}
