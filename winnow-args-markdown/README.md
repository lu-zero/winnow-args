# winnow-args-markdown

Renders a command stitched by [winnow-args-spec](https://docs.rs/winnow-args-spec)
as one Markdown page per subcommand.

```rust,no_run
use std::path::Path;
use winnow_args_spec::Catalog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Written by `WINNOW_ARGS_SPEC=/abs/target/spec cargo check`.
    let catalog = Catalog::load(Path::new("target/spec"))?;
    let command = catalog.stitch(&catalog.root()?)?;
    for (name, page) in winnow_args_markdown::render_pages(&command, "tool")? {
        std::fs::write(Path::new("md").join(name), page)?;
    }
    Ok(())
}
```

A page is named after its command, `tool-use-add.md`, and a subcommand is a
link to its own page.

The repository's
[examples](https://github.com/lu-zero/winnow-args/tree/HEAD/examples) have a
whole generator, for one target or another.
