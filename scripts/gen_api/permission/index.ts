import type { PermissionInfo } from "./type";
import { splitBlock } from "../block";
import { permRenderer } from "./markdown";
import type { Top } from "../type";
import { writeWiki } from "../writer";

export function genPermissionsWiki({
  file,
  levelTop,
  keyTop,
  top,
  toFileName,
}: {
  file: string;
  levelTop: Top;
  keyTop: Top;
  top: Top;
  toFileName: string;
}) {
  const levelInfos = getPermInfo(file, "PermissionLevel");
  const levelText = permRenderer(levelTop, levelInfos, { titleLevel: 2 });
  const keyInfos = getPermInfo(file, "PermissionKey");
  const keyText = permRenderer(keyTop, keyInfos, { titleLevel: 2 });

  writeWiki(
    {
      title: top.title,
      description: top.description,
      content: `${levelText}\n\n${keyText}`,
    },
    toFileName,
  );
}

function getPermInfo(file: string, enumName: string): PermissionInfo[] {
  return splitBlock({
    file,
    startsWith: `pub enum ${enumName}`,
  }).map((node) => ({ name: node.declaration.slice(0, -1), wiki: node.wiki }));
}
