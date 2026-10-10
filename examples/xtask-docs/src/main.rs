//! Generate markdown and man pages the way brush's xtask does.
//!
//! Brush's tasks are `cargo xtask …`. The documentation ones are
//! `cargo xtask gen docs`. This crate is that command for a winnow-args
//! program:
//!
//! ```text
//! cargo run -p xtask-docs -- gen docs --example brush_builtins
//! cargo run -p xtask-docs -- gen docs --package just-docs
//! ```
//!
//! Pages land in `target/docs/<name>/`.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use winnow_args::Args;
use winnow_args::Subcommand;
use winnow_args_man::Manual;
use winnow_args_spec::Catalog;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Documentation tasks, in brush's xtask shape.
#[derive(Debug, Args)]
#[arg(name = "xtask-docs")]
struct Cli {
    #[arg(subcommand)]
    task: Task,
}

#[derive(Debug, Subcommand)]
enum Task {
    /// Generate documentation.
    Gen(Generate),
}

#[derive(Debug, Args)]
struct Generate {
    #[arg(subcommand)]
    what: Generated,
}

#[derive(Debug, Subcommand)]
enum Generated {
    /// Markdown and man pages for a derived command.
    Docs(Docs),
}

#[derive(Debug, Args)]
struct Docs {
    /// Document this example of the `winnow-args` package.
    ///
    /// `brush_builtins` is brush's builtins, and the default.
    #[arg(long)]
    example: Option<String>,

    /// Document this workspace package instead of an example.
    #[arg(long)]
    package: Option<String>,

    /// Directory for the pages. The default is `target/docs/<name>`.
    ///
    /// Its `md` and `man` subdirectories lose the pages of an earlier run:
    /// the files with a page's extension, and nothing else.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// Describe the command as this target triple compiles it.
    ///
    /// The crate is only checked, so the target needs no linker.
    #[arg(long)]
    target: Option<String>,

    /// Document this type. The default is the one command no other names.
    ///
    /// A crate with several commands names each one. `NAME=TYPE` also gives
    /// the word the pages call it, for a command that has no name of its own.
    /// `CRATE::TYPE` is the type of one crate, when two have one of that name.
    #[arg(long, value_name = "[NAME=]TYPE")]
    root: Vec<String>,
}

fn main() -> ExitCode {
    match generate(&Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask-docs: {error}");
            ExitCode::from(1)
        }
    }
}

fn generate(cli: &Cli) -> Result<()> {
    let Task::Gen(Generate {
        what: Generated::Docs(docs),
    }) = &cli.task;
    let (package, example) = documented(docs)?;
    let root = workspace()?;
    let name = example.unwrap_or(package);
    let work = root.join("target/xtask-docs");
    let target_dir = work.join("target");
    let runs = work.join("spec").join(name);
    if runs.exists() {
        fs::remove_dir_all(&runs)?;
    }
    // Cargo compiles a crate again when WINNOW_ARGS_SPEC changes. A new
    // directory each run has every crate that derives write its fragments,
    // and none from a type that is gone.
    let spec = runs.join(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_nanos()
            .to_string(),
    );

    // The derive writes the fragments while it expands. Nothing is linked.
    let mut check = cargo("check", &root, &target_dir);
    check.args(["-p", package]);
    if let Some(example) = example {
        check.args(["--example", example]);
    }
    triple(&mut check, docs.target.as_deref());
    check.env("WINNOW_ARGS_SPEC", &spec);
    run(&mut check, &format!("cargo check -p {package}"))?;

    if !spec.is_dir() {
        return Err(format!(
            "no derive wrote a fragment: does {package} derive `Args`, and does `cargo check -p {package}` compile it?"
        )
        .into());
    }
    let catalog = Catalog::load(&spec)?;
    // (the name given, the name of last resort, the type)
    let roots: Vec<(Option<&str>, &str, String)> = if docs.root.is_empty() {
        vec![(None, name, catalog.root()?)]
    } else {
        docs.root
            .iter()
            .map(|root| match root.split_once('=') {
                Some((name, ty)) => (Some(name), ty, ty.to_owned()),
                // A crate is not part of the word a command goes by.
                None => (None, root.rsplit("::").next().unwrap_or(root), root.clone()),
            })
            .collect()
    };
    let out = match &docs.out {
        Some(path) => path.clone(),
        None => root.join("target/docs").join(name),
    };
    let mut markdown = Vec::new();
    let mut manual = Vec::new();
    for (given, fallback, ty) in &roots {
        let command = catalog.stitch(ty)?;
        let bin = given.unwrap_or(if command.name.is_empty() {
            fallback
        } else {
            &command.name
        });
        markdown.extend(winnow_args_markdown::render_pages(&command, bin)?);
        manual.extend(Manual::default().render_pages(&command, bin)?);
    }
    for pages in [&markdown, &manual] {
        let mut seen = HashSet::new();
        if let Some((name, _)) = pages.iter().find(|(name, _)| !seen.insert(name)) {
            return Err(format!("two roots would both write {name}").into());
        }
    }
    write_pairs(&out.join("md"), &markdown)?;
    write_pairs(&out.join("man"), &manual)?;
    println!(
        "{md} markdown, {man} man, in {out}",
        md = markdown.len(),
        man = manual.len(),
        out = out.display()
    );
    Ok(())
}

/// `--example brush_builtins` is the default. A package replaces it.
fn documented(docs: &Docs) -> Result<(&str, Option<&str>)> {
    match (docs.example.as_deref(), docs.package.as_deref()) {
        (Some(_), Some(_)) => Err("pass either --example or --package".into()),
        (None, Some(package)) => Ok((package, None)),
        (Some(example), None) => Ok(("winnow-args", Some(example))),
        (None, None) => Ok(("winnow-args", Some("brush_builtins"))),
    }
}

fn workspace() -> Result<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    Ok(manifest.canonicalize()?)
}

fn cargo(subcommand: &str, root: &Path, target_dir: &Path) -> Command {
    // The cargo that runs this program, when it was run through one.
    let mut command = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command
        .arg(subcommand)
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target_dir);
    command
}

fn triple(command: &mut Command, triple: Option<&str>) {
    if let Some(triple) = triple {
        command.arg("--target").arg(triple);
    }
}

fn run(command: &mut Command, what: &str) -> Result<()> {
    if command.status()?.success() {
        Ok(())
    } else {
        Err(format!("{what} failed").into())
    }
}

/// The pages, and no page of an earlier run: it would describe a command that
/// is gone. Only files with a page's extension are removed.
fn write_pairs(dir: &Path, pages: &[(String, String)]) -> Result<()> {
    fs::create_dir_all(dir)?;
    let kinds: HashSet<_> = pages
        .iter()
        .filter_map(|(name, _)| Path::new(name).extension())
        .collect();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|kind| kinds.contains(kind)) {
            fs::remove_file(path)?;
        }
    }
    for (name, body) in pages {
        fs::write(dir.join(name), body)?;
    }
    Ok(())
}
