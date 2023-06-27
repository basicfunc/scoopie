use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Locate a shim/executable (similar to 'which' on Linux)
#[argh(subcommand, name = "which")]
pub struct WhichCommand {
    #[argh(positional)]
    shim: String,
}
