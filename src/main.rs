use std::eprintln;

// External Crate Imports.
use argh::from_env;

use scoopie::bucket::BucketCommand;
// Internal Module Imports
use scoopie::cat::CatCommand;
use scoopie::init::InitCommand;
use scoopie::nuke::NukeCommand;
use scoopie::ScoopieInfo;
use scoopie::{Command, Scoopie};

fn main() {
    let cmd: Scoopie = from_env();

    let info = match ScoopieInfo::get() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };

    let scoopie_home = &info.home;
    let config_dir = &info.config;

    match (&cmd.cmd, &scoopie_home.exists()) {
        (Command::Init(_), true) => {
            println!("{info}");
            println!("INFO: Scoopie is already initialized.");
            return;
        }
        (Command::Init(config), false) => match InitCommand::from(&config) {
            Ok(x) => println!("{x}"),
            Err(e) => {
                eprintln!("Error: {e}");
                return;
            }
        },
        (_, false) => {
            // If init is not passed and prefix doesn't exist
            eprintln!(
                "Error: Scoopie home directory does not exist. Run `scoopie init` to set it up."
            );
            return;
        }
        _ => {}
    }

    match &cmd.cmd {
        Command::Nuke(_) => match NukeCommand::nuke(&info) {
            Ok(_) => println!("👋🏻 Goodbye!!"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Command::Cat(config) => match CatCommand::run(&config) {
            Ok(()) => {}
            Err(e) => eprintln!("Error: {e}"),
        },
        Command::Bucket(config) => BucketCommand::run(&config, &info.buckets),
        _ => {}
    }

    println!("{:?}", cmd);
}
