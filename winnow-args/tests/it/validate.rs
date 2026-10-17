//! `validate = function`: the program's own check of a parsed struct.

use winnow_args::help::Style;
use winnow_args::{Args, Error, ErrorKind, Subcommand};

#[derive(Args, Debug)]
#[arg(validate = check_tool)]
struct Tool {
    #[arg(long, global)]
    root: Option<String>,
    #[arg(flatten)]
    range: Range,
    #[arg(subcommand)]
    command: Command,
}

/// The root sees the subcommand that was selected.
fn check_tool(tool: &Tool) -> Result<(), Error> {
    match (&tool.root, &tool.command) {
        (Some(root), Command::Crossdev) => Err(Error::invalid_value(
            0,
            "--root",
            root.as_bytes(),
            "not valid with `crossdev`",
        )),
        _ => Ok(()),
    }
}

#[derive(Args, Debug)]
#[arg(validate = check_range)]
struct Range {
    #[arg(long, default = "0")]
    from: u32,
    #[arg(long, default = "10")]
    to: u32,
}

fn check_range(range: &Range) -> Result<(), Error> {
    if range.from > range.to {
        return Err(Error::invalid("`--from` is after `--to`"));
    }
    Ok(())
}

#[derive(Subcommand, Debug)]
enum Command {
    Crossdev,
    /// A variant with fields takes the option as a struct does.
    #[arg(validate = check_build)]
    Build {
        #[arg(long)]
        jobs: Option<u32>,
        #[arg(long)]
        serial: bool,
    },
}

/// A variant's function is given the enum.
fn check_build(command: &Command) -> Result<(), Error> {
    match command {
        Command::Build {
            jobs: Some(_),
            serial: true,
        } => Err(Error::invalid(
            "`--serial` runs one job: leave `--jobs` out",
        )),
        _ => Ok(()),
    }
}

#[test]
fn a_struct_that_passes_is_returned() {
    let tool = Tool::parse_from(["--from", "2", "build", "--jobs", "4"]).unwrap();
    assert_eq!((tool.range.from, tool.range.to), (2, 10));
}

#[test]
fn the_root_sees_the_selected_subcommand() {
    let error = Tool::parse_from(["--root", "/r", "crossdev"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(
        error.to_string(),
        "invalid value `/r` for `--root`: not valid with `crossdev`"
    );
    // A global flag given after the subcommand is the root's all the same.
    let error = Tool::parse_from(["crossdev", "--root", "/r"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert!(Tool::parse_from(["--root", "/r", "build"]).is_ok());
}

#[test]
fn a_flattened_struct_checks_itself() {
    let error = Tool::parse_from(["--from", "5", "--to", "1", "build"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Invalid);
    assert_eq!(
        error.render(Style::PLAIN),
        "error: `--from` is after `--to`\n\nFor more information, try '--help'."
    );
}

#[test]
fn a_variant_with_fields_is_checked_as_the_enum() {
    let error = Tool::parse_from(["build", "--jobs", "2", "--serial"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Invalid);
    assert!(Tool::parse_from(["build", "--serial"]).is_ok());
}

#[test]
fn help_is_not_checked() {
    let error = Tool::parse_from(["--from", "5", "--to", "1", "--help"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::HelpRequested);
}

/// A `keywords` vocabulary's check is reported as it was written.
mod keywords {
    use winnow_args::{Args, Error, ErrorKind};

    #[derive(Args, Debug)]
    #[arg(validate = check)]
    struct Words {
        #[arg(long)]
        now: bool,
        #[arg(long)]
        lazy: bool,
    }

    fn check(words: &Words) -> Result<(), Error> {
        if words.now && words.lazy {
            return Err(Error::invalid("`now` and `lazy` exclude each other"));
        }
        Ok(())
    }

    #[derive(Args, Debug)]
    struct Ld {
        #[arg(short = 'z', keywords)]
        z: Words,
    }

    #[test]
    fn its_error_is_kept() {
        let error = Ld::parse_from(["-z", "now", "-z", "lazy"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert_eq!(error.to_string(), "`now` and `lazy` exclude each other");
    }
}
