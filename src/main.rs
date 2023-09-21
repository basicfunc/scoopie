mod commands;
mod core;
mod error;
mod utils;

use commands::*;
use console::Emoji;

fn main() {
    let start = std::time::Instant::now();

    match argh::from_env::<Commands>().exec() {
        Ok(_) => {
            println!(
                "{} Done, took {} secs.",
                Emoji("✨", ":)"),
                start.elapsed().as_secs()
            );
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
