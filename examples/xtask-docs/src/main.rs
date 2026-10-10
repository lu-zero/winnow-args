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

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use winnow_args::Args;
use winnow_args::Subcommand;
use winnow_args_man::Manual;
use winnow_args_spec::Catalog;

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
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// Build the documented crate for this target triple.
    #[arg(long)]
    target: Option<String>,
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

fn generate(cli: &Cli) -> Result<(), String> {
    let Task::Gen(Generate {
        what: Generated::Docs(docs),
    }) = &cli.task;
    let (package, example) = documented(docs)?;
    let root = workspace()?;
    let name = example.unwrap_or(package);
    let work = root.join("target/xtask-docs");
    let target_dir = work.join("target");
    let spec = work.join("spec").join(name);
    if spec.exists() {
        fs::remove_dir_all(&spec).map_err(|error| error.to_string())?;
    }
    // Cargo does not rebuild when only WINNOW_ARGS_SPEC changes, and a
    // fragment that already exists with a different body is an error.
    if target_dir.exists() {
        let mut clean = cargo("clean", &root, &target_dir);
        clean.args(["-p", package]);
        triple(&mut clean, docs.target.as_deref());
        let status = clean.status().map_err(|error| error.to_string())?;
        if !status.success() {
            return Err(format!("cargo clean -p {package} failed"));
        }
    }

    let mut build = cargo("build", &root, &target_dir);
    build.args(["-p", package]);
    if let Some(example) = example {
        build.args(["--example", example]);
    }
    triple(&mut build, docs.target.as_deref());
    build.env("WINNOW_ARGS_SPEC", &spec);
    let status = build.status().map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!("cargo build -p {package} failed"));
    }

    let crate_dir = one_dir(&spec)?;
    let catalog = Catalog::load(&crate_dir).map_err(|error| error.to_string())?;
    let root_ident = catalog.root().map_err(|error| error.to_string())?;
    let command = catalog
        .stitch(&root_ident)
        .map_err(|error| error.to_string())?;
    let bin = if command.name.is_empty() {
        name.to_owned()
    } else {
        command.name.clone()
    };
    let out = match &docs.out {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => root.join(path),
        None => root.join("target/docs").join(name),
    };
    let markdown =
        winnow_args_markdown::render_pages(&command, &bin).map_err(|error| error.to_string())?;
    let manual = Manual::default()
        .render_pages(&command, &bin)
        .map_err(|error| error.to_string())?;
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
fn documented(docs: &Docs) -> Result<(&str, Option<&str>), String> {
    match (docs.example.as_deref(), docs.package.as_deref()) {
        (Some(_), Some(_)) => Err("pass either --example or --package".to_owned()),
        (None, Some(package)) => Ok((package, None)),
        (Some(example), None) => Ok(("winnow-args", Some(example))),
        (None, None) => Ok(("winnow-args", Some("brush_builtins"))),
    }
}

fn workspace() -> Result<PathBuf, String> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    manifest.canonicalize().map_err(|error| error.to_string())
}

fn cargo(subcommand: &str, root: &Path, target_dir: &Path) -> Command {
    let mut command = Command::new("cargo");
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

fn one_dir(spec: &Path) -> Result<PathBuf, String> {
    let mut dirs = Vec::new();
    let entries = fs::read_dir(spec).map_err(|error| format!("{}: {error}", spec.display()))?;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    match dirs.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(format!("{} has no crate directory", spec.display())),
        _ => Err(format!("{} has more than one crate", spec.display())),
    }
}

fn write_pairs(dir: &Path, pages: &[(String, String)]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    for (name, body) in pages {
        fs::write(dir.join(name), body).map_err(|error| error.to_string())?;
    }
    Ok(())
}
