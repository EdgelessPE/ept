export default {
    "*.ts": "eslint --fix",
    "*.rs":()=>["cargo fmt","cargo clippy"]
}