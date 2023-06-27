use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Update apps, or Scoopie itself
#[argh(subcommand, name = "update")]
pub struct UpdateCommand {
    // update command-specific options here, if any
    #[argh(positional)]
    app: Option<String>,
}
