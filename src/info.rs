use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Display information about an app
#[argh(subcommand, name = "info")]
pub struct InfoCommand {
    // info command-specific options here, if any
    #[argh(positional)]
    app: String,
}
