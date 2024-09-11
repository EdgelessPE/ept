import readline from "node:readline";
import cp from "node:child_process";

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
  // 等待 22s，因为当前账号的 RPM 是 3
  await sleep(22000);
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
