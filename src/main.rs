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
use search::SearchCommand;
use shim::ShimCommand;
use status::StatusCommand;
use uninstall::UninstallCommand;
use update::UpdateCommand;
use utils::get_prefix;
use which::WhichCommand;

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

    let scoopie_home = match get_prefix() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    match (&cmd.cmd, &scoopie_home.exists()) {
        (Command::Init(_), true) => {
            println!("INFO: $SCOOPIE_HOME already exists.");
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

    println!("{:?}", cmd);
}
