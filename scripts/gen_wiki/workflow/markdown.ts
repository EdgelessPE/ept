import type { CommonFieldInfo } from "../type";

export function renderWorkflow(
  {
    wiki,
    extra,
    type,
    declaration,
    demo,
  }: CommonFieldInfo & {
    type:
      | {
          identifier: string;
          optional: boolean;
          name: string;
        }
      | undefined;
  },
  titleLevel: number,
) {
  if (!type) {
    throw new Error(
      `Error:Can't parse type info for workflow field declaration ${declaration}`,
    );
  }
  const validationText = extra ? `\n* 校验规则：${extra}` : "";
  return `${"#".repeat(titleLevel)} ${type.name}
${type.optional ? "<Tag>可选</Tag> " : ""}${wiki ?? ""}${validationText}
* 示例：\n${demo}`;
}
