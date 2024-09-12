import type { PermissionInfo } from "./type";
import type { Top } from "../type";

export function permRenderer(
  top: Top,
  infos: PermissionInfo[],
  { titleLevel }: { titleLevel: number },
): string {
  const fieldRenderer = (info: PermissionInfo) =>
    `${"#".repeat(titleLevel + 1)} ${info.name}\n${info.wiki ?? ""}`;
  return `${"#".repeat(titleLevel)} ${top.title}
${top.description}
${infos.map(fieldRenderer).join("\n")}`;
}
