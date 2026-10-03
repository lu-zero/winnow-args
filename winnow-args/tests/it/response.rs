//! `@file` response files, with mold's and wild's cases.

use std::path::PathBuf;

use winnow_args::response::{ResponseErrorKind, ResponseFiles, expand};

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("winnow-args-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn at(path: &std::path::Path) -> String {
    format!("@{}", path.display())
}

fn strings(words: &[&winnow::stream::BStr]) -> Vec<String> {
    words
        .iter()
        .map(|w| String::from_utf8(w.to_vec()).unwrap())
        .collect()
}

#[test]
fn expands_in_place_and_recursively() {
    let dir = dir("nested");
    let inner = dir.join("inner.rsp");
    std::fs::write(&inner, "-lm '-L/with space'").unwrap();
    let outer = dir.join("outer.rsp");
    std::fs::write(&outer, format!("-o out {} main.o", at(&inner))).unwrap();

    let args = ["-shared".to_owned(), at(&outer), "-lc".to_owned()];
    let mut files = ResponseFiles::default();
    let words = expand(&args, &mut files).unwrap();
    assert_eq!(
        strings(&words),
        [
            "-shared",
            "-o",
            "out",
            "-lm",
            "-L/with space",
            "main.o",
            "-lc"
        ]
    );
}

#[test]
fn quoted_at_names_a_file_too() {
    // mold `response-file2`: the word is unquoted before `@` is looked at.
    let dir = dir("quoted");
    let inner = dir.join("in ner.rsp");
    std::fs::write(&inner, "a").unwrap();
    let outer = dir.join("outer.rsp");
    std::fs::write(&outer, format!("'@{}' b", inner.display())).unwrap();
    let args = [at(&outer)];
    let mut files = ResponseFiles::default();
    assert_eq!(strings(&expand(&args, &mut files).unwrap()), ["a", "b"]);
}

#[test]
fn nesting_is_capped() {
    let dir = dir("loop");
    let file = dir.join("self.rsp");
    std::fs::write(&file, at(&file)).unwrap();
    let args = [at(&file)];
    let mut files = ResponseFiles::default();
    let error = expand(&args, &mut files).unwrap_err();
    assert!(matches!(error.kind, ResponseErrorKind::TooDeep));
    assert!(
        error
            .to_string()
            .ends_with("response file nesting too deep")
    );
}

#[test]
fn errors_name_the_file() {
    let dir = dir("errors");
    let open = dir.join("open.rsp");
    std::fs::write(&open, "'unterminated").unwrap();
    let mut files = ResponseFiles::default();
    let args = [at(&open)];
    let error = expand(&args, &mut files).unwrap_err();
    assert_eq!(
        error.to_string(),
        format!("{}: premature end of input", open.display())
    );

    let missing = dir.join("missing.rsp");
    let args = [at(&missing)];
    let mut files = ResponseFiles::default();
    let error = expand(&args, &mut files).unwrap_err();
    assert!(matches!(error.kind, ResponseErrorKind::Io(_)));
}

#[cfg(feature = "derive")]
#[test]
fn parses_the_expanded_line() {
    use winnow_args::Args;

    #[derive(Args, Debug)]
    #[arg(long_only)]
    struct Ld {
        #[arg(short = 'o')]
        output: Option<String>,
        #[arg(long)]
        shared: bool,
        #[arg(positional)]
        inputs: Vec<String>,
    }

    let dir = dir("derive");
    let file = dir.join("link.rsp");
    std::fs::write(&file, "-shared -o 'a b.so' x.o").unwrap();
    let args = [at(&file), "y.o".to_owned()];
    let mut files = ResponseFiles::default();
    let words = expand(&args, &mut files).unwrap();
    let ld = Ld::parse_words(&words).unwrap();
    assert!(ld.shared);
    assert_eq!(ld.output.as_deref(), Some("a b.so"));
    assert_eq!(ld.inputs, ["x.o", "y.o"]);
}

#[test]
fn the_number_of_files_is_capped() {
    // Each file names the next one twice, ten deep: 2^10 leaves would be read
    // twice over without a cap on the total.
    let dir = dir("fanout");
    let name = |i: usize| dir.join(format!("f{i}.rsp"));
    for i in 0..12 {
        let next = at(&name(i + 1));
        let body = (0..8).map(|_| next.clone()).collect::<Vec<_>>().join(" ");
        std::fs::write(name(i), if i < 9 { body } else { "x".into() }).unwrap();
    }
    let args = [at(&name(0))];
    let mut files = ResponseFiles::default();
    let error = expand(&args, &mut files).unwrap_err();
    assert!(matches!(error.kind, ResponseErrorKind::TooMany), "{error}");
}
