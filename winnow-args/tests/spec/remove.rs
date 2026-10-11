use winnow_args::Args;

/// Remove one.
#[derive(Args)]
pub struct Opts {
    /// And what it holds.
    #[arg(long)]
    recursive: bool,
}
