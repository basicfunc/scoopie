use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Install apps
#[argh(subcommand, name = "install")]
pub struct InstallCommand {
    // install command-specific options here, if any
    #[argh(positional)]
    app: String,
}
