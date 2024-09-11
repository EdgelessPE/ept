import path from "path";
import { existsSync } from "node:fs";
import { readdir, mkdir, stat, cp } from "node:fs/promises";
import { askYn, translate } from "./utils";

const DOC_ROOT = path.join(__dirname, "../../doc");

async function main(): Promise<boolean> {
  // 检查配置文件
  if (!existsSync(".chatgpt-md-translator")) {
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
      if (dirent.isDirectory()) {
        // 递归文件夹
        await procDir(path.join(base, dirent.name));
      } else if (dirent.isFile()) {
        const zhMdPath = path.join(curZhBase, dirent.name);
        const enMdPath = path.join(curEnDir, dirent.name);
        if (dirent.name.endsWith(".md") || dirent.name.endsWith(".mdx")) {
          // 读取对应的英文 md 状态
          if (existsSync(enMdPath)) {
            const { mtime: zhMTime } = await stat(zhMdPath);
            const { mtime: enMTime } = await stat(enMdPath);
            // 如果修改间隔超过 1h 则需要更新
            if (zhMTime.valueOf() - enMTime.valueOf() > 60 * 60 * 1000) {
              markdowns.push(path.join(base, dirent.name));
            }
          } else {
            // 文件不存在，也需要处理
            const parentDir = path.dirname(enMdPath);
            if (!existsSync(parentDir)) {
              await mkdir(parentDir, { recursive: true });
            }
            markdowns.push(path.join(base, dirent.name));
          }
        } else {
          // 其他文件需要复制
          await cp(zhMdPath, enMdPath);
        }
      }
    }
  }

  await procDir();

  // 如果只是检查，直接可以返回结果了

  if (markdowns.length > 0) {
    if (process.argv.includes("--check")) {
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

  return true;
}

main().then((res) => process.exit(res ? 0 : 1));
