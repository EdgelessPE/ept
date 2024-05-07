export default {
    "*.ts": "eslint --fix",
    "*.rs":()=>["cargo fmt","cargo fix --allow-staged","cargo clippy --fix --allow-staged"]
}