//! `multicall`: run under a subcommand's name, the program is that subcommand.

use std::ffi::OsString;

use winnow_args::complete::Shell;
use winnow_args::{Args, ErrorKind, Subcommand};

#[derive(Args, Debug, PartialEq)]
struct Ls {
    #[arg(short)]
    long: bool,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Applet {
    Ls(Ls),
    #[arg(alias = "dir")]
    Echo {
        #[arg(positional)]
        words: Vec<String>,
    },
}

#[derive(Args, Debug, PartialEq)]
#[arg(multicall, name = "busybox")]
struct Busybox {
    #[arg(subcommand)]
    applet: Applet,
}

#[test]
fn the_program_s_name_selects_the_subcommand() {
    let ls = Applet::Ls(Ls { long: true });
    assert_eq!(
        Busybox::parse_from_argv(["/bin/ls", "-l"]).unwrap().applet,
        ls
    );
    assert_eq!(Busybox::parse_from_argv(["ls", "-l"]).unwrap().applet, ls);
    let echo = Applet::Echo {
        words: vec!["a".into()],
    };
    assert_eq!(Busybox::parse_from_argv(["dir", "a"]).unwrap().applet, echo);
}

#[test]
fn under_another_name_the_line_is_read_as_usual() {
    let ls = Applet::Ls(Ls { long: true });
    let parsed = Busybox::parse_from_argv(["/usr/bin/busybox", "ls", "-l"]).unwrap();
    assert_eq!(parsed.applet, ls);
    let e = Busybox::parse_from_argv(["busybox"]).unwrap_err();
    assert_eq!(e.kind(), ErrorKind::MissingSubcommand);
    // Without `multicall`, the name is skipped whatever it is.
    #[derive(Args, Debug)]
    struct Plain {
        #[arg(subcommand)]
        applet: Option<Applet>,
    }
    assert_eq!(Plain::parse_from_argv(["ls"]).unwrap().applet, None);
}

#[test]
fn offsets_count_from_the_first_word() {
    let e = Busybox::parse_from_argv(["ls", "-z"]).unwrap_err();
    assert_eq!((e.kind(), e.offset()), (ErrorKind::UnknownFlag, 0));
    let e = Busybox::parse_from_argv(["dir", "a", "-z"]).unwrap_err();
    assert_eq!(e.offset(), 2);
    let e = Busybox::parse_from_argv(["busybox", "ls", "-z"]).unwrap_err();
    assert_eq!(e.offset(), 3);
}

#[test]
fn an_enum_as_the_whole_line() {
    #[derive(Subcommand, Debug, PartialEq)]
    #[arg(multicall)]
    enum Tools {
        Ls(Ls),
    }
    let ls = Tools::Ls(Ls { long: true });
    assert_eq!(Tools::parse_from_argv(["ls", "-l"]).unwrap(), ls);
    assert_eq!(Tools::parse_from_argv(["tools", "ls", "-l"]).unwrap(), ls);
}

#[cfg(feature = "help-text")]
#[test]
fn help_names_the_subcommand_alone() {
    let e = Busybox::parse_from_argv(["ls", "--help"]).unwrap_err();
    let help = e.render_help("ls").unwrap();
    assert!(help.contains("Usage: ls [OPTIONS]"), "{help}");
    // By an alias, help names the alias.
    let e = Busybox::parse_from_argv(["dir", "--help"]).unwrap_err();
    let help = e.render_help("dir").unwrap();
    assert!(help.contains("Usage: dir [OPTIONS]"), "{help}");
    let e = Busybox::parse_from_argv(["busybox", "ls", "--help"]).unwrap_err();
    let help = e.render_help("busybox").unwrap();
    assert!(help.contains("Usage: busybox ls [OPTIONS]"), "{help}");
}

/// What a completion callback answers, for `line` typed up to the cursor.
fn complete(line: &str) -> String {
    let args: Vec<OsString> = ["__complete_word__", "--shell", "fish", "--line", line]
        .into_iter()
        .map(OsString::from)
        .collect();
    Busybox::completion_request(&args).unwrap()
}

#[test]
fn names_a_script_cannot_hold_are_left_out() {
    #[derive(Subcommand)]
    enum Tool {
        #[arg(name = "[", alias = "test")]
        Test,
        #[arg(name = "a-b")]
        Dash,
        #[arg(name = "a_b")]
        Underscore,
    }
    #[derive(Args)]
    #[arg(multicall, name = "box")]
    struct Box {
        #[arg(subcommand)]
        tool: Tool,
    }
    let _ = |b: Box| b.tool;
    let bash = Box::completion_script(Shell::Bash);
    assert!(!bash.contains(" [\n") && bash.contains("_box_test_complete test\n"));
    // `a-b` and `a_b` are one function name apart.
    assert!(bash.contains("_box_a_b_complete a-b\n"), "{bash}");
    assert!(bash.contains("_box_a_b2_complete a_b\n"), "{bash}");
}

#[test]
fn completion_follows_the_name_typed() {
    assert!(complete("ls -").contains("-l"), "{}", complete("ls -"));
    assert!(complete("/bin/ls -").contains("-l"));
    assert!(complete("busybox ls -").contains("-l"));
    let root = complete("busybox ");
    assert!(root.contains("ls") && root.contains("echo"), "{root}");
    assert!(!complete("busybox -").contains("-l"));
}

#[test]
fn the_script_completes_each_name() {
    let bash = Busybox::completion_script(Shell::Bash);
    // Each name asks busybox: an `ls` not linked to it never runs.
    assert_eq!(bash.matches("command busybox __complete_word__").count(), 4);
    assert!(!bash.contains("command ls"), "{bash}");
    for (name, function) in [
        ("busybox", "busybox"),
        ("ls", "busybox_ls"),
        ("echo", "busybox_echo"),
        ("dir", "busybox_dir"),
    ] {
        let line = format!("complete -F _{function}_complete {name}\n");
        assert!(bash.contains(&line), "{name}: {bash}");
    }
    let zsh = Busybox::completion_script(Shell::Zsh);
    assert!(zsh.starts_with("#compdef busybox ls echo dir\n"), "{zsh}");
    assert_eq!(zsh.matches("#compdef").count(), 1);
    // zsh's own `_ls` is left alone.
    assert!(!zsh.contains("\n_ls()"), "{zsh}");
    assert!(zsh.contains("        ls) _busybox_ls \"$@\" ;;\n"), "{zsh}");
    assert!(zsh.contains("\ncompdef _busybox_dir dir\n"), "{zsh}");
    // Without `multicall`, the one name.
    #[derive(Args)]
    #[arg(name = "busybox")]
    struct Plain {
        #[arg(subcommand)]
        applet: Applet,
    }
    let _ = |p: Plain| p.applet;
    let plain = Plain::completion_script(Shell::Zsh);
    assert!(plain.starts_with("#compdef busybox\n"), "{plain}");
    assert!(plain.contains("then\n    _busybox \"$@\"\nelse\n    compdef _busybox busybox\nfi\n"));
}
