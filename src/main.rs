mod executor;
mod parsers;
mod types;
mod utils;
// use parsers::{parse_package,parse_workflow};
use executor::step_execute;
use types::StepExecute;

fn main() {
    // let pkg=parse_package(String::from("./examples/VSCode/package.toml"));
    // println!("{:?}",pkg);

    // let flows=parse_workflow(String::from("./examples/VSCode/workflows/setup.toml")).unwrap();
    // println!("{:?}",flows);

    // let res = step_execute(StepExecute {
    //     command: String::from("echo hello world"),
    //     pwd: None,
    // });
    // println!("{:?}", res)
}
