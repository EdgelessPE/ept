mod parsers;
mod types;
use parsers::{parse_package};

fn main() {
    let pkg=parse_package(String::from("./examples/VSCode/package.toml"));
    println!("{:?}",pkg);
}
