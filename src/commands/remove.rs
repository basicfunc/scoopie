use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Removes the specified app
#[argh(subcommand, name = "rm")]
pub struct RemoveCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch, short = 'a')]
    /// remove all apps
    all: bool,

    #[argh(switch, short = 'c')]
    /// remove cached apps and repos
    cache: bool,
}
