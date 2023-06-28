pub mod bucket;
pub mod cache;
pub mod cat;
pub mod cleanup;
pub mod download;
pub mod info;
pub mod init;
pub mod install;
pub mod list;
pub mod nuke;
pub mod prefix;
pub mod search;
pub mod shim;
pub mod status;
pub mod uninstall;
pub mod update;
pub mod which;

// External Crate Imports.
use argh::FromArgs;

// Internal Module Imports
use bucket::BucketCommand;
use cache::CacheCommand;
use cat::CatCommand;
use cleanup::CleanupCommand;
use download::DownloadCommand;
use info::InfoCommand;
use init::InitCommand;
use install::InstallCommand;
use list::ListCommand;
use nuke::NukeCommand;
use prefix::PrefixCommand;
use search::SearchCommand;
use shim::ShimCommand;
use status::StatusCommand;
use uninstall::UninstallCommand;
use update::UpdateCommand;
use which::WhichCommand;

pub const PREFIX_KEY: &'static str = "SCOOPIE_HOME";
pub const DEFAULT_PREFIX: &'static str = "scoopie";

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your simple package manager
pub struct Scoopie {
    #[argh(subcommand)]
    pub cmd: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Bucket(BucketCommand),
    Cache(CacheCommand),
    Cat(CatCommand),
    Cleanup(CleanupCommand),
    Download(DownloadCommand),
    Info(InfoCommand),
    Init(InitCommand),
    Install(InstallCommand),
    List(ListCommand),
    Nuke(NukeCommand),
    Prefix(PrefixCommand),
    Search(SearchCommand),
    Shim(ShimCommand),
    Status(StatusCommand),
    Uninstall(UninstallCommand),
    Update(UpdateCommand),
    Which(WhichCommand),
}