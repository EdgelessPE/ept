import readline from "node:readline";
import cp from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

export async function askYn(prompt: string): Promise<boolean> {
  return await new Promise((resolve) => {
    rl.question(prompt, (answer) => {
      resolve(answer.toLowerCase() === "y");
    });
  });
}

async function sleep(ms: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, ms));
}

export async function translate(zh: string, en: string): Promise<boolean> {
  await sleep(1000);
  return await new Promise((resolve) => {
    cp.exec(
      `pnpm chatgpt-md-translator "${zh}" -o "${en}"`,
      (error, stdout, stderr) => {
        const err = stderr || error;
        if (err) {
          console.error(err);
          resolve(false);
        } else {
          resolve(true);
        }
      },
    );
  });
}

export async function calcMD5(filePath: string): Promise<string> {
  return new Promise((resolve) => {
    const rs = createReadStream(filePath);
    const hash = createHash("md5");
    rs.on("data", hash.update.bind(hash));
    rs.on("end", () => {
      resolve(hash.digest("hex"));
    });
  });
}
