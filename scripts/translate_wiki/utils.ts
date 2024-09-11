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

export async function translate(zh: string, en: string): Promise<boolean> {
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
