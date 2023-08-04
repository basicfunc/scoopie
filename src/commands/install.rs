use crate::core::{bucket::*, config::*, download::*};

use argh::FromArgs;
use rayon::prelude::*;

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
    pub fn install(&self) {
        if self.sync {
            let config = Config::read().unwrap();
            let buckets = config.known_buckets();
            let status: Result<Vec<_>, _> = buckets.par_iter().map(Bucket::sync_from).collect();
            let status = status.unwrap();
            println!("{:?}", status);
        } else if self.download_only {
            match &self.app {
                Some(app) => {
                    let downloader = Downloader::build_for(app).unwrap();
                    let status = downloader.download(true);
                    println!("{:?}", status);
                }
                None => eprintln!("App argument required"),
            }
        } else {
            todo!()
        }
    }
}
