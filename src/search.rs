use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps
#[argh(subcommand, name = "search")]
pub struct SearchCommand {
    // search command-specific options here, if any
    #[argh(positional)]
    query: String,
}
