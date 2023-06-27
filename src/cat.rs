use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Show content of specified manifest
#[argh(subcommand, name = "cat")]
pub struct CatCommand {
    #[argh(positional)]
    app: String,
}
