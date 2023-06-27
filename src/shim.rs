use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Manipulate Scoop shims
#[argh(subcommand, name = "shim")]
pub struct ShimCommand {
    // shim command-specific options here, if any
}
