//! Rendering a failure with the command line and a caret.

use winnow_args::{Args, BStr, help::Style};

#[derive(Args, Debug)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

#[test]
fn a_caret_marks_the_byte_the_error_is_about() {
    let words = [BStr::new("-x"), BStr::new("file")];
    let error = Cli::parse_words(&words).unwrap_err();
    assert_eq!(
        error.render_with_line("tool", &words, Style::PLAIN),
        "error: unknown flag `-x`\n\ntool -x file\n     ^\n\nFor more information, try '--help'."
    );

    let words = [BStr::new("-v"), BStr::new("--bad")];
    let error = Cli::parse_words(&words).unwrap_err();
    assert_eq!(
        error.render_with_line("tool", &words, Style::PLAIN),
        "error: unknown flag `--bad`\n\ntool -v --bad\n        ^\n\nFor more information, try '--help'."
    );
}

#[test]
fn the_caret_counts_characters_not_bytes() {
    let words = [BStr::new("--bad")];
    let error = Cli::parse_words(&words).unwrap_err();
    assert_eq!(
        error.render_with_line("tóol", &words, Style::PLAIN),
        "error: unknown flag `--bad`\n\ntóol --bad\n     ^\n\nFor more information, try '--help'."
    );
}
