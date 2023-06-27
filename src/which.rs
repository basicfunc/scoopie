use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Locate a shim/executable (similar to 'which' on Linux)
#[argh(subcommand, name = "which")]
pub struct WhichCommand {
    // which command-specific options here, if any
    #[argh(positional)]
    shim: String,
}
