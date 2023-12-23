import { type Top } from "../type";
import { splitBlock } from "../block";
import { parseTypeDeclaration } from "../utils";

export function genWorkflowWiki({ file, top }: { file: string; top: Top }) {
  const blocks = splitBlock({
    file,
    startsWith: `pub struct WorkflowHeader`,
  }).map((raw) => ({
    ...raw,
    type: parseTypeDeclaration(raw.declaration),
  }));
  console.log(blocks);
}
