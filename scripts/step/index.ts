import { type CommonFieldInfo, type Top } from "../type";
import fs from "fs";
import { parseFilePath } from "../utils";
import path from "path";
import { getCommentsInBlock, splitBlock } from "../block";
import { type StepInfo } from "./type";
import { writeWiki } from "../writer";
import { stepsRenderer } from "./markdown";
import { parsePermission } from "./permission";

function getExtra(file: string): StepInfo["extra"] {
  // TODO:添加更多字段
  return {
    run: getCommentsInBlock({ file, startsWith: "fn run" }).wiki ?? "",
    reverseRun: getCommentsInBlock({ file, startsWith: "fn reverse_run" }).wiki,
    manifest: getCommentsInBlock({
      file,
      startsWith: "fn get_manifest",
    }).extra?.split("\n"),
    permissions: parsePermission(file),
  };
}
function formatField({
  declaration,
  wiki,
  demo,
  extra,
  enums,
}: CommonFieldInfo): StepInfo["fields"][number] {
  const getEnums = (
    line: string | undefined,
  ): StepInfo["fields"][number]["type"]["enums"] => {
    if (!line) {
      return undefined;
    }
    const [a, defaultValue] = line.split("|");
    return {
      values: a
        .split(" ")
        .filter((v) => !!v)
        .map((val) => val.trim()),
      default: defaultValue.trim(),
    };
  };
  const m = declaration.match(/(\w+):\s?([\w<>()]+)/);
  if (m) {
    const [, name, rawType] = m;
    const optional = rawType.startsWith("Option<") && rawType.endsWith(">");
    return {
      name,
      type: {
        optional,
        identifier: optional ? rawType.slice(7, -1) : rawType,
        enums: getEnums(enums),
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
      imports: ['import Tag from "../../components/tag.tsx"'],
      content: stepsRenderer(steps, { titleLevel: 1 }),
    },
    toFileName,
  );
}
