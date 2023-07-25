mod download;
mod sync;

use argh::FromArgs;

use download::*;
use sync::Sync;

#[derive(FromArgs, PartialEq, Debug)]
/// Install specified app or update app(s)
#[argh(subcommand, name = "install")]
pub struct InstallCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch, short = 'd')]
    /// download app to cache
    download_only: bool,

    #[argh(switch, short = 'S')]
    /// sync all repos
    sync: bool,

    #[argh(switch, short = 'a')]
    /// update all apps
    update_all: bool,
}

impl InstallCommand {
    pub fn from(args: InstallCommand) {
        if args.sync {
            Sync::now().map_or_else(
                |e| eprintln!("{e}"),
                |buckets| buckets.iter().for_each(|b| println!("{b}")),
            );
        } else if args.download_only {
            match args.app {
                Some(app) => DownloadEntry::try_from(&app)
                    .and_then(|d| d.download(false))
                    .unwrap(),
                None => eprintln!("App argument required"),
            }
        } else {
            todo!()
        }
    }
}
