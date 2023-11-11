import type { StepInfo } from "./type";

function fieldRenderer(
  field: StepInfo["fields"][number],
  titleLevel: number,
): string {
  const demoText = field.demo ? `\n* 示例：${field.demo}` : "";
  const rulesText = field.rules?.length
    ? "\n" + field.rules.map((rule) => `* ${rule}`).join("\n")
    : "";
  return `${"#".repeat(titleLevel)} ${field.name}
${field.type.optional ? "<Tag>可选</Tag> " : ""}${field.wiki ?? ""}
* 类型：\`${field.type.identifier}\` ${demoText} ${rulesText}`;
}

function stepRenderer(
  info: StepInfo,
  { titleLevel }: { titleLevel: number },
): string {
  const reverseText = info.extra.reverseRun
    ? `\n${"#".repeat(titleLevel + 1)} 反向步骤\n${info.extra.reverseRun}`
    : "";
  return `${"#".repeat(titleLevel)} ${info.name}
${info.extra.run}
${"#".repeat(titleLevel + 1)} 字段
${info.fields
  .map((field) => fieldRenderer(field, titleLevel + 2))
  .join("\n")} ${reverseText}`;
}
export function stepsRenderer(
  infos: StepInfo[],
  { titleLevel }: { titleLevel: number },
) {
  return `${infos
    .map((info) => stepRenderer(info, { titleLevel: titleLevel + 1 }))
    .join("\n")}`;
}
