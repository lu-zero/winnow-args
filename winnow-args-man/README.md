# winnow-args-man

Renders a command stitched by [winnow-args-spec](https://docs.rs/winnow-args-spec)
as one [roff](https://docs.rs/roff) man page per subcommand.

```rust,no_run
use std::path::Path;
use winnow_args_spec::Catalog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Written by `WINNOW_ARGS_SPEC=/abs/target/spec cargo check`.
    let catalog = Catalog::load(Path::new("target/spec"))?;
    let command = catalog.stitch(&catalog.root()?)?;
    for (name, page) in winnow_args_man::Manual::default().render_pages(&command, "tool")? {
        std::fs::write(Path::new("man").join(name), page)?;
    }
    Ok(())
}
```

A page is named after its command, `tool-use-add.1`. `Manual` holds the
header's date, source and title; `Manual::date_of` turns
`SOURCE_DATE_EPOCH` into the date. A description's backticks and stars are
printed as written.

The repository's
[examples](https://github.com/lu-zero/winnow-args/tree/HEAD/examples) have a
whole generator, for one target or another.
