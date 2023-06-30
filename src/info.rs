use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Shows information related to specified app
#[argh(subcommand, name = "info")]
pub struct InfoCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch)]
    /// show mainfest of app
    show_mainfest: bool,
}
