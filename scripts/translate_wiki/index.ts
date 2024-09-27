import path from "path";
import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readdir, cp, mkdir, readFile } from "node:fs/promises";
import { ask, calcMD5, hasChinese, translate } from "./utils";
import { hasStoreChanged, readStoreMd5, writeStoreMd5 } from "./store";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

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
  const allMarkdowns: string[] = [];

  // dfs 处理闭包，base 是相对语言目录的相对路径
  async function procDir(base: string = "") {
    // 拼接绝对路径
    const curZhBase = path.join(zhDir, base);
    const curEnBase = path.join(enDir, base);
    // 读取目录
    const zhList = await readdir(curZhBase, { withFileTypes: true });
    const enList = await readdir(curEnBase, { withFileTypes: true });
    // 逐一对比这两个目录是否一致
    if (zhList.length !== enList.length) {
      console.error(
        `Error: The number of dirents in '${curZhBase}' and '${curEnBase}' is not equal`,
      );
      process.exit(1);
    }
    for (let i = 0; i < zhList.length; i++) {
      const zh = zhList[i];
      const en = enList[i];
      if (zh.name !== en.name || zh.isFile() !== en.isFile()) {
        console.error(
          `Error: The dirent name in '${base}' is not equal: zh: ${zh.name}, en: ${en.name}`,
        );
        process.exit(1);
      }
    }
    // 迭代 md 文件
    for (const dirent of zhList) {
      const fileBasePath = path.join(base, dirent.name);
      if (dirent.isDirectory()) {
        // 递归文件夹
        await procDir(fileBasePath);
      } else if (dirent.isFile()) {
        const zhMdPath = path.join(curZhBase, dirent.name);
        const enMdPath = path.join(curEnBase, dirent.name);
        if (dirent.name.endsWith(".md") || dirent.name.endsWith(".mdx")) {
          allMarkdowns.push(fileBasePath);
          // 判断中英文的 md5 是否匹配
          if (existsSync(enMdPath)) {
            const { zh, en } = await readStoreMd5(fileBasePath);
            if (zh && en) {
              const zhMd5 = await calcMD5(zhMdPath);
              const enMd5 = await calcMD5(enMdPath);
              if (zhMd5 !== zh) {
                // 中文的 md5 匹配不上，需要重新生成
                markdowns.push(fileBasePath);
              } else if (enMd5 !== en) {
                // 英文的 md5 匹配不上，理解为手动润色，重新生成英文的 md5
                console.log(
                  `Info: Manually translated '${fileBasePath}', regenerate en md5`,
                );
                const enMd5 = await calcMD5(enMdPath);
                await writeStoreMd5(fileBasePath, { zh, en: enMd5 });
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
          // 其他文件会被强行同步
          await cp(zhMdPath, enMdPath);
          // 检查是否包含中文
          const zhText = await readFile(zhMdPath);
          if (hasChinese(zhText.toString())) {
            console.warn(
              `Warning: '${fileBasePath}' contains Chinese, please use i18n key`,
            );
          }
        }
      }
    }
  }

  await procDir();

  if (IS_CHECK_MODE) {
    // 如果只是检查出问题，直接抛出
    if (markdowns.length > 0) {
      console.error(
        `Error: The following ${markdowns.length} markdown files needs translation:\n${markdowns.join("\n")}`,
      );
      return false;
    }
    // 检查模式下额外检查英文文档中是否包含中文
    let has = false;
    for (const base of allMarkdowns) {
      const enPath = path.join(enDir, base);
      const enText = await readFile(enPath);
      const res = hasChinese(enText.toString());
      if (res) {
        console.error(`Error: '${base}' contains Chinese : ${res}}`);
        has = true;
      }
    }
    if (has) return false;
    // 如果更新了 store，提交暂存
    if (hasStoreChanged()) {
      console.log("Info: Update store.json, staging it");
      execSync(`git add ./store.json`);
    }
    return true;
  } else {
    if (markdowns.length > 0) {
      // 确认是否开始翻译
      console.log(
        `Info: The following ${markdowns.length} markdown files needs translation:\n${markdowns.map((t) => `    ${t}`).join("\n")}\n`,
      );
      const res = await ask(
        "Start translation now? Or input 'u' to update md5 without translation(y/u/n) : ",
      );
      if (res === "y") {
        // 确认翻译，继续往下走
      } else if (res === "u") {
        // 不翻译，确认当前中文文档和英文文档经过手动润色
        for (const relativePath of markdowns) {
          const zhPath = path.join(zhDir, relativePath);
          const enPath = path.join(enDir, relativePath);
          const zhMd5 = await calcMD5(zhPath);
          const enMd5 = await calcMD5(enPath);
          await writeStoreMd5(relativePath, { zh: zhMd5, en: enMd5 });
        }
        return true;
      } else {
        return false;
      }
    } else {
      // 没有需要翻译的文档
      console.log("Info: All markdown files are up to date");
      return true;
    }
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
    } else {
      // 检查是否包含中文
      const enText = await readFile(enPath);
      if (hasChinese(enText.toString())) {
        console.warn(
          `Warning: '${relativePath}' contains Chinese, please check it manually`,
        );
      }
      // 更新 md5
      const zhMd5 = await calcMD5(zhPath);
      const enMd5 = await calcMD5(enPath);
      await writeStoreMd5(relativePath, { zh: zhMd5, en: enMd5 });
    }
  }

  return true;
}

main().then((res) => process.exit(res ? 0 : 1));
