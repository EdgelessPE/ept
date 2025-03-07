import type { Top } from "../type";
import { writeWiki } from "../writer";
import { parseInnerFn } from "./functions";
import { functionRenderer, valuesRenderer } from "./markdown";
import type { ValueInfo } from "./type";
import { parseInnerValues } from "./values";

export function genContextWiki(
	{ valuesTop, fnTop, top }: { valuesTop: Top; fnTop: Top; top: Top },
	{
		valuesFile,
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
