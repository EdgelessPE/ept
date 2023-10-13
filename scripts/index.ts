import { structRenderer } from "./markdownRenderer";
import { parseStruct } from "./struct";
import { FileInfo } from "./type";
import { writeWiki } from "./writer";

// 支持从一个或多个文件中读取结构体并生成 wiki
function genStructsWiki(
  top: { title: string; description?: string },
  fileInfos: FileInfo[],
  toFileName: string
) {
  const onlyOneStruct = fileInfos.length === 1;

  const structInfos = fileInfos.map((info) => ({
    fields: parseStruct(info).unwrap(),
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
      }
    )
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
    toFileName
  );
}

genStructsWiki(
  {
    title: "包描述文件",
    description:
      "描述 Nep 包的基本信息，位于 [`package.toml`](/nep/struct/2-inner.html#包描述文件)",
  },
  [
    {
      file: "@/types/package.rs",
      structName: "Package",
      description: "通用信息表",
    },
    {
      file: "@/types/software.rs",
      structName: "Software",
      description: "软件包独占表",
    },
  ],
  "1-package"
);
