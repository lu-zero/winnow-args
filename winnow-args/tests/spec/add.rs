use winnow_args::Args;

/// Add one.
#[derive(Args)]
pub struct Opts {
    /// Over what is there.
    #[arg(long)]
    force: bool,
}
