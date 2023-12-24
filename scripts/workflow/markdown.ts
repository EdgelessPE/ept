import type { CommonFieldInfo } from "../type";

export function renderWorkflow(
  {
    wiki,
    extra,
    type,
    declaration,
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
${wiki}${validationText}`;
}
