use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Show or clear the download cache.
#[argh(subcommand, name = "cache")]
pub struct CacheCommand {
    #[argh(switch)]
    /// show the cache
    show: bool,
    #[argh(option)]
    /// removes the specified cache
    remove: Option<String>,
}
