export default {
  "*.ts": "biome check --write",
  "*.rs": () => [
    "cargo fmt",
    "cargo clippy --fix --allow-dirty --allow-staged",
  ],
  "*.{md,mdx}": () => ["pnpm doc:translate --check"],
};
