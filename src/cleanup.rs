use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Cleanup apps by removing old versions
#[argh(subcommand, name = "cleanup")]
pub struct CleanupCommand {
    #[argh(switch, short = 'a')]
    /// cleanup all apps    
    all: bool,
    #[argh(switch, short = 'k')]
    /// cleanup outdated apps    
    cache: bool,
    #[argh(positional)]
    app: Option<String>,
}
