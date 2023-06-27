use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Manage Scoopie buckets
#[argh(subcommand, name = "bucket")]
pub struct BucketCommand {
    #[argh(option)]
    /// add new bucket
    add: Option<String>,
    #[argh(option)]
    /// remove a bucket
    rm: Option<String>,
    #[argh(switch)]
    /// list all buckets
    list: bool,
}
