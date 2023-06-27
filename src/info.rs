use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Display information about an app
#[argh(subcommand, name = "info")]
pub struct InfoCommand {
    #[argh(positional)]
    app: String,
}
