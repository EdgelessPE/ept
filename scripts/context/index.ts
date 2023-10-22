import { type Top } from "../type";
import { parseInnerValues } from "./values";
import { functionRenderer, valuesRenderer } from "./markdown";
import { writeWiki } from "../writer";
import { type ValueInfo } from "./type";
import { parseInnerFn } from "./functions";

export function genContextWiki(
  { valuesTop, fnTop, top }: { valuesTop: Top; fnTop: Top; top: Top },
  {
    valuesFile,
    fnDir,
    appendValues,
  }: { valuesFile: string; fnDir: string; appendValues: ValueInfo[] },
  toFileName: string,
) {
  const valuesInfo = parseInnerValues(valuesFile);
  const valuesText = valuesRenderer(
    valuesTop,
    appendValues.concat(valuesInfo),
    { titleLevel: 2 },
  );
  const fnInfo = parseInnerFn("@/executor/functions");
  const fnText = functionRenderer(fnTop, fnInfo, { titleLevel: 2 });

  writeWiki(
    {
      title: top.title,
      description: top.description,
      content: `${valuesText}\n\n${fnText}`,
    },
    toFileName,
  );
}
