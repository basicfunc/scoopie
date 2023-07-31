mod commands;
mod core;
mod error;
mod utils;

use commands::Commands;

fn main() {
    argh::from_env::<Commands>().run();

    // match cmd.cmd {
    //     Command::Install(args) => InstallCommand::from(args),
    //     Command::Remove(_) => todo!(),
    //     Command::Query(query) => match BucketData::try_from(query) {
    //         Ok(results) => println!("{results}"),
    //         Err(e) => eprintln!("{e}"),
    //     },
    //     Command::Locate(_) => todo!(),
    //     Command::Info(_) => todo!(),
    //     Command::Init(config) => match InitCommand::from(config) {
    //         Ok(x) => println!("{x}"),
    //         Err(e) => eprintln!("{e}"),
    //     },
    //     Command::List(_) => todo!(),
    //     Command::Nuke(_) => match NukeCommand::from() {
    //         Ok(_) => println!("👋🏻 Goodbye, Scoopie has been deleted!"),
    //         Err(e) => eprintln!("{e}"),
    //     },
    // }
}
