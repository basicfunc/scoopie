mod commands;
mod core;
mod error;
mod utils;

use commands::*;

fn main() {
    let r = argh::from_env::<Commands>().exec();
    println!("{:?}", r);
}
