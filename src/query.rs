use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps (supports regex and full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}
