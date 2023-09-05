use crate::core::{buckets::*, download::*, install::install};

use argh::FromArgs;

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
            let status = Buckets::sync();
            println!("{:?}", status);
        } else if self.download_only {
            match &self.app {
                Some(app) => {
                    let st = Downloader::download(app, true);
                    println!("{:?}", st);
                }
                None => {
                    eprintln!("App argument required");
                }
            };
        } else {
            let app = self.app.as_ref().unwrap();
            let _ = install(app);
        }

        Ok(())
    }
}
