use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// List all installed apps
#[argh(subcommand, name = "list")]
pub struct ListCommand {}
