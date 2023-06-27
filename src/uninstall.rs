use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Uninstall an app
#[argh(subcommand, name = "uninstall")]
pub struct UninstallCommand {
    // uninstall command-specific options here, if any
    #[argh(positional)]
    app: String,
}
