use argh::FromArgs;

use crate::download::Downloader;
use crate::sync::Sync;

#[derive(FromArgs, PartialEq, Debug)]
/// Install specified app or Update app(s)
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
            match Sync::now() {
                Ok(db) => println!("{:?}", db),
                Err(e) => eprintln!("{e}"),
            }
        } else if args.download_only {
            match args.app {
                Some(app) => Downloader::from().unwrap().download(&app).unwrap(),
                None => eprintln!("App argument required"),
            }
        } else {
            todo!()
        }
    }
}
