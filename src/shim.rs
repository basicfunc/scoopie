use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Manipulate Scoop shims
#[argh(subcommand, name = "shim")]
pub struct ShimCommand {
    #[argh(subcommand)]
    /// subcommand: add, rm, list, info, alter
    subcommand: Subcommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommand {
    /// add a custom shim
    Add(AddCommand),

    /// remove shims
    Remove(RemoveCommand),

    /// list shims
    List(ListCommand),

    /// show shim information
    Info(InfoCommand),

    /// alternate a shim's target source
    Alter(AlterCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "add")]
/// Add a custom shim
pub struct AddCommand {
    #[argh(positional)]
    /// shim name
    pub shim_name: String,

    #[argh(positional)]
    /// command path
    pub command_path: String,

    #[argh(positional)]
    /// additional arguments
    pub args: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "rm")]
/// Remove shims
pub struct RemoveCommand {
    #[argh(positional)]
    /// shim names
    pub shim_names: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
/// List shims
pub struct ListCommand {
    #[argh(positional)]
    /// shim names or patterns
    pub shim_names: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "info")]
/// Show shim information
pub struct InfoCommand {
    #[argh(positional)]
    /// shim name
    pub shim_name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "alter")]
/// Alternate a shim's target source
pub struct AlterCommand {
    #[argh(positional)]
    /// shim name
    pub shim_name: String,
}
