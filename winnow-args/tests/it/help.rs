//! Help and version: rendered from doc comments and attributes, requested with
//! `-h`, `--help`, `help <command>`, `-V`, or a bare `arg_required_else_help` call.
//! The text is prose, so these need the `help-text` feature (see
//! `no_help_text.rs` for help without it).

use winnow_args::color::{Color, Depth, Paint, Palette, Theme};
use winnow_args::help::Style;
use winnow_args::{Args, Error, ErrorKind, Subcommand, ValueEnum, report, with_env};

/// Dev tools, env vars, and tasks in one CLI
///
/// mise prepares your development environment before each command runs.
#[derive(Args, Debug)]
#[arg(
    name = "mise",
    version = "2026.9.0",
    after_help = "Examples:\n    $ mise use -g node@20"
)]
struct Cli {
    /// Show extra output (repeat for more)
    #[arg(short, long, count, global)]
    verbose: u8,
    /// Change to this directory before executing the command
    #[arg(short = 'C', long, value_name = "DIR", env = "MISE_CD")]
    cd: Option<String>,
    /// Number of jobs to run in parallel
    ///
    /// Defaults to the number of CPUs.
    #[arg(short, long, default = "4", help_heading = "Performance")]
    jobs: Option<u32>,
    /// Internal
    #[arg(long, hide)]
    debug_internal: bool,
    #[arg(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Installs a tool and adds the version to mise.toml
    #[arg(alias = "u")]
    Use(UseArgs),
    /// Replaced by the `help` attribute in the listing.
    #[arg(
        name = "ls",
        alias = "list",
        alias_hidden = "l",
        help = "List installed tools"
    )]
    Ls,
}

#[derive(ValueEnum, Debug)]
enum When {
    Auto,
    Always,
    Never,
    /// Not for users.
    #[arg(hide)]
    Debug,
}

/// Installs a tool and adds the version to mise.toml.
#[derive(Args, Debug)]
#[arg(arg_required_else_help)]
struct UseArgs {
    /// Use the global config file
    #[arg(short, long)]
    global: bool,
    /// Output format
    #[arg(long, choices("json", "toml"))]
    format: Option<String>,
    /// When to use color
    #[arg(long)]
    color: Option<When>,
    /// Tool(s) to add
    #[arg(positional, value_name = "TOOL@VERSION")]
    tools: Vec<String>,
}

fn parse<T: Args>(line: &[&str]) -> Result<T, Error> {
    T::parse_from(line)
}

/// Help at the default width: `COLUMNS` unset whatever the test runner has.
fn help(line: &[&str]) -> String {
    help_at(line, &[])
}

fn help_at(line: &[&str], env: &[(&str, &str)]) -> String {
    let e = parse::<Cli>(line).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::HelpRequested, "{line:?}");
    with_env(env, || e.render_help("mise").unwrap())
}

#[test]
fn short_help_lists_what_is_visible() {
    assert_eq!(
        help(&["-h"]),
        "\
Dev tools, env vars, and tasks in one CLI

Usage: mise [OPTIONS] [COMMAND]

Commands:
  use, u    Installs a tool and adds the version to mise.toml
  ls, list  List installed tools

Options:
  -v, --verbose...  Show extra output (repeat for more)
  -C, --cd <DIR>    Change to this directory before executing the command [env: MISE_CD]
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version

Performance:
  -j, --jobs <JOBS>  Number of jobs to run in parallel [default: 4]

Examples:
    $ mise use -g node@20
"
    );
}

#[test]
fn long_help_has_the_whole_descriptions() {
    let long = help(&["--help"]);
    assert!(long.contains("mise prepares your development environment"));
    assert!(long.contains("          Defaults to the number of CPUs.\n          [default: 4]"));
    assert!(!long.contains("debug-internal"), "hidden flags stay hidden");
}

#[test]
fn a_subcommand_help_names_its_path() {
    let text = help(&["use", "-h"]);
    assert!(text.starts_with("Installs a tool and adds the version to mise.toml.\n\nUsage: mise use [OPTIONS] [TOOL@VERSION]...\n"));
    assert!(text.contains("--format <FORMAT>  Output format [possible values: json, toml]"));
    // A `ValueEnum`'s visible variants, without being declared again.
    assert!(
        text.contains(
            "--color <COLOR>    When to use color [possible values: auto, always, never]"
        ),
        "{text}"
    );
    // A bundle can ask for help too.
    assert!(help(&["-vh"]).starts_with("Dev tools"));
}

#[test]
fn the_help_word_follows_subcommand_names() {
    // Through an alias to the canonical name, and in the long form.
    assert!(help(&["help", "list"]).contains("Usage: mise ls [OPTIONS]"));
    assert!(help(&["help", "u"]).contains("Usage: mise use [OPTIONS]"));
    assert!(help(&["help"]).contains("mise prepares your development environment"));
}

#[test]
fn report_prints_and_chooses_the_exit_status() {
    assert_eq!(report(&parse::<Cli>(&["-h"]).unwrap_err(), "mise"), 0);
    let version = parse::<Cli>(&["-V"]).unwrap_err();
    assert_eq!(
        version.version_text("mise").as_deref(),
        Some("mise 2026.9.0")
    );
    assert_eq!(report(&version, "mise"), 0);
    // A bare `arg_required_else_help` call is help, but also a failure.
    let bare = parse::<Cli>(&["use"]).unwrap_err();
    assert!(bare.is_bare_help());
    assert_eq!(report(&bare, "mise"), 2);
    assert_eq!(report(&parse::<Cli>(&["--nope"]).unwrap_err(), "mise"), 2);
}

#[derive(Args, Debug)]
struct OwnsH {
    /// Host to connect to
    #[arg(short = 'h', long)]
    host: Option<String>,
}

#[derive(Args, Debug)]
#[arg(disable_help_flag)]
struct NoHelp {}

#[test]
fn a_declared_or_disabled_help_flag_is_not_supplied() {
    // `-h` is the CLI's own; `--help` is still supplied.
    assert_eq!(
        parse::<OwnsH>(&["-h", "x"]).unwrap().host.as_deref(),
        Some("x")
    );
    assert_eq!(
        parse::<OwnsH>(&["--help"]).unwrap_err().kind(),
        ErrorKind::HelpRequested
    );
    assert_eq!(
        parse::<NoHelp>(&["--help"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
}

#[test]
fn descriptions_wrap_under_their_column() {
    assert_eq!(
        help_at(&["-h"], &[("COLUMNS", "60")]),
        "\
Dev tools, env vars, and tasks in one CLI

Usage: mise [OPTIONS] [COMMAND]

Commands:
  use, u    Installs a tool and adds the version to
            mise.toml
  ls, list  List installed tools

Options:
  -v, --verbose...  Show extra output (repeat for more)
  -C, --cd <DIR>    Change to this directory before
                    executing the command [env: MISE_CD]
  -h, --help        Print help (see more with '--help')
  -V, --version     Print version

Performance:
  -j, --jobs <JOBS>  Number of jobs to run in parallel
                     [default: 4]

Examples:
    $ mise use -g node@20
"
    );
}

#[test]
fn a_narrow_page_puts_descriptions_under_wide_items() {
    // At 40 columns the column is at most 14 wide: `-C, --cd <DIR>` fits,
    // `-v, --verbose...` goes to its own line.
    let text = help_at(&["-h"], &[("COLUMNS", "40")]);
    assert!(
        text.contains("  -v, --verbose...\n                  Show extra output\n"),
        "{text}"
    );
    for line in text.lines() {
        assert!(
            line.chars().count() <= 40 || !line.contains(' '),
            "{line:?}"
        );
    }
}

#[test]
fn long_help_wraps_its_indented_blocks() {
    let text = help_at(&["--help"], &[("COLUMNS", "50")]);
    assert!(
        text.contains(
            "          Change to this directory before\n          executing the command\n"
        ),
        "{text}"
    );
}

/// `text` without its SGR escapes.
fn strip(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('\u{1b}') {
        out.push_str(&rest[..at]);
        let end = rest[at..].find('m').expect("an SGR sequence ends in `m`");
        rest = &rest[at + end + 1..];
    }
    out + rest
}

fn styled_as(style: Style, line: &[&str], env: &[(&str, &str)]) -> String {
    let e = parse::<Cli>(line).unwrap_err();
    with_env(env, || e.render_help_styled("mise", style).unwrap())
}

fn styled(line: &[&str], env: &[(&str, &str)]) -> String {
    styled_as(Style::CLAP, line, env)
}

#[test]
fn color_paints_without_moving_anything() {
    for line in [&["-h"][..], &["--help"], &["use", "-h"]] {
        for (columns, style) in [
            ("100", Style::COLORED),
            ("50", Style::COLORED),
            ("50", Style::CLAP),
        ] {
            let env = [("COLUMNS", columns)];
            let colored = styled_as(style, line, &env);
            assert_ne!(colored, help_at(line, &env), "{line:?} is painted");
            assert_eq!(
                strip(&colored),
                help_at(line, &env),
                "{line:?} at {columns}"
            );
        }
    }
}

/// `text` in the SGR parameters `sgr`, reset after.
fn sgr(sgr: &str, text: &str) -> String {
    format!("\u{1b}[{sgr}m{text}\u{1b}[0m")
}

#[test]
fn color_defaults_to_greens_and_cyans() {
    // 16 colors: bold cyan headings, bold green flags and commands, cyan value
    // names, dim brackets and labels.
    let dim = |t| sgr("2", t);
    let text = styled_as(Style::COLORED, &["-h"], &[]);
    // The program stays plain; in `[NAME]` only the name is colored.
    let usage = format!(
        "{} mise {}{}{} {}{}{}",
        sgr("1;36", "Usage:"),
        dim("["),
        sgr("36", "OPTIONS"),
        dim("]"),
        dim("["),
        sgr("36", "COMMAND"),
        dim("]"),
    );
    assert!(text.contains(&usage), "{text:?}");
    assert!(
        text.contains(&format!("{}\n", sgr("1;36", "Options:"))),
        "{text:?}"
    );
    let cd = format!(
        "  {}, {} {}",
        sgr("1;32", "-C"),
        sgr("1;32", "--cd"),
        sgr("36", "<DIR>")
    );
    assert!(text.contains(&cd), "{text:?}");
    let verbose = format!(
        "{}, {}{}",
        sgr("1;32", "-v"),
        sgr("1;32", "--verbose"),
        dim("...")
    );
    assert!(text.contains(&verbose), "{text:?}");
    let commands = format!("  {}, {}    Installs", sgr("1;32", "use"), sgr("1;32", "u"));
    assert!(text.contains(&commands), "{text:?}");
    // Annotations: dim labels, the environment variable cyan, the default
    // green, possible values bright green.
    let env = format!("{}{}{}", dim("[env: "), sgr("36", "MISE_CD"), dim("]"));
    assert!(text.contains(&env), "{text:?}");
    let default = format!("{}{}{}", dim("[default: "), sgr("32", "4"), dim("]"));
    assert!(text.contains(&default), "{text:?}");
    let text = styled_as(Style::COLORED, &["use", "-h"], &[]);
    let tools = format!(
        "  {}{}{}{}  Tool(s) to add",
        dim("["),
        sgr("36", "TOOL@VERSION"),
        dim("]"),
        dim("...")
    );
    assert!(text.contains(&tools), "{text:?}");
    let choices = format!(
        "{}{}, {}{}",
        dim("[possible values: "),
        sgr("92", "json"),
        sgr("92", "toml"),
        dim("]")
    );
    assert!(text.contains(&choices), "{text:?}");

    // 256 colors: teal 73, green 71, slate teal 109, gray 245 frames.
    let at256 = Style::at(Palette::DEFAULT_256, Depth::Ansi256);
    let text = styled_as(at256, &["-h"], &[]);
    let usage = format!(
        "{} mise {}{}{}",
        sgr("1;38;5;73", "Usage:"),
        sgr("38;5;245", "["),
        sgr("38;5;109", "OPTIONS"),
        sgr("38;5;245", "]")
    );
    assert!(text.contains(&usage), "{text:?}");
    let cd = format!("{} {}", sgr("1;38;5;71", "--cd"), sgr("38;5;109", "<DIR>"));
    assert!(text.contains(&cd), "{text:?}");
    let env = format!("{}{}", sgr("38;5;245", "[env: "), sgr("38;5;37", "MISE_CD"));
    assert!(text.contains(&env), "{text:?}");
    let default = format!("{}{}", sgr("38;5;245", "[default: "), sgr("38;5;72", "4"));
    assert!(text.contains(&default), "{text:?}");
    let text = styled_as(at256, &["use", "-h"], &[]);
    assert!(text.contains(&sgr("38;5;71", "auto")), "{text:?}");

    // clap leaves annotations plain.
    let text = styled_as(Style::CLAP, &["use", "-h"], &[]);
    assert!(text.contains("[possible values: json, toml]"), "{text:?}");

    // No yellow anywhere in either default.
    for palette in [Palette::DEFAULT, Palette::DEFAULT_256] {
        let e = parse::<Cli>(&["--fore"]).unwrap_err();
        let text = e.render(Style::at(palette, palette.depth()))
            + &styled_as(Style::at(palette, palette.depth()), &["--help"], &[]);
        assert!(!text.contains("33m") && !text.contains(";136m"), "{text:?}");
    }
}

#[test]
fn quoted_spans_in_descriptions_are_code() {
    let text = styled_as(Style::COLORED, &["use", "-h"], &[]);
    // "Installs a tool and adds the version to mise.toml." has none; the
    // `help` example's "as `name@version`" is checked where it is written.
    assert!(!text.contains("\u{1b}[1m`"), "{text:?}");
    #[derive(Args, Debug)]
    struct Quoted {
        /// Tools as `name@version`, or `name` alone, `unpaired
        #[arg(positional)]
        tools: Vec<String>,
    }
    let e = parse::<Quoted>(&["-h"]).unwrap_err();
    let text = with_env(&[], || e.render_help_styled("q", Style::COLORED).unwrap());
    let quoted = format!(
        "Tools as {}, or {} alone, `unpaired",
        sgr("1", "`name@version`"),
        sgr("1", "`name`")
    );
    assert!(text.contains(&quoted), "{text:?}");
    let plain = with_env(&[], || e.render_help("q").unwrap());
    assert!(plain.contains("Tools as `name@version`, or `name` alone, `unpaired"));
}

#[test]
fn color_uses_clap_codes() {
    // clap 4's default styles for the bench CLI (bold and underline in help,
    // color only in errors), with each paint written as one
    // sequence (`1;4` where clap writes `1` then `4`): the same on screen.
    let text = styled(&["-h"], &[]);
    assert!(
        text.contains("\u{1b}[1;4mUsage:\u{1b}[0m \u{1b}[1mmise\u{1b}[0m [OPTIONS] [COMMAND]"),
        "{text:?}"
    );
    assert!(text.contains("\u{1b}[1;4mOptions:\u{1b}[0m\n"), "{text:?}");
    assert!(
        text.contains("  \u{1b}[1m-v\u{1b}[0m, \u{1b}[1m--verbose\u{1b}[0m...  Show extra output"),
        "{text:?}"
    );
    assert!(
        text.contains("  \u{1b}[1muse\u{1b}[0m, \u{1b}[1mu\u{1b}[0m    Installs"),
        "{text:?}"
    );
    // Placeholders stay plain, as in clap.
    assert!(text.contains("\u{1b}[1m--cd\u{1b}[0m <DIR>"), "{text:?}");
}

#[test]
fn color_follows_the_environment() {
    let auto = |env: &[(&str, &str)], terminal| with_env(env, || Style::auto_for(terminal));
    // Under `with_env` a terminal does not count: only the variables decide.
    assert_eq!(auto(&[], true), Style::PLAIN);
    assert_eq!(auto(&[("CLICOLOR_FORCE", "1")], false), Style::COLORED);
    assert_eq!(auto(&[("CLICOLOR_FORCE", "0")], false), Style::PLAIN);
    assert_eq!(
        auto(&[("CLICOLOR_FORCE", "1"), ("NO_COLOR", "1")], true),
        Style::PLAIN
    );
    // An empty `NO_COLOR` refuses nothing.
    assert_eq!(
        auto(&[("CLICOLOR_FORCE", "1"), ("NO_COLOR", "")], false),
        Style::COLORED
    );
}

#[test]
fn errors_paint_what_was_typed() {
    let e = parse::<Cli>(&["--fore"]).unwrap_err();
    assert_eq!(
        e.render(Style::CLAP),
        "\u{1b}[1;31merror:\u{1b}[0m unknown flag `\u{1b}[33m--fore\u{1b}[0m`\n\n\
         For more information, try '\u{1b}[1m--help\u{1b}[0m'."
    );
    assert_eq!(
        e.render(Style::PLAIN),
        format!("error: {e}\n\nFor more information, try '--help'.")
    );
}

#[test]
fn depth_follows_the_terminal_and_the_environment() {
    let depth = |env: &[(&str, &str)]| with_env(env, || Depth::detect(true));
    let forced = |env: &[(&str, &str)]| {
        let mut all = vec![("CLICOLOR_FORCE", "1")];
        all.extend_from_slice(env);
        with_env(&all, || Depth::detect(false))
    };
    // Under `with_env` nothing is a terminal: only forcing gives color.
    assert_eq!(depth(&[("TERM", "xterm")]), Depth::None);
    assert_eq!(forced(&[]), Depth::Ansi16);
    assert_eq!(forced(&[("TERM", "xterm-256color")]), Depth::Ansi256);
    assert_eq!(forced(&[("TERM", "screen-256")]), Depth::Ansi256);
    assert_eq!(forced(&[("COLORTERM", "truecolor")]), Depth::TrueColor);
    assert_eq!(forced(&[("COLORTERM", "24bit")]), Depth::TrueColor);
    assert_eq!(forced(&[("TERM", "xterm-direct")]), Depth::TrueColor);
    assert_eq!(forced(&[("TERM_PROGRAM", "iTerm.app")]), Depth::TrueColor);
    assert_eq!(
        forced(&[("TERM_PROGRAM", "Apple_Terminal")]),
        Depth::Ansi256
    );
    // `NO_COLOR` wins over everything; an empty one says nothing.
    assert_eq!(
        forced(&[("NO_COLOR", "1"), ("COLORTERM", "truecolor")]),
        Depth::None
    );
    assert_eq!(forced(&[("NO_COLOR", "")]), Depth::Ansi16);
    // `FORCE_COLOR` levels, as supports-color reads them.
    let force = |level| with_env(&[("FORCE_COLOR", level)], || Depth::detect(false));
    assert_eq!(force(""), Depth::Ansi16);
    assert_eq!(force("true"), Depth::Ansi16);
    assert_eq!(force("2"), Depth::Ansi256);
    assert_eq!(force("3"), Depth::TrueColor);
    assert_eq!(force("0"), Depth::None);
    assert_eq!(force("false"), Depth::None);
    // A forced level is a floor: the terminal may say more.
    assert_eq!(
        with_env(&[("FORCE_COLOR", "1"), ("COLORTERM", "truecolor")], || {
            Depth::detect(false)
        }),
        Depth::TrueColor
    );
}

#[test]
fn a_rich_theme_falls_back_by_depth() {
    const ORANGE: Color = Color::Rgb(255, 135, 0);
    let rich = Palette {
        header: Paint::fg(ORANGE).bold(),
        ..Palette::DEFAULT
    };
    let theme = Theme {
        truecolor: Some(rich),
        ..Theme::DEFAULT
    };
    let usage = |depth| {
        let e = parse::<Cli>(&["-h"]).unwrap_err();
        let style = Style::at(theme.palette(depth), depth);
        let text = with_env(&[], || e.render_help_styled("mise", style).unwrap());
        text.lines()
            .find(|l| l.contains("Usage:"))
            .unwrap()
            .to_owned()
    };
    assert!(usage(Depth::TrueColor).starts_with("\u{1b}[1;38;2;255;135;0mUsage:\u{1b}[0m"));
    // Below 24-bit the theme's own palettes are used as they are.
    assert!(usage(Depth::Ansi256).starts_with("\u{1b}[1;38;5;73mUsage:\u{1b}[0m"));
    assert!(usage(Depth::Ansi16).starts_with("\u{1b}[1;36mUsage:\u{1b}[0m"));
    assert_eq!(usage(Depth::None), "Usage: mise [OPTIONS] [COMMAND]");
    // A palette used below its depth maps each color to the nearest one.
    let e = parse::<Cli>(&["-h"]).unwrap_err();
    let text = with_env(&[], || {
        e.render_help_styled("mise", Style::at(rich, Depth::Ansi256))
            .unwrap()
    });
    assert!(text.contains("\u{1b}[1;38;5;208mUsage:"), "{text:?}");
    let text = with_env(&[], || {
        e.render_help_styled("mise", Style::at(rich, Depth::Ansi16))
            .unwrap()
    });
    assert!(text.contains("\u{1b}[1;33mUsage:"), "{text:?}");
}

/// bash's builtins: help is `--help`, and `-h` is an unknown flag.
#[derive(Args, Debug)]
#[arg(disable_help_short)]
struct LongHelpOnly {
    /// Print physical directory.
    #[arg(short = 'P')]
    physical: bool,
}

#[test]
fn disable_help_short_keeps_only_the_long_help_flag() {
    assert!(parse::<LongHelpOnly>(&["-P"]).unwrap().physical);
    assert_eq!(
        parse::<LongHelpOnly>(&["-h"]).unwrap_err().kind(),
        ErrorKind::UnknownFlag
    );
    let e = parse::<LongHelpOnly>(&["--help"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::HelpRequested);
    let text = e.render_help("pwd").unwrap();
    assert!(
        text.contains("      --help\n          Print help\n"),
        "{text}"
    );
    assert!(!text.contains("-h,"), "{text}");
}

/// Every spelling and value form a flag can have, as help spells them.
#[derive(Args, Debug)]
#[arg(plus_options, disable_help_flag)]
struct Spellings {
    #[arg(short = 'e', plus = 'e')]
    errexit: Option<bool>,
    #[arg(short = 'o', value_name = "OPT")]
    enable: Vec<String>,
    #[arg(plus = 'o', value_name = "OPT")]
    disable: Vec<String>,
    #[arg(short = 'l', short = 'L')]
    list: bool,
    #[arg(long, require_equals, value_name = "PORT")]
    inspect: Option<u16>,
    #[arg(long, value_name = "WHEN", default_missing = "auto")]
    pager: Option<String>,
    #[arg(long, require_equals, value_name = "KIND", default_missing = "fast")]
    build_id: Option<String>,
    #[arg(long, values = 3, value_name = "V")]
    platform: Vec<String>,
}

#[test]
fn help_spells_every_form_of_a_flag() {
    let text = winnow_args::help::render(Spellings::HELP, &["x"], false);
    for row in [
        "  -e, +e",
        "  -o <OPT>...",
        "  +o <OPT>...",
        "  -l, -L",
        "      --inspect=<PORT>",
        "      --pager [<WHEN>]",
        "      --build-id[=<KIND>]",
        "      --platform <V> <V> <V>...",
    ] {
        assert!(
            text.lines().any(|line| line.trim_end() == row),
            "{row:?} in\n{text}"
        );
    }
}

#[test]
fn auto_styles_follow_the_environment_of_their_stream() {
    // Under `with_env` neither stream counts as a terminal: plain unless forced.
    assert_eq!(with_env(&[], Style::auto), Style::PLAIN);
    assert_eq!(with_env(&[], Style::auto_stderr), Style::PLAIN);
    let forced = [("FORCE_COLOR", "1")];
    let colored = with_env(&forced, || Style::auto_for(false));
    assert_ne!(colored, Style::PLAIN);
    assert_eq!(with_env(&forced, Style::auto), colored);
    assert_eq!(with_env(&forced, Style::auto_stderr), colored);
}
