mod parsers;
mod types;
use parsers::{parse_package,parse_workflow};

fn main() {
    // let pkg=parse_package(String::from("./examples/VSCode/package.toml"));
    // println!("{:?}",pkg);

    let flows=parse_workflow(String::from("./examples/VSCode/workflows/setup.toml")).unwrap();
    println!("{:?}",flows);
}
