#[macro_use]
extern crate lazy_static;
extern crate tar;

mod ca;
mod compression;
mod entrances;
mod executor;
mod parsers;
mod signature;
mod types;
mod utils;

use executor::step_execute;
use types::StepExecute;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   #[arg(short, long)]
   /// Name of the person to greet
   name: String,

   /// Number of times to greet
   #[arg(short, long, default_value_t = 1)]
   count: u8,
}

fn main() {
    let args = Args::parse();

   for _ in 0..args.count {
       println!("Hello {}!", args.name)
   }
}
