import path from "path";
import { existsSync } from "node:fs";
import { readdir, cp, mkdir } from "node:fs/promises";
import { askYn, calcMD5, translate } from "./utils";
import { flushStoreMd5, readStoreMd5, writeStoreMd5 } from "./store";

const DOC_ROOT = path.join(__dirname, "../../doc");
const IS_CHECK_MODE = process.argv.includes("--check");

async function main(): Promise<boolean> {
  // 检查配置文件
  if (!IS_CHECK_MODE && !existsSync(".chatgpt-md-translator")) {
    console.error(
      "Error: Config not found, read the 'scripts/translate_wiki/README.md'",
    );
    return false;
  }

  // 拼接语言根目录
  const zhDir = path.join(DOC_ROOT, "zh");
  const enDir = path.join(DOC_ROOT, "en");

  // 迭代中文目录，收集需要翻译的 Markdown 文件，保存的时候使用相对于语言目录的相对路径
  const markdowns: string[] = [];

  // dfs 处理闭包，base 是相对语言目录的相对路径
  async function procDir(base: string = "") {
    // 拼接绝对路径
    const curZhBase = path.join(zhDir, base);
    const curEnDir = path.join(enDir, base);
    // 读取 zh 目录
    const list = await readdir(curZhBase, { withFileTypes: true });
    // 迭代 md 文件
    for (const dirent of list) {
      const fileBasePath = path.join(base, dirent.name);
      if (dirent.isDirectory()) {
        // 递归文件夹
        await procDir(fileBasePath);
      } else if (dirent.isFile()) {
        const zhMdPath = path.join(curZhBase, dirent.name);
        const enMdPath = path.join(curEnDir, dirent.name);
        if (dirent.name.endsWith(".md") || dirent.name.endsWith(".mdx")) {
          // 判断中英文的 md5 是否匹配
          if (existsSync(enMdPath)) {
            const { zh, en } = await readStoreMd5(fileBasePath);
            if (zh && en) {
              const zhMd5 = await calcMD5(zhMdPath);
              const enMd5 = await calcMD5(enMdPath);
              if (zhMd5 !== zh || enMd5 !== en) {
                // 中英文的 md5 匹配不上，需要重新生成
                markdowns.push(fileBasePath);
              }
            } else {
              // 英文文件存在但是没有 md5，同样需要生成
              markdowns.push(fileBasePath);
            }
          } else {
            // 英文文件不存在，需要生成
            const parentDir = path.dirname(enMdPath);
            if (!existsSync(parentDir)) {
              await mkdir(parentDir, { recursive: true });
            }
            markdowns.push(fileBasePath);
          }
        } else {
          // 其他文件需要复制
          if (!existsSync(enMdPath)) await cp(zhMdPath, enMdPath);
        }
      }
    }
  }

  await procDir();

  // 如果只是检查，直接可以返回结果了
  if (markdowns.length > 0) {
    if (IS_CHECK_MODE) {
      console.error(
        `Error: The following ${markdowns.length} markdown files needs translation:\n${markdowns.join("\n")}`,
      );
      return false;
    } else {
      // 确认是否开始翻译
      console.log(
        `Info: The following ${markdowns.length} markdown files needs translation:\n${markdowns.join("\n")}\n`,
      );
      const res = await askYn("Start now?(y/n)");
      if (!res) {
        return false;
      }
    }
  } else {
    return true;
  }

  // 翻译
  for (const relativePath of markdowns) {
    console.log(`Info: Processing '${relativePath}'`);
    const zhPath = path.join(zhDir, relativePath);
    const enPath = path.join(enDir, relativePath);
    const res = await translate(zhPath, enPath);
    if (!res) {
      console.error(`Error: Failed to translate '${relativePath}'`);
      return false;
    }
  }

  // 对列表中的文件重新生成 md5
  for (const base of markdowns) {
    const zhPath = path.join(zhDir, base);
    const enPath = path.join(enDir, base);
    const zhMd5 = await calcMD5(zhPath);
    const enMd5 = await calcMD5(enPath);
    await writeStoreMd5(base, { zh: zhMd5, en: enMd5 });
  }

  // 保存 store
  await flushStoreMd5();

  return true;
}

main().then((res) => process.exit(res ? 0 : 1));
