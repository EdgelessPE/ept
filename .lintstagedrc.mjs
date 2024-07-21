export default {
    "*.ts": "eslint --fix",
    "*.rs":()=>["cargo fmt","cargo clippy --fix --allow-dirty --allow-staged"]
}