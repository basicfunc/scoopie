mod info;
mod init;
mod install;
mod list;
mod locate;
mod nuke;
mod prelude;
mod query;
mod remove;

use argh::FromArgs;

use crate::error::ScoopieError;

use info::InfoCommand;
use init::InitCommand;
use install::InstallCommand;
use list::ListCommand;
use locate::LocateCommand;
use nuke::NukeCommand;
use query::QueryCommand;
use remove::RemoveCommand;

pub trait ExecuteCommand {
    fn exec(&self) -> Result<(), ScoopieError>;
}

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your favorite package manager
pub struct Commands {
    #[argh(subcommand)]
    cmd: Command,
}

impl ExecuteCommand for Commands {
    fn exec(&self) -> Result<(), ScoopieError> {
        println!("{:?}", self);
        self.cmd.exec()?;
        Ok(())
    }
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

impl ExecuteCommand for Command {
    fn exec(&self) -> Result<(), ScoopieError> {
        match self {
            Command::Info(x) => x.exec(),
            Command::Init(x) => x.exec(),
            Command::Install(x) => x.exec(),
            Command::List(x) => x.exec(),
            Command::Locate(x) => x.exec(),
            Command::Nuke(x) => x.exec(),
            Command::Query(x) => x.exec(),
            Command::Remove(x) => x.exec(),
        }
    }
}
