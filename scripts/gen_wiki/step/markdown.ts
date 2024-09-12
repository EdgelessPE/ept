import type { StepInfo } from "./type";

function fieldRenderer(
  field: StepInfo["fields"][number],
  titleLevel: number,
): string {
  const identifier = field.type.enums
    ? field.type.identifier.replace("String", "String 枚举")
    : field.type.identifier;
  const typeText = `* 类型：\`${identifier}\``;
  const enumText = field.type.enums
    ? `
* ${identifier.includes("Vec") ? "元素" : ""}有效值：${field.type.enums.values
        .map((v) => `\`${v}\``)
        .join(" ")} ${
        field.type.enums.default
          ? `，缺省值：\`${field.type.enums.default}\``
          : ""
      }`
    : "";
  const demoText = field.demo ? `\n* 示例：${field.demo}` : "";
  const rulesText = field.rules?.length
    ? "\n* 校验规则：\n" + field.rules.map((rule) => `  * ${rule}`).join("\n")
    : "";
  return `${"#".repeat(titleLevel)} ${field.name}
${field.type.optional ? "<Tag>可选</Tag> " : ""}${field.wiki ?? ""}
${typeText} ${enumText} ${demoText} ${rulesText}`;
}

function permissionRenderer({
  key,
  level,
  targets,
  scene,
}: {
  key: string;
  level: string;
  targets: string;
  scene?: string;
}) {
  const sceneText = scene ? `\n* 场景：${scene}` : "";
  return `* 类型：${key}
* 等级：${level}
* 目标：${targets}${sceneText}`;
}

export function stepRenderer(
  info: StepInfo,
  { titleLevel }: { titleLevel: number },
): string {
  const reverseText = info.extra.reverseRun
    ? `\n${"#".repeat(titleLevel + 1)} 反向步骤\n${info.extra.reverseRun}`
    : "";
  const manifestText = info.extra.manifest
    ? `\n${"#".repeat(titleLevel + 1)} 装箱单\n${info.extra.manifest
        .map((item) => `* ${item}`)
        .join("\n")}`
    : "";
  return `\n${"#".repeat(titleLevel + 1)} 字段
${info.fields
  .map((field) => fieldRenderer(field, titleLevel + 2))
  .join("\n")} ${reverseText}${manifestText}
${"#".repeat(titleLevel + 1)} 权限
${
  info.extra.permissions?.length
    ? `${info.extra.permissions.map(permissionRenderer).join("\n<br/>\n")}`
    : "无需权限\n"
}`;
}
