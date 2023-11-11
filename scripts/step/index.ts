import { type CommonFieldInfo, type Top } from "../type";
import fs from "fs";
import { parseFilePath } from "../utils";
import path from "path";
import { getCommentsInBlock, splitBlock } from "../block";
import { type StepInfo } from "./type";
import { writeWiki } from "../writer";
import { stepsRenderer } from "./markdown";

function getExtra(file: string): StepInfo["extra"] {
  // TODO:添加更多字段
  return {
    run: getCommentsInBlock({ file, startsWith: "fn run" }).wiki ?? "",
    reverseRun: getCommentsInBlock({ file, startsWith: "fn reverse_run" }).wiki,
  };
}
function formatField({
  declaration,
  wiki,
  demo,
  extra,
}: CommonFieldInfo): StepInfo["fields"][number] {
  const m = declaration.match(/(\w+):\s?([\w<>()]+)/);
  if (m) {
    const [, name, rawType] = m;
    const optional = rawType.startsWith("Option<") && rawType.endsWith(">");
    return {
      name,
      type: {
        optional,
        identifier: optional ? rawType.slice(7, -1) : rawType,
      },
      wiki,
      demo,
      rules: extra?.split("\n"),
    };
  } else {
    throw new Error(
      `Error:Failed to parse line '${declaration}' as valid rust field declaration`,
    );
  }
}

export function genStepsWiki(
  top: Top,
  { srcDir }: { srcDir: string },
  toFileName: string,
) {
  const dir = parseFilePath(srcDir);
  const fileNames = fs
    .readdirSync(dir)
    .filter((name) => name.endsWith(".rs") && name !== "mod.rs");
  const getStepName = (fileName: string): string => {
    const stem = fileName.split(".")[0];
    if (stem === "mv") {
      return "Move";
    }
    return `${stem[0].toUpperCase()}${stem.slice(1)}`;
  };
  const steps: StepInfo[] = [];
  for (const fileName of fileNames) {
    const file = path.join(dir, fileName);
    const stepName = getStepName(fileName);
    const fields = splitBlock({
      file,
      startsWith: `pub struct Step${stepName}`,
    });
    const extra = getExtra(file);
    steps.push({
      name: stepName,
      fields: fields.map(formatField),
      extra,
    });
  }
  writeWiki(
    {
      ...top,
      imports: ['import { Tag } from "../../components/tag.tsx"'],
      content: stepsRenderer(steps, { titleLevel: 1 }),
    },
    toFileName,
  );
}
