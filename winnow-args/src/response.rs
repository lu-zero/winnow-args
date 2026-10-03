//! Response files: `@file` words that stand for the file's contents.
//!
//! A linker's command line is often too long for the kernel, so compilers pass
//! it as `@file`: the file holds the words, separated by whitespace, with
//! `'…'` and `"…"` quoting and `\` escaping one byte, as GNU's `buildargv`
//! reads them. A word in a file may itself be `@file`, up to [`MAX_DEPTH`]
//! files deep.
//!
//! [`expand`] reads the files into a [`ResponseFiles`] that owns their bytes,
//! and returns the whole command line as words that borrow from it and from
//! the arguments, ready for [`Argv::new`](crate::Argv::new). Only a word that
//! had quotes or backslashes removed is copied.
//!
//! ```
//! use winnow_args::response::{ResponseFiles, expand};
//!
//! let dir = std::env::temp_dir().join("winnow-args-response-doc");
//! std::fs::create_dir_all(&dir).unwrap();
//! let file = dir.join("link.rsp");
//! std::fs::write(&file, "-o 'my out' main.o\n").unwrap();
//! let at = format!("@{}", file.display());
//!
//! let args = ["-shared", at.as_str(), "-lc"];
//! let mut files = ResponseFiles::default();
//! let words = expand(&args, &mut files).unwrap();
//! let words: Vec<&[u8]> = words.iter().map(|w| w.as_ref()).collect();
//! assert_eq!(words, [&b"-shared"[..], b"-o", b"my out", b"main.o", b"-lc"]);
//! ```
//!
//! A program expands its arguments, then parses the words with
//! [`Args::parse_words`](crate::Args::parse_words) where it would call `parse()`;
//! nothing turns `@file` on but that call. `examples/ld.rs` does it.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

use winnow::stream::BStr;

/// How many response files deep `@file` may nest: a file named on the command
/// line is at depth 1. mold's limit.
pub const MAX_DEPTH: usize = 10;

/// How many response files one command line may read in all: nesting fans
/// out, and a file that names several others ten deep would never end.
pub const MAX_FILES: usize = 4096;

/// The storage [`expand`]'s words borrow from: the files' contents and the
/// words that had to be unquoted.
#[derive(Default, Debug)]
pub struct ResponseFiles {
    files: Vec<Vec<u8>>,
    owned: Vec<Vec<u8>>,
}

/// Why a response file could not be read.
#[derive(Debug)]
pub struct ResponseError {
    /// The file, as it was named.
    pub path: PathBuf,
    /// What went wrong.
    pub kind: ResponseErrorKind,
}

/// What went wrong reading a response file.
#[derive(Debug)]
#[non_exhaustive]
pub enum ResponseErrorKind {
    /// The file could not be read.
    Io(std::io::Error),
    /// A quote was not closed, or the file ends in a lone `\`.
    PrematureEnd,
    /// `@file` nested deeper than [`MAX_DEPTH`].
    TooDeep,
    /// More than [`MAX_FILES`] response files.
    TooMany,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.display();
        match &self.kind {
            ResponseErrorKind::Io(error) => write!(f, "{path}: {error}"),
            ResponseErrorKind::PrematureEnd => write!(f, "{path}: premature end of input"),
            ResponseErrorKind::TooDeep => write!(f, "{path}: response file nesting too deep"),
            ResponseErrorKind::TooMany => write!(f, "{path}: too many response files"),
        }
    }
}

impl std::error::Error for ResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ResponseErrorKind::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// A word of the expanded command line, before the storage is borrowed.
#[derive(Clone, Copy)]
enum Word {
    Arg(usize),
    File {
        file: usize,
        start: usize,
        end: usize,
    },
    Owned(usize),
}

/// Replace every `@file` argument with the words of the file, recursively.
///
/// Every other argument is kept as it is. Inside a file, a word that starts
/// with `@` once unquoted names a file too, as in GNU ld and mold.
pub fn expand<'a, S: AsRef<OsStr>>(
    args: &'a [S],
    files: &'a mut ResponseFiles,
) -> Result<Vec<&'a BStr>, ResponseError> {
    let mut words = Vec::with_capacity(args.len());
    for (index, arg) in args.iter().enumerate() {
        match arg.as_ref().as_encoded_bytes().strip_prefix(b"@") {
            Some(path) => read(files, path_of(path), 1, &mut words)?,
            None => words.push(Word::Arg(index)),
        }
    }
    let files: &'a ResponseFiles = files;
    Ok(words
        .into_iter()
        .map(|word| {
            BStr::new(match word {
                Word::Arg(index) => args[index].as_ref().as_encoded_bytes(),
                Word::File { file, start, end } => &files.files[file][start..end],
                Word::Owned(index) => &files.owned[index],
            })
        })
        .collect())
}

fn read(
    files: &mut ResponseFiles,
    path: &Path,
    depth: usize,
    words: &mut Vec<Word>,
) -> Result<(), ResponseError> {
    let error = |kind| ResponseError {
        path: path.to_owned(),
        kind,
    };
    if depth > MAX_DEPTH {
        return Err(error(ResponseErrorKind::TooDeep));
    }
    if files.files.len() >= MAX_FILES {
        return Err(error(ResponseErrorKind::TooMany));
    }
    let data = std::fs::read(path).map_err(|e| error(ResponseErrorKind::Io(e)))?;
    let file = files.files.len();
    let tokens = tokenize(&data).ok_or_else(|| error(ResponseErrorKind::PrematureEnd))?;
    files.files.push(data);

    for token in tokens {
        let bytes = match &token {
            Token::Range(start, end) => &files.files[file][*start..*end],
            Token::Owned(bytes) => bytes,
        };
        if let Some(nested) = bytes.strip_prefix(b"@") {
            let nested = path_of(nested).to_owned();
            read(files, &nested, depth + 1, words)?;
            continue;
        }
        words.push(match token {
            Token::Range(start, end) => Word::File { file, start, end },
            Token::Owned(bytes) => {
                files.owned.push(bytes);
                Word::Owned(files.owned.len() - 1)
            }
        });
    }
    Ok(())
}

#[cfg(unix)]
fn path_of(bytes: &[u8]) -> &Path {
    use std::os::unix::ffi::OsStrExt;
    Path::new(OsStr::from_bytes(bytes))
}

#[cfg(not(unix))]
fn path_of(bytes: &[u8]) -> &Path {
    // Outside Unix a path must be valid UTF-8 to be named in a response file.
    Path::new(std::str::from_utf8(bytes).unwrap_or_default())
}

enum Token {
    Range(usize, usize),
    Owned(Vec<u8>),
}

fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// Split a response file into words: ranges of `data` where nothing was
/// unquoted, copies where something was. `None` on an unclosed quote or a
/// trailing `\`.
fn tokenize(data: &[u8]) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if is_space(data[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < data.len() && !is_space(data[i]) && !matches!(data[i], b'\\' | b'\'' | b'"') {
            i += 1;
        }
        if i == data.len() || is_space(data[i]) {
            tokens.push(Token::Range(start, i));
            continue;
        }

        let mut word = data[start..i].to_vec();
        let mut quote = None;
        while i < data.len() {
            let c = data[i];
            if c == b'\\' {
                word.push(*data.get(i + 1)?);
                i += 2;
            } else if let Some(q) = quote {
                if c == q {
                    quote = None;
                } else {
                    word.push(c);
                }
                i += 1;
            } else if c == b'\'' || c == b'"' {
                quote = Some(c);
                i += 1;
            } else if is_space(c) {
                break;
            } else {
                word.push(c);
                i += 1;
            }
        }
        if quote.is_some() {
            return None;
        }
        tokens.push(Token::Owned(word));
    }
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(data: &str) -> Option<Vec<String>> {
        let tokens = tokenize(data.as_bytes())?;
        Some(
            tokens
                .into_iter()
                .map(|token| match token {
                    Token::Range(start, end) => data[start..end].to_owned(),
                    Token::Owned(bytes) => String::from_utf8(bytes).unwrap(),
                })
                .collect(),
        )
    }

    #[test]
    fn splits_on_whitespace() {
        assert_eq!(words(" a\tb\n\nc \r\n").unwrap(), ["a", "b", "c"]);
        assert!(words("").unwrap().is_empty());
    }

    #[test]
    fn quotes_and_backslashes() {
        assert_eq!(
            words(r#"'a b' "c d" e\ f g'h i'j "it's" 'say "hi"' '' \\"#).unwrap(),
            ["a b", "c d", "e f", "gh ij", "it's", "say \"hi\"", "", "\\"]
        );
    }

    #[test]
    fn plain_words_borrow() {
        let tokens = tokenize(b"plain 'quoted'").unwrap();
        assert!(matches!(tokens[0], Token::Range(0, 5)));
        assert!(matches!(tokens[1], Token::Owned(_)));
    }

    #[test]
    fn premature_end() {
        assert!(words("'open").is_none());
        assert!(words("trailing\\").is_none());
    }
}
