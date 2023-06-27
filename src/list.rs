use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// List installed apps
#[argh(subcommand, name = "list")]
pub struct ListCommand {
    // list command-specific options here, if any
}
