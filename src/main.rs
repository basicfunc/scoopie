mod info;
mod init;
mod install;
mod list;
mod locate;
mod nuke;
mod query;
mod remove;

use argh::FromArgs;

use init::InitCommand;
use nuke::NukeCommand;

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your favorite package manager
struct Scoopie {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Info(InfoCommand),
    Init(InitCommand),
    Install(InstallCommand),
    List(ListCommand),
    Locate(LocateCommand),
    Nuke(NukeCommand),
    Query(QueryCommand),
    Remove(RemoveCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Install specified app or Update app(s)
#[argh(subcommand, name = "install")]
struct InstallCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch, short = 'S')]
    /// sync all repos
    sync: bool,

    #[argh(switch, short = 'a')]
    /// update all apps
    update_all: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Removes the specified app
#[argh(subcommand, name = "rm")]
struct RemoveCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch, short = 'a')]
    /// remove all apps
    all: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps (supports regex and full-text search)
#[argh(subcommand, name = "query")]
struct QueryCommand {
    #[argh(positional)]
    query: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Shows the location of specified app
#[argh(subcommand, name = "locate")]
struct LocateCommand {
    #[argh(positional)]
    app: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Shows information related to specified app
#[argh(subcommand, name = "info")]
struct InfoCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch)]
    /// show mainfest of app
    show_mainfest: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// List all installed apps
#[argh(subcommand, name = "list")]
struct ListCommand {}

fn main() {
    let cmd: Scoopie = argh::from_env();
    let cmd = cmd.cmd;
    println!("{:?}", cmd);

    match cmd {
        Command::Install(_) => todo!(),
        Command::Remove(_) => todo!(),
        Command::Query(_) => todo!(),
        Command::Locate(_) => todo!(),
        Command::Info(_) => todo!(),
        Command::Init(config) => match InitCommand::from(config) {
            Ok(x) => println!("{x}"),
            Err(e) => eprintln!("{e:?}"),
        },
        Command::List(_) => todo!(),
        Command::Nuke(_) => match NukeCommand::from() {
            Ok(_) => println!("👋🏻 Goodbye, Scoopie has been deleted!"),
            Err(e) => eprintln!("{e:?}"),
        },
    }
}
