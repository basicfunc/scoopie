use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Show status and check for new app versions
#[argh(subcommand, name = "status")]
pub struct StatusCommand {}
