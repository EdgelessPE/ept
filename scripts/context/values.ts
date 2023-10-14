import { PermissionLevel } from "../type";
import { splitBlock } from "../block";

export function parseInnerValues(file: string):{name:string,level:PermissionLevel,wiki?:string,demo?:string}[] {
  const splittedBlock = splitBlock({ file, startsWith: "define_values!" });
  return splittedBlock.map(({ wiki, declaration, demo }) => {
    const m = declaration.match(
      /\{"\$\{(\w+)\}",[\w.()]+\(\),PermissionLevel::(\w+)\},?/
    );
    if (m) {
      return {
        name: m[1],
        level: m[2] as PermissionLevel,
        wiki,
        demo,
      };
    } else {
      throw new Error(
        `Error:Failed to parse value declaration line '${declaration}'`
      );
    }
  });
}
