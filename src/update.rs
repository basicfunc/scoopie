use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Update apps, or Scoopie itself
#[argh(subcommand, name = "update")]
pub struct UpdateCommand {
    #[argh(positional)]
    app: Option<String>,
}
