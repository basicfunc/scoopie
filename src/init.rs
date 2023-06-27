use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize all scoopie related stuff.
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(option)]
    /// path where you would like give home to scoopie.
    path: Option<String>,
}
