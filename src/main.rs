use std::eprintln;

// External Crate Imports.
use argh::from_env;

// Internal Module Imports
use scoopie::init::InitCommand;
use scoopie::nuke::NukeCommand;
use scoopie::prefix::PrefixCommand;
use scoopie::{Command, Scoopie};

fn main() {
    let cmd: Scoopie = from_env();

    let info = match PrefixCommand::show() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let scoopie_home = info.0;
    let config_dir = info.1;

    match (&cmd.cmd, &scoopie_home.exists()) {
        (Command::Init(_), true) => {
            println!(
                "Prefix: {}\nConfig: {}",
                PrefixCommand::prefix().unwrap().display(),
                PrefixCommand::config().unwrap().display()
            );
            println!("INFO: Scoopie is already initialized.");
            return;
        }
        (Command::Init(config), false) => match InitCommand::from(&config) {
            Ok(x) => println!("{x}"),
            Err(e) => {
                eprintln!("{e}");
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
        Command::Nuke(_) => match NukeCommand::nuke(&[&scoopie_home, &config_dir]) {
            Ok(_) => println!("👋🏻 Goodbye!!"),
            Err(e) => eprintln!("{e}"),
        },
        _ => {}
    }

    println!("{:?}", cmd);
}
