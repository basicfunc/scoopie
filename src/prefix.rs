use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Get Scoopie Prefix
#[argh(subcommand, name = "prefix")]
pub struct PrefixCommand {}
