use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Uninstall an app
#[argh(subcommand, name = "uninstall")]
pub struct UninstallCommand {
    #[argh(positional)]
    app: String,
}
