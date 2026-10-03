//! Value delimiters: one word, several values. Combinators and derive must agree.

use winnow::combinator::{dispatch, fail};
use winnow::prelude::*;
use winnow_args::combinator::{Named, args, long, short};
use winnow_args::token::{Arg, arg};
use winnow_args::{Args, Argv, Error, ErrorKind};

#[derive(Args, Debug, PartialEq, Default)]
struct Cli {
    #[arg(short = 'E', long, delimiter = ',')]
    env: Vec<String>,
    #[arg(long, delimiter = ',')]
    num: Vec<u32>,
    #[arg(positional, delimiter = ':')]
    ports: Vec<u16>,
}

const ENV: Named = short('E').long("env");
const NUM: Named = long("num");

fn combinator(input: &mut Argv<'_>) -> Result<Cli, Error> {
    let mut cli = Cli::default();
    let c = &mut cli;
    args(dispatch! {arg;
        a @ Arg::Long(_) | a @ Arg::Short(_) if ENV.matches(&a) => {
            a.values_as(b',').map(|v: Vec<String>| c.env.extend(v))
        },
        a @ Arg::Long(_) if NUM.matches(&a) => a.values_as(b',').map(|v: Vec<u32>| c.num.extend(v)),
        Arg::Word(w) => |_: &mut Argv<'_>| {
            for piece in w.split(b':') {
                c.ports.push(piece.convert("PORTS")?);
            }
            Ok(())
        },
        _ => fail,
    })
    .parse_next(input)?;
    Ok(cli)
}

fn parse(line: &[&str]) -> Result<Cli, Error> {
    let words = crate::words(line);
    let a = combinator.parse_next(&mut Argv::new(&words));
    let b = Cli::parse_words(&words);
    assert_eq!(a, b, "combinator and derive disagree on {line:?}");
    a
}

fn ok(line: &[&str]) -> Cli {
    parse(line).unwrap_or_else(|e| panic!("{line:?}: {e}"))
}

#[test]
fn every_piece_is_a_value() {
    let cli = ok(&["-E", "a,b", "-E", "c", "--env=d,,e", "-Ef,g"]);
    assert_eq!(cli.env, ["a", "b", "c", "d", "", "e", "f", "g"]);
    assert_eq!(ok(&["--num", "1,2", "--num=3"]).num, [1, 2, 3]);
    assert_eq!(ok(&["80:443", "8080"]).ports, [80, 443, 8080]);
    // An empty word is one empty value, not none.
    assert_eq!(ok(&["--env="]).env, [""]);
}

#[test]
fn a_bad_piece_is_reported_on_its_own() {
    let e = parse(&["--num", "1,x"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.token(), e.value()),
        (ErrorKind::InvalidValue, Some("--num"), Some("x"))
    );
    // A positional piece is reported where it starts: "80:" is 3 bytes into the word.
    let e = parse(&["80:bad"]).unwrap_err();
    assert_eq!(
        (e.kind(), e.value(), e.offset()),
        (ErrorKind::InvalidValue, Some("bad"), 3)
    );
}
