mod info;
mod init;
mod install;
mod list;
mod locate;
mod nuke;
mod query;
mod remove;

use argh::FromArgs;

use info::InfoCommand;
use init::InitCommand;
use install::InstallCommand;
use list::ListCommand;
use locate::LocateCommand;
use nuke::NukeCommand;
use query::QueryCommand;
use remove::RemoveCommand;

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your favorite package manager
pub struct Commands {
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

impl Commands {
    pub fn run(self) {
        println!("{:?}", self);

        match self.cmd {
            Command::Info(_) => todo!(),
            Command::Init(_) => todo!(),
            Command::Install(args) => args.install(),
            Command::List(_) => todo!(),
            Command::Locate(_) => todo!(),
            Command::Nuke(_) => todo!(),
            Command::Query(args) => args.query(),
            Command::Remove(_) => todo!(),
        };
    }
}
