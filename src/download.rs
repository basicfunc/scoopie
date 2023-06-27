use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Download apps in the cache folder and verify hashes
#[argh(subcommand, name = "download")]
pub struct DownloadCommand {
    #[argh(positional)]
    app: String,
}
