//! Shell completion, answered by the program itself.
//!
//! A generated script ([`script`]) registers a completion function with the
//! shell; on Tab it runs the program as `PROG __complete_word__ --shell SHELL
//! --line LINE` (see [`answer`]), and the program answers from its own help
//! data ([`help::Command`](crate::help::Command)): flag names, a flag's
//! choices, subcommand names, or a request for the shell's own file
//! completion. The script stays a few lines and never goes stale: what can be
//! typed is whatever the binary parses.
//!
//! [`Args::completion_request`](crate::Args::completion_request) answers the
//! callback before the command line is parsed, and
//! [`Args::completion_script`](crate::Args::completion_script) writes the
//! script.
//!
//! What is completed:
//!
//! - after `-` or `--`, the visible flags of the command reached so far, with
//!   their descriptions;
//! - a flag's value (`--color <TAB>`, `--color=<TAB>`), from its choices, or
//!   what its [`Hint`] says the shell knows: files for a path or a word;
//! - elsewhere, subcommand names and a positional's choices, or its hint's.
//!
//! A value's type gives the hint ([`FromArg::HINT`](crate::FromArg::HINT)):
//! a `PathBuf`, a `String` or an `OsString` is a file, and a number or a
//! type of the program nothing a shell knows. A field says otherwise with
//! `#[arg(hint = "dir")]`, `"command"`, `"file"` or `"none"`. A value only
//! the program knows is answered by a function,
//! `#[arg(complete = branches)]`, given what is typed of it; each candidate
//! is one plain word, as the scripts quote nothing for the shell.
//!
//! In a program, the callback is answered first and the script is printed on
//! request, here by a flag of its own:
//!
#![cfg_attr(feature = "derive", doc = "```no_run")]
#![cfg_attr(not(feature = "derive"), doc = "```ignore")]
//! use winnow_args::Args;
//! use winnow_args::complete::Shell;
//!
//! #[derive(Args)]
//! #[arg(name = "tool")]
//! struct Cli {
//!     /// Print a completion script: `source <(tool --completions bash)`.
//!     #[arg(long, value_name = "SHELL")]
//!     completions: Option<Shell>,
//! }
//!
//! let args: Vec<_> = std::env::args_os().skip(1).collect();
//! if let Some(answer) = Cli::completion_request(&args) {
//!     print!("{answer}");
//!     return;
//! }
//! let cli = Cli::parse();
//! if let Some(shell) = cli.completions {
//!     print!("{}", Cli::completion_script(shell));
//! }
//! ```

use std::ffi::OsString;
use std::fmt::Write as _;

use winnow::stream::BStr;

use crate::help::{Command, Item};

/// The word a completion script calls the program with.
pub const REQUEST: &str = "__complete_word__";

/// What a value is, when it is not one of a fixed set: what the shell is
/// asked to complete itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Hint {
    /// Nothing a shell can know: a number, a name, free text. What a value
    /// is taken for when its field does not say and its type is no path or
    /// word. PowerShell
    /// offers files all the same, as it does wherever it is answered nothing.
    #[default]
    Nothing,
    /// A file name, directories among them: what a `PathBuf`, a `String`
    /// and an `OsString` are.
    Files,
    /// A directory.
    Dirs,
    /// A command on the `PATH`.
    Commands,
}

impl Hint {
    /// The line in an answer that asks the shell for it.
    const fn line(self) -> Option<&'static str> {
        match self {
            Hint::Files => Some("\u{1}files"),
            Hint::Dirs => Some("\u{1}dirs"),
            Hint::Commands => Some("\u{1}commands"),
            Hint::Nothing => None,
        }
    }
}

/// The shells a script can be written for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shell {
    /// GNU bash, with its `complete -F`.
    Bash,
    /// zsh's completion system (`compinit`).
    Zsh,
    /// fish.
    Fish,
    /// Elvish.
    Elvish,
    /// PowerShell.
    PowerShell,
}

impl Shell {
    /// Every shell a script can be written for.
    pub const ALL: &'static [Shell] = &[
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Elvish,
        Shell::PowerShell,
    ];

    /// The shell's name: `bash`, `zsh`, `fish`, `elvish`, `powershell`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Elvish => "elvish",
            Shell::PowerShell => "powershell",
        }
    }

    /// The shell named `name`, if a script can be written for it.
    pub fn from_name(name: &str) -> Option<Shell> {
        Shell::ALL.iter().copied().find(|s| s.as_str() == name)
    }
}

/// So a program's own `--completions <SHELL>` flag can be a `Shell`.
impl crate::FromArg for Shell {
    // `winnow-args-spec` knows these choices under this name.
    const SPEC: &'static str = "Shell";

    const CHOICES: &'static [&'static str] = &["bash", "zsh", "fish", "elvish", "powershell"];

    fn from_arg(value: &BStr) -> Result<Self, crate::error::BoxError> {
        let name = std::str::from_utf8(value).ok();
        match name.and_then(Shell::from_name) {
            Some(shell) => Ok(shell),
            None => Err(Box::new(crate::ChoiceError {
                choices: Self::CHOICES,
            })),
        }
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One thing that can be typed at the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The whole word, as it replaces the one being typed.
    pub value: String,
    /// What it is, for shells that show descriptions.
    pub help: &'static str,
}

/// What can be typed at the cursor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Completions {
    /// The candidates, in help order.
    pub candidates: Vec<Candidate>,
    /// What fits here too, which the shell completes itself.
    pub hint: Hint,
    /// The part of the word that comes before what the shell completes:
    /// `--name=` for a value attached to its flag, empty for a whole word.
    pub before: String,
}

/// What can be typed as the last of `words`, the command line after the
/// program's name up to the cursor. The last word is the one being typed,
/// empty after a space.
pub fn complete(root: &'static Command, words: &[&str]) -> Completions {
    let (current, before) = match words.split_last() {
        Some((current, before)) => (*current, before),
        None => ("", words),
    };
    let mut state = Walk {
        command: root,
        globals: Vec::new(),
        owing: None,
        stopped: false,
        position: 0,
    };
    for word in before {
        state.step(word);
    }
    state.complete(current)
}

/// Words still taken by the flag just seen: `fixed` of them whole, then
/// non-flag-like ones, a single word when `once`.
#[derive(Clone, Copy)]
struct Owing {
    item: &'static Item,
    fixed: usize,
    flexible: bool,
    once: bool,
}

/// Where a command line stands after the words before the cursor.
struct Walk {
    command: &'static Command,
    /// The global flags of the commands above.
    globals: Vec<&'static Item>,
    /// A flag still taking values.
    owing: Option<Owing>,
    /// Past a `--`.
    stopped: bool,
    /// How many positionals were given.
    position: usize,
}

impl Walk {
    fn flags(&self) -> impl Iterator<Item = &'static Item> + '_ {
        let own = self.command.items.iter().filter(|i| !i.positional);
        own.chain(self.globals.iter().copied())
    }

    fn long(&self, name: &str) -> Option<&'static Item> {
        self.flags()
            .find(|i| i.long == Some(name) || i.negate == Some(name) || i.aliases.contains(&name))
    }

    /// Whether `item` takes the next word as its value.
    fn takes_next(item: &Item) -> bool {
        item.value_name.is_some() && !item.require_equals
    }

    fn letter(&self, letter: char, plus: bool) -> Option<&'static Item> {
        self.flags().find(|i| {
            if plus {
                i.plus == Some(letter)
            } else {
                i.short == Some(letter) || i.more_shorts.contains(&letter)
            }
        })
    }

    fn plus_options(&self) -> bool {
        self.flags().any(|i| i.plus.is_some())
    }

    /// The letters of a `+abc` bundle, when this command has `+` options.
    fn plus_letters<'a>(&self, word: &'a str) -> Option<&'a str> {
        let letters = word.strip_prefix('+').filter(|l| !l.is_empty())?;
        self.plus_options().then_some(letters)
    }

    /// `attached` is the value already in this word (`--name=value`, `-nrest`).
    fn owe(&mut self, item: &'static Item, attached: bool) {
        if item.value_name.is_none() || (!attached && item.require_equals) {
            return;
        }
        if item.more_values {
            self.owing = Some(Owing {
                item,
                fixed: 0,
                flexible: true,
                once: false,
            });
            return;
        }
        if !attached && item.optional_value {
            self.owing = Some(Owing {
                item,
                fixed: 0,
                flexible: true,
                once: true,
            });
            return;
        }
        let left = item.values.max(1) - usize::from(attached);
        if left > 0 {
            self.owing = Some(Owing {
                item,
                fixed: left,
                flexible: false,
                once: false,
            });
        }
    }

    /// Whether `word` is one of the values still owed. A flag-like word ends a
    /// flexible run and is left for the caller to read as a flag.
    fn consume_owing(&mut self, word: &str) -> bool {
        let Some(owing) = self.owing else {
            return false;
        };
        if owing.fixed > 0 {
            let fixed = owing.fixed - 1;
            self.owing = (fixed > 0).then_some(Owing { fixed, ..owing });
            return true;
        }
        if owing.flexible && !crate::token::is_flag_like(word.as_bytes()) {
            if owing.once {
                self.owing = None;
            }
            return true;
        }
        self.owing = None;
        false
    }

    fn bundle(&mut self, letters: &str, plus: bool) {
        for (at, letter) in letters.char_indices() {
            let Some(item) = self.letter(letter, plus) else {
                break;
            };
            if item.value_name.is_some() {
                // The value is the rest of the word, or the next word.
                if at + letter.len_utf8() == letters.len() && Self::takes_next(item) {
                    self.owe(item, false);
                }
                break;
            }
        }
    }

    fn step(&mut self, word: &str) {
        if self.consume_owing(word) {
            return;
        }
        if self.stopped {
            self.position += 1;
            return;
        }
        if word == "--" {
            self.stopped = true;
        } else if let Some(body) = word.strip_prefix("--") {
            let (name, value) = match body.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (body, None),
            };
            if let Some(item) = self.long(name) {
                self.owe(item, value.is_some());
            }
        } else if let Some(letters) = word.strip_prefix('-').filter(|l| !l.is_empty()) {
            self.bundle(letters, false);
        } else if let Some(letters) = self.plus_letters(word) {
            self.bundle(letters, true);
        } else if let Some(sub) = self
            .command
            .subcommands
            .iter()
            .find(|s| s.names.contains(&word))
        {
            let globals = self.command.items.iter().filter(|i| i.global);
            self.globals.extend(globals);
            self.command = sub.command;
            self.position = 0;
        } else {
            self.position += 1;
        }
    }

    fn complete(&self, current: &str) -> Completions {
        let mut out = Completions::default();
        if let Some(owing) = self.owing {
            let as_value = owing.fixed > 0
                || (owing.flexible && !crate::token::is_flag_like(current.as_bytes()));
            if as_value {
                values(owing.item, "", current, &mut out);
                return out;
            }
        }
        if !self.stopped && current.starts_with('+') && self.plus_options() {
            self.plus_names(current, &mut out);
            return out;
        }
        if !self.stopped && current.starts_with('-') {
            if let Some((name, value)) = current
                .strip_prefix("--")
                .and_then(|body| body.split_once('='))
            {
                if let Some(item) = self.long(name).filter(|i| i.value_name.is_some()) {
                    values(
                        item,
                        &current[..current.len() - value.len()],
                        value,
                        &mut out,
                    );
                }
                return out;
            }
            self.flag_names(current, &mut out);
            return out;
        }
        if !self.stopped {
            for sub in self.command.subcommands.iter().filter(|s| !s.hide) {
                push(&mut out, sub.name, sub.about, current);
            }
        }
        let mut positionals = self.command.items.iter().filter(|i| i.positional);
        let positional = positionals
            .clone()
            .nth(self.position)
            .or_else(|| positionals.rfind(|i| i.multiple));
        if let Some(item) = positional {
            values(item, "", current, &mut out);
        }
        out
    }

    fn flag_names(&self, current: &str, out: &mut Completions) {
        // `--` typed: only long names fit.
        let shorts = !current.starts_with("--");
        for item in self.flags().filter(|i| !i.hide) {
            if let Some(long) = item.long {
                push(out, &format!("--{long}"), item.help, current);
            }
            if let Some(negate) = item.negate {
                push(out, &format!("--{negate}"), item.help, current);
            }
            if shorts {
                if let Some(short) = item.short {
                    push(out, &format!("-{short}"), item.help, current);
                }
                for &letter in item.more_shorts {
                    push(out, &format!("-{letter}"), item.help, current);
                }
            }
        }
        let command = self.command;
        if command.help_flag {
            if command.help_short && shorts {
                push(out, "-h", "Print help", current);
            }
            push(out, "--help", "Print help", current);
        }
        if command.version.is_some() {
            if shorts {
                push(out, "-V", "Print version", current);
            }
            push(out, "--version", "Print version", current);
        }
    }

    fn plus_names(&self, current: &str, out: &mut Completions) {
        for item in self.flags().filter(|i| !i.hide) {
            if let Some(letter) = item.plus {
                push(out, &format!("+{letter}"), item.help, current);
            }
        }
    }
}

/// A value for `item`: what its function answers, else one of its choices,
/// else what its hint says the shell knows.
fn values(item: &'static Item, prefix: &str, typed: &str, out: &mut Completions) {
    if let Some(complete) = item.complete {
        let fitting = complete(typed)
            .into_iter()
            .filter(|c| c.value.starts_with(typed));
        out.candidates.extend(fitting.map(|c| Candidate {
            value: format!("{prefix}{}", c.value),
            help: c.help,
        }));
        return;
    }
    if item.choices.is_empty() {
        out.hint = item.hint;
        prefix.clone_into(&mut out.before);
        return;
    }
    for choice in item.choices {
        if choice.starts_with(typed) {
            out.candidates.push(Candidate {
                value: format!("{prefix}{choice}"),
                help: "",
            });
        }
    }
}

fn push(out: &mut Completions, value: &str, help: &'static str, typed: &str) {
    if value.starts_with(typed) {
        out.candidates.push(Candidate {
            value: value.to_owned(),
            help,
        });
    }
}

/// Answer a completion script's callback: `args` is the command line after
/// the program's name. `None` when it is not a callback, so the program goes
/// on to parse it.
///
/// The callback is `__complete_word__ --shell SHELL --line LINE`, `LINE`
/// being the command line up to the cursor, program name first; bash adds
/// `--bash-word WORD`, the part of the current word it will replace (it breaks
/// words at `=` and `:` too). A shell that hands over words already unquoted
/// (elvish) ends with `--words WORD…`, the program's name first, instead of
/// `--line`. The answer is one candidate a line, as [`render`] writes it.
pub fn answer(root: &'static Command, args: &[OsString]) -> Option<String> {
    let (first, rest) = args.split_first()?;
    if first.to_str()? != REQUEST {
        return None;
    }
    let (mut shell, mut line, mut replaced) = (None, String::new(), None);
    let mut given: Option<Vec<String>> = None;
    let mut rest = rest.iter().map(|a| a.to_string_lossy().into_owned());
    while let Some(option) = rest.next() {
        if option == "--words" {
            // Every later argument is a word, the program's name first.
            given = Some(rest.by_ref().collect());
            break;
        }
        let value = rest.next().unwrap_or_default();
        match option.as_str() {
            "--shell" => shell = Shell::from_name(&value),
            "--line" => line = value,
            "--bash-word" => replaced = Some(value),
            _ => {}
        }
    }
    let shell = shell?;
    let mut words = given.unwrap_or_else(|| split_line(&line));
    if !words.is_empty() {
        words.remove(0);
    }
    if words.is_empty() {
        words.push(String::new());
    }
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    let completions = complete(root, &words);
    Some(match (shell, replaced) {
        (Shell::Bash, Some(replaced)) => {
            render_bash(&completions, words.last().copied().unwrap_or(""), &replaced)
        }
        _ => render(&completions, shell),
    })
}

/// The answer as `shell`'s script reads it: one candidate a line, a
/// description after a colon (zsh) or a tab (the others), and a line asking for
/// what the shell completes itself when that fits: its mark, a tab, and the
/// part of the word before what the shell is to complete.
pub fn render(completions: &Completions, shell: Shell) -> String {
    let mut out = String::new();
    for c in &completions.candidates {
        match shell {
            Shell::Bash => out.push_str(&c.value),
            Shell::Zsh => {
                out.push_str(&c.value.replace(':', "\\:"));
                if !c.help.is_empty() {
                    let _ = write!(out, ":{}", first_line(c.help));
                }
            }
            Shell::Fish | Shell::Elvish | Shell::PowerShell => {
                out.push_str(&c.value);
                if !c.help.is_empty() {
                    let _ = write!(out, "\t{}", first_line(c.help));
                }
            }
        }
        out.push('\n');
    }
    ask_the_shell(completions, shell, &mut out);
    out
}

/// The line that asks the shell for what it completes itself, when that fits.
/// bash breaks a word at `=`, and so needs no word of what comes before.
fn ask_the_shell(completions: &Completions, shell: Shell, out: &mut String) {
    let Some(line) = completions.hint.line() else {
        return;
    };
    out.push_str(line);
    if shell != Shell::Bash {
        out.push('\t');
        out.push_str(&completions.before);
    }
    out.push('\n');
}

/// bash replaces only the part of the word after its last `=` or `:`: the
/// candidates lose what came before it.
fn render_bash(completions: &Completions, typed: &str, replaced: &str) -> String {
    // Right after `--name=`, bash's current word is the `=` itself.
    let replaced = if replaced == "=" { "" } else { replaced };
    let cut = typed
        .strip_suffix(replaced)
        .map_or(0, |before| before.len());
    let mut out = String::new();
    for c in &completions.candidates {
        out.push_str(c.value.get(cut..).unwrap_or(&c.value));
        out.push('\n');
    }
    ask_the_shell(completions, Shell::Bash, &mut out);
    out
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

/// Split a command line as a shell would into words: whitespace between,
/// `'…'`, `"…"` and `\` quoting. A trailing space starts an empty word.
fn split_line(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote = None;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some('"') | None, '\\') => {
                if let Some(next) = chars.next() {
                    word.get_or_insert_default().push(next);
                }
            }
            (Some(_), c) => word.get_or_insert_default().push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                word.get_or_insert_default();
            }
            (None, c) if c.is_whitespace() => words.extend(word.take()),
            (None, c) => word.get_or_insert_default().push(c),
        }
    }
    words.push(word.unwrap_or_default());
    words
}

/// The completion script for `bin` in `shell`, ready to be sourced or
/// installed where the shell looks for completions.
///
/// # Panics
///
/// If `bin` is not one plain word (letters, digits, `-`, `_`, `.`, `+`): the
/// scripts name it unquoted where no shell allows quotes.
pub fn script(bin: &str, shell: Shell) -> String {
    assert!(
        !bin.is_empty()
            && bin
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+')),
        "a completion script needs a plain program name, not {bin:?}"
    );
    let function: String = bin
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    match shell {
        Shell::Bash => format!(
            r#"# Completion for {bin}, answered by `{bin} {REQUEST}`.
_{function}_complete() {{
    local line ask= cur="${{COMP_WORDS[COMP_CWORD]}}"
    # Right after `--name=`, the current word is the `=` itself.
    [[ $cur == = ]] && cur=
    COMPREPLY=()
    while IFS= read -r line; do
        case $line in
            $'\001files') ask=f ;;
            $'\001dirs') ask=d ;;
            $'\001commands') ask=c ;;
            ?*) COMPREPLY+=("$line") ;;
        esac
    done < <(command {bin} {REQUEST} --shell bash --line "${{COMP_LINE:0:COMP_POINT}}" \
        --bash-word "${{COMP_WORDS[COMP_CWORD]}}" 2>/dev/null)
    if [[ -n $ask ]]; then
        [[ $ask == c ]] || compopt -o filenames 2>/dev/null
        local IFS=$'\n'
        COMPREPLY+=($(compgen -$ask -- "$cur"))
    fi
}}
complete -F _{function}_complete {bin}
"#
        ),
        Shell::Zsh => format!(
            r#"#compdef {bin}
# Completion for {bin}, answered by `{bin} {REQUEST}`.
_{function}() {{
    local -a candidates
    local line ask= before=
    while IFS= read -r line; do
        case $line in
            $'\001files\t'*) ask=_files ;;
            $'\001dirs\t'*) ask='_path_files -/' ;;
            $'\001commands\t'*) ask=_command_names ;;
            ?*) candidates+=("$line"); continue ;;
        esac
        before=${{line#*$'\t'}}
    done < <(command {bin} {REQUEST} --shell zsh --line "${{BUFFER[1,CURSOR]}}" 2>/dev/null)
    (( ${{#candidates}} )) && _describe -t values '{bin}' candidates
    if [[ -n $ask ]]; then
        # `--name=`, when the value is attached: not a part of what is asked for.
        if [[ -n $before && $PREFIX == "$before"* ]]; then
            IPREFIX+=$before
            PREFIX=${{PREFIX#"$before"}}
        fi
        ${{=ask}}
    fi
    return 0
}}
if [[ $zsh_eval_context[-1] == loadautofunc ]]; then
    _{function} "$@"
else
    compdef _{function} {bin}
fi
"#
        ),
        Shell::Fish => format!(
            r#"# Completion for {bin}, answered by `{bin} {REQUEST}`.
function __{function}_complete
    set -l token (commandline -ct)
    for line in (command {bin} {REQUEST} --shell fish --line (commandline -cp) 2>/dev/null)
        if not string match -q \u0001'*' -- $line
            test -n "$line"; and printf '%s\n' $line
            continue
        end
        # The mark, then `--name=` when the value is attached to its flag.
        set -l asked (string split -m 1 \t -- $line)
        set -l before $asked[2]
        set -l typed (string sub -s (math (string length -- "$before") + 1) -- $token)
        switch $asked[1]
            case \u0001files
                printf '%s\n' $before(__fish_complete_path "$typed")
            case \u0001dirs
                printf '%s\n' $before(__fish_complete_directories "$typed")
            case \u0001commands
                printf '%s\n' $before(complete -C "$typed")
        end
    end
end
complete -c {bin} -f -a '(__{function}_complete)'
"#
        ),
        Shell::Elvish => format!(
            r#"# Completion for {bin}, answered by `{bin} {REQUEST}`.
use str

set edit:completion:arg-completer[{bin}] = {{|@words|
    for answer [(e:{bin} {REQUEST} --shell elvish --words $@words 2>/dev/null | from-lines)] {{
        if (str:has-prefix $answer "\x01") {{
            # The mark, then `--name=` when the value is attached to its flag.
            var asked before = (str:split &max=2 "\t" $answer)
            var typed = (str:trim-prefix $words[-1] $before)
            var found = []
            if (has-value ["\x01files" "\x01dirs"] $asked) {{
                set found = [(edit:complete-filename $typed)]
            }} elif (eq $asked "\x01commands") {{
                set found = [(edit:complete-sudo sudo $typed)]
            }}
            for one $found {{
                if (eq (kind-of $one) string) {{
                    put $before$one
                }} elif (or (not-eq $asked "\x01dirs") (str:has-suffix $one[stem] /)) {{
                    edit:complex-candidate $before$one[stem] &code-suffix=$one[code-suffix] &display=$one[display]
                }}
            }}
        }} elif (not-eq $answer '') {{
            var parts = [(str:split "\t" $answer)]
            var display = $parts[0]
            if (> (count $parts) 1) {{
                set display = (str:join '  -- ' $parts)
            }}
            edit:complex-candidate $parts[0] &display=$display
        }}
    }}
}}
"#
        ),
        Shell::PowerShell => format!(
            r#"# Completion for {bin}, answered by `{bin} {REQUEST}`.
Register-ArgumentCompleter -Native -CommandName '{bin}' -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    # The command's text up to the cursor: `$cursorPosition` counts from the
    # start of the whole input, the extent from its own start.
    $extent = $commandAst.Extent
    $offset = [Math]::Min([Math]::Max($cursorPosition - $extent.StartOffset, 0), $extent.Text.Length)
    $line = $extent.Text.Substring(0, $offset)
    # The extent ends at the last token: a word not yet typed needs its space.
    if ($wordToComplete -eq '' -and -not $line.EndsWith(' ')) {{ $line += ' ' }}
    $answers = @(& '{bin}' {REQUEST} --shell powershell --line $line 2>$null)

    foreach ($answer in $answers) {{
        if ([string]::IsNullOrEmpty($answer)) {{ continue }}
        $parts = $answer -split "`t", 2
        if ($answer[0] -eq [char]1) {{
            # The mark, then `--name=` when the value is attached to its flag.
            $before = $parts[1]
            $typed = $wordToComplete
            if ($before -and $typed.StartsWith($before)) {{ $typed = $typed.Substring($before.Length) }}
            $found = switch ($parts[0].Substring(1)) {{
                'files' {{ [System.Management.Automation.CompletionCompleters]::CompleteFilename($typed) }}
                'dirs' {{
                    [System.Management.Automation.CompletionCompleters]::CompleteFilename($typed) |
                        Where-Object ResultType -eq 'ProviderContainer'
                }}
                'commands' {{ [System.Management.Automation.CompletionCompleters]::CompleteCommand($typed) }}
            }}
            foreach ($one in $found) {{
                [System.Management.Automation.CompletionResult]::new(
                    $before + $one.CompletionText, $one.ListItemText, $one.ResultType, $one.ToolTip)
            }}
            continue
        }}
        $help = if ($parts.Count -gt 1 -and $parts[1]) {{ $parts[1] }} else {{ $parts[0] }}
        [System.Management.Automation.CompletionResult]::new(
            $parts[0], $parts[0], 'ParameterValue', $help)
    }}
}}
"#
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_split_as_a_shell_splits_them() {
        assert_eq!(
            split_line("prog -o 'a b' c\\ d"),
            ["prog", "-o", "a b", "c d"]
        );
        assert_eq!(split_line("prog --x="), ["prog", "--x="]);
        assert_eq!(split_line("prog "), ["prog", ""]);
        assert_eq!(split_line("prog \"q\\\"x"), ["prog", "q\"x"]);
    }

    #[test]
    fn bash_gets_the_part_after_its_word_break() {
        let c = Completions {
            candidates: vec![Candidate {
                value: "--color=always".into(),
                help: "",
            }],
            ..Completions::default()
        };
        assert_eq!(render_bash(&c, "--color=al", "al"), "always\n");
        assert_eq!(render_bash(&c, "--color=", "="), "always\n");
        assert_eq!(render_bash(&c, "--col", "--col"), "--color=always\n");
    }

    #[test]
    fn scripts_call_back_the_program() {
        for &shell in Shell::ALL {
            let script = script("my-tool", shell);
            assert!(script.contains("my-tool __complete_word__"), "{script}");
        }
    }
}
