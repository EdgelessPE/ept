export default {
  "*.ts": "biome check --write",
  "*.rs": () => [
    "cargo fmt",
    "cargo clippy --fix --allow-dirty --allow-staged",
  ],
  "doc/**/*.{md,mdx}": () => ["pnpm doc:translate --check"],
};
