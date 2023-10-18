import { type FieldInfo, type FileInfo } from "./type";
import { parseEnumDefinitions } from "./enum";
import { structRenderer } from "./markdown";
import { writeWiki } from "../writer";
import { splitBlock } from "../block";
import { type Top } from "../type";

// 读取 Rust 中的某个 struct，分析出所有字段信息
function parseStruct(fileInfo: FileInfo): FieldInfo[] {
  const { file, structName } = fileInfo;

  // 分割代码块
  const splittedBlock = splitBlock({
    file,
    startsWith: `pub struct ${structName}`,
  });
  // console.log(splittedBlock);

  // 解析枚举定义
  const enumValuesMap = parseEnumDefinitions(fileInfo);

  return splittedBlock.map(({ wiki, declaration, demo }) => {
    // 解析字段名和类型
    const m = declaration.match(/(\w+):\s?([\w<>()]+)/);
    if (m) {
      const [, name, rawType] = m;
      const enumValues = enumValuesMap[name];
      const type: FieldInfo["type"] =
        rawType.startsWith("Option<") && rawType.endsWith(">")
          ? {
              identifier: rawType.slice(7, -1),
              optional: true,
              enum: enumValues,
            }
          : {
              identifier: rawType,
              optional: false,
              enum: enumValues,
            };
      if (enumValues) {
        if (type.identifier !== "String") {
          throw new Error(
            `Error:Field '${name}' has enum but not a string (got '${type.identifier}')`,
          );
        } else {
          type.identifier = "String 枚举";
        }
      }
      return {
        name,
        type,
        wiki,
        demo,
      };
    } else {
      throw new Error(
        `Error:Failed to parse line '${declaration}' as valid rust field declaration`,
      );
    }
  });
}

// 支持从一个或多个文件中读取结构体并生成 wiki
export function genStructsWiki(
  top: Top,
  fileInfos: FileInfo[],
  toFileName: string,
) {
  const onlyOneStruct = fileInfos.length === 1;

  const structInfos = fileInfos.map((info) => ({
    fields: parseStruct(info),
    structName: info.structName,
    description: info.description,
  }));
  const structWikiTexts = structInfos.map((info) =>
    structRenderer(
      {
        title: info.structName.toLocaleLowerCase(),
        description: info.description,
      },
      info.fields,
      {
        titleLevel: onlyOneStruct ? 0 : 2,
      },
    ),
  );
  const needImportTag = structWikiTexts.find((node) => node.needImportTag);
  writeWiki(
    {
      title: top.title,
      imports: needImportTag
        ? ['import { Tag } from "../../components/tag.tsx"']
        : undefined,
      description: top.description,
      content: structWikiTexts.map((node) => node.text).join("\n\n"),
    },
    toFileName,
  );
}
