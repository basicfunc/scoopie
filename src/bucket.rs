use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Manage Scoopie buckets
#[argh(subcommand, name = "bucket")]
pub struct BucketCommand {
    #[argh(positional)]
    /// add new bucket
    add: bool,
    #[argh(switch)]
    /// remove a bucket
    rm: bool,
    #[argh(switch)]
    /// list all buckets
    list: bool,
}
