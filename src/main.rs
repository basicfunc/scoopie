mod bucket;
mod cache;
mod cat;
mod cleanup;
mod download;
mod info;
mod init;
mod install;
mod list;
mod search;
mod shim;
mod status;
mod uninstall;
mod update;
mod utils;
mod which;

use argh::FromArgs;

use bucket::BucketCommand;
use cache::CacheCommand;
use cat::CatCommand;
use cleanup::CleanupCommand;
use download::DownloadCommand;
use info::InfoCommand;
use init::InitCommand;
use install::InstallCommand;
use list::ListCommand;
use search::SearchCommand;
use shim::ShimCommand;
use status::StatusCommand;
use uninstall::UninstallCommand;
use update::UpdateCommand;
use utils::get_prefix;
use which::WhichCommand;

use std::path::PathBuf;
use std::{eprintln, format};

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your simple package manager
struct Scoopie {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Bucket(BucketCommand),
    Cache(CacheCommand),
    Cat(CatCommand),
    Cleanup(CleanupCommand),
    Download(DownloadCommand),
    Info(InfoCommand),
    Init(InitCommand),
    Install(InstallCommand),
    List(ListCommand),
    Search(SearchCommand),
    Shim(ShimCommand),
    Status(StatusCommand),
    Uninstall(UninstallCommand),
    Update(UpdateCommand),
    Which(WhichCommand),
}

fn main() {
    let cmd: Scoopie = argh::from_env();

    let _scoopie_home = match get_prefix() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    // let prefix = PathBuf::from(scoopie_home);

    // if !prefix.exists() {
    //     eprintln!(
    //         "Won't able to find home for Scoopie.\nIt was expected at: {prefix:?}\nConfigure it properly or run `scoopie init`"
    //     );
    //     return;
    // }

    println!("{:?}", cmd);
}
