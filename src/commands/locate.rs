use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Shows the location of specified app
#[argh(subcommand, name = "locate")]
pub struct LocateCommand {
    #[argh(positional)]
    app: String,
}
