mod commands;
mod core;
mod error;
mod utils;

use commands::*;

fn main() {
    match argh::from_env::<Commands>().exec() {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {e}"),
    }
}
