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
//!   files when it has none;
//! - elsewhere, subcommand names and a positional's choices, or files.
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

/// A line in an answer that asks the shell to complete file names itself.
const FILES: &str = "\u{1}files";

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
    /// Whether a file name fits here too: the shell completes those itself.
    pub files: bool,
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
        pending: None,
        stopped: false,
        position: 0,
    };
    for word in before {
        state.step(word);
    }
    state.complete(current)
}

/// Where a command line stands after the words before the cursor.
struct Walk {
    command: &'static Command,
    /// The global flags of the commands above.
    globals: Vec<&'static Item>,
    /// A flag waiting for its value in the next word.
    pending: Option<&'static Item>,
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

    fn short(&self, letter: char) -> Option<&'static Item> {
        self.flags().find(|i| i.short == Some(letter))
    }

    fn step(&mut self, word: &str) {
        if self.pending.take().is_some() {
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
            self.pending = self
                .long(name)
                .filter(|i| Self::takes_next(i) && value.is_none());
        } else if let Some(letters) = word.strip_prefix('-').filter(|l| !l.is_empty()) {
            for (at, letter) in letters.char_indices() {
                let Some(item) = self.short(letter) else {
                    break;
                };
                if item.value_name.is_some() {
                    // The value is the rest of the word, or the next word.
                    if at + letter.len_utf8() == letters.len() && Self::takes_next(item) {
                        self.pending = Some(item);
                    }
                    break;
                }
            }
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
        if let Some(item) = self.pending {
            values(item, "", current, &mut out);
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
            if let Some(short) = item.short.filter(|_| shorts) {
                push(out, &format!("-{short}"), item.help, current);
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
}

/// A value for `item`: one of its choices, else a file name.
fn values(item: &'static Item, prefix: &str, typed: &str, out: &mut Completions) {
    if item.choices.is_empty() {
        out.files = true;
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
/// file names when they fit.
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
    ask_for_files(completions, &mut out);
    out
}

/// The line that asks the shell for file names, when they fit.
fn ask_for_files(completions: &Completions, out: &mut String) {
    if completions.files {
        out.push_str(FILES);
        out.push('\n');
    }
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
    ask_for_files(completions, &mut out);
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
    local line files= cur="${{COMP_WORDS[COMP_CWORD]}}"
    # Right after `--name=`, the current word is the `=` itself.
    [[ $cur == = ]] && cur=
    COMPREPLY=()
    while IFS= read -r line; do
        if [[ $line == $'\001files' ]]; then
            files=1
        elif [[ -n $line ]]; then
            COMPREPLY+=("$line")
        fi
    done < <(command {bin} {REQUEST} --shell bash --line "${{COMP_LINE:0:COMP_POINT}}" \
        --bash-word "${{COMP_WORDS[COMP_CWORD]}}" 2>/dev/null)
    if [[ -n $files ]]; then
        compopt -o filenames 2>/dev/null
        local IFS=$'\n'
        COMPREPLY+=($(compgen -f -- "$cur"))
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
    local line files=
    while IFS= read -r line; do
        if [[ $line == $'\001files' ]]; then
            files=1
        elif [[ -n $line ]]; then
            candidates+=("$line")
        fi
    done < <(command {bin} {REQUEST} --shell zsh --line "${{BUFFER[1,CURSOR]}}" 2>/dev/null)
    (( ${{#candidates}} )) && _describe -t values '{bin}' candidates
    [[ -n $files ]] && _files
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
    for line in (command {bin} {REQUEST} --shell fish --line (commandline -cp) 2>/dev/null)
        if test "$line" = \u0001files
            __fish_complete_path (commandline -ct)
        else if test -n "$line"
            echo $line
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
        if (eq $answer "\x01files") {{
            edit:complete-filename $words[-1]
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
        if ($answer -eq ([char]1 + 'files')) {{
            [System.Management.Automation.CompletionCompleters]::CompleteFilename($wordToComplete)
            continue
        }}
        $parts = $answer -split "`t", 2
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
            files: false,
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
