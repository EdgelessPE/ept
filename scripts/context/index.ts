import { type Top } from "../type";
import { parseInnerValues } from "./values";
import { valuesRenderer } from "./markdown";
import { writeWiki } from "../writer";

export function genContext(
  { valuesTop, fnTop, top }: { valuesTop: Top; fnTop: Top; top: Top },
  { valuesFile, fnDir }: { valuesFile: string; fnDir: string },
  toFileName: string,
) {
  const valuesInfo = parseInnerValues(valuesFile);
  const valuesText = valuesRenderer(valuesTop, valuesInfo, { titleLevel: 2 });
  writeWiki(
    {
      title: top.title,
      description: top.description,
      content: valuesText,
    },
    toFileName,
  );
}
