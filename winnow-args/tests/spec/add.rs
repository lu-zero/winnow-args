use winnow_args::Args;

/// Add one.
#[derive(Args)]
pub struct Opts {
    /// Over what is there.
    #[arg(long)]
    force: bool,
}

// A bound and no import: what is below it is still read where it is.
pub fn bound<'a>(text: &'a str) -> impl Sized + use<'a> {
    text
}

pub mod both {
    use winnow_args::Args;

    /// `self` and `super` are the modules they stand for.
    #[derive(Args)]
    pub struct SpecBoth {
        #[arg(flatten)]
        own: self::SpecInner,
        #[arg(flatten)]
        above: super::Opts,
    }

    /// Add both.
    #[derive(Args)]
    pub struct SpecInner {
        /// The two of them.
        #[arg(long)]
        both: bool,
    }
}

#[cfg(test)]
mod stub {
    // No derive: the `Opts` this file is asked for is the derived one.
    struct Opts;
}
