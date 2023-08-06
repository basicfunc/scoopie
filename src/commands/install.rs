use crate::core::{bucket::*, config::*, download::*};

use argh::FromArgs;
use rayon::prelude::*;

use super::prelude::*;
use crate::error::ScoopieError;

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

impl ExecuteCommand for InstallCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        if self.sync {
            let config = Config::read()?;
            let status = config
                .known_buckets()
                .par_iter()
                .map(Bucket::sync_from)
                .collect::<Result<Vec<_>, _>>()?;

            println!("{:?}", status);
            Ok(())
        } else if self.download_only {
            match &self.app {
                Some(app) => {
                    let status = Downloader::build_for(app)?.download(true);
                    println!("{:?}", status);
                }
                None => eprintln!("App argument required"),
            };
            Ok(())
        } else {
            todo!()
        }
    }
}
