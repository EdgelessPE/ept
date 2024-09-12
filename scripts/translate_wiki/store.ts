import { existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";

// 从保存的 json 文件中读取先前文件的 md5 值
interface StoreNode {
  zh: string | undefined;
  en: string | undefined;
}
let cachedJson: Record<string, StoreNode> | undefined = undefined;
async function getCachedJson(): Promise<Record<string, StoreNode>> {
  if (!cachedJson) {
    const filePath = path.join(__dirname, "./store.json");
    if (existsSync(filePath)) {
      const text = await readFile(filePath);
      cachedJson = JSON.parse(text.toString());
    } else {
      cachedJson = {};
    }
  }

  return cachedJson!;
}
export async function readStoreMd5(fileBasePath: string): Promise<StoreNode> {
  const json = await getCachedJson();
  return json[fileBasePath] ?? { zh: undefined, en: undefined };
}

export async function writeStoreMd5(fileBasePath: string, node: StoreNode) {
  const json = await getCachedJson();
  json[fileBasePath] = node;
  const text = JSON.stringify(json, null, 2);
  await writeFile(path.join(__dirname, "./store.json"), text);
}
