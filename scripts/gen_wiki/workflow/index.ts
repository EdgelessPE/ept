import { type Top } from "../type";
import { splitBlock } from "../block";
import { parseTypeDeclaration } from "../utils";
import { renderWorkflow } from "./markdown";
import { writeWiki } from "../writer";

export function genWorkflowWiki({
  file,
  top,
  titleLevel,
}: {
  file: string;
  top: Top;
  titleLevel: number;
}) {
  const blocks = splitBlock({
    file,
    startsWith: `pub struct WorkflowHeader`,
  }).map((raw) => ({
    ...raw,
    type: parseTypeDeclaration(raw.declaration),
  }));
  const content = blocks
    .map((b) => renderWorkflow(b, titleLevel + 1))
    .join("\n\n");
  writeWiki(
    {
      title: top.title,
      description: top.description,
      imports: ['import Tag from "../../../components/tag.tsx"'],
      content,
    },
    "4-steps/0-general",
  );
}
