//! A shadow of `mise.usage.kdl` in winnow-args' vocabulary, translated from
//! usage's shadow by `tasks/gen-mise-shadow.py`.
//!
//! Do not edit: regenerate it. It exists to be compiled and parsed against, so
//! that the parser can be measured at a real CLI's scale rather than a toy one.
#![allow(dead_code, unused_imports)]
// mise's examples are indented like code blocks; they are not doctests.
#![cfg(not(doctest))]
#![allow(
    rustdoc::broken_intra_doc_links,
    rustdoc::bare_urls,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks
)]

use winnow_args::{Args, Subcommand, ValueEnum};

/// Initializes mise in the current shell session
///
/// This should go into your shell's rc file or login shell. Otherwise, it will only take effect in the current session. (e.g. ~/.zshrc, ~/.zprofile, ~/.zshenv, ~/.bashrc, ~/.bash_profile, ~/.profile, ~/.config/fish/config.fish, or $PROFILE for powershell)
///
/// Typically, this can be added with something like the following:
///
///     echo 'eval "$(mise activate zsh)"' >> ~/.zshrc
///
/// However, this requires that "mise" is in your PATH. If it is not, you need to specify the full path like this:
///
///     echo 'eval "$(/path/to/mise activate zsh)"' >> ~/.zshrc
///
/// Customize status output with `status` settings.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1meval \"$(mise activate bash)\"\u{1b}[22m\n    $ \u{1b}[1meval \"$(mise activate zsh)\"\u{1b}[22m\n    $ \u{1b}[1mmise activate fish | source\u{1b}[22m\n    $ \u{1b}[1mexecx($(mise activate xonsh))\u{1b}[22m\n    $ \u{1b}[1m(&mise activate pwsh) | Out-String | Invoke-Expression\u{1b}[22m\n"
)]
pub struct ActivateArgs {
    /// Suppress non-error messages
    #[arg(long = "quiet", short = 'q')]
    pub quiet: bool,
    /// Shell type to generate the script for
    #[arg(
        long = "shell",
        short = 's',
        hide,
        value_name = "SHELL",
        choices("bash", "elvish", "fish", "nu", "xonsh", "zsh", "pwsh")
    )]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Do not automatically call hook-env
    ///
    /// This can be helpful for debugging mise. If you run `eval "$(mise activate --no-hook-env)"`, then you can call `mise hook-env` manually which will output the env vars to stdout without actually modifying the environment. That way you can do things like `mise hook-env --trace` to get more information or just see the values that hook-env is outputting.
    #[arg(long = "no-hook-env")]
    pub no_hook_env: bool,
    #[arg(
        help = "Use shims instead of modifying PATH\nEffectively the same as:",
        long_help = "Use shims instead of modifying PATH\nEffectively the same as:\n\n    PATH=\"$HOME/.local/share/mise/shims:$PATH\"\n\n`mise activate --shims` does not support all the features of `mise activate`.\nSee https://mise.jdx.dev/dev-tools/shims.html#shims-vs-path for more information",
        long = "shims"
    )]
    pub shims: bool,
    /// Show "mise: <TOOL>@<VERSION>" message when changing directories
    #[arg(long = "status", hide)]
    pub status: bool,
    /// Shell type to generate the script for
    #[arg(
        positional,
        value_name = "SHELL_TYPE",
        choices("bash", "elvish", "fish", "nu", "xonsh", "zsh", "pwsh")
    )]
    pub shell_type: ::std::option::Option<::std::string::String>,
}

/// Show an alias for a tool
///
/// This is the contents of a tool_alias.<TOOL> entry in ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tool-alias get node lts-hydrogen\u{1b}[22m\n    20.0.0\n"
)]
pub struct ToolAliasGetArgs {
    /// The tool to show the alias for
    #[arg(positional, value_name = "TOOL")]
    pub tool: ::std::string::String,
    /// The alias to show
    #[arg(positional, value_name = "ALIAS")]
    pub alias: ::std::string::String,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tool-alias ls\u{1b}[22m\n    node  lts-jod      22\n"
)]
pub struct ToolAliasLsArgs {
    /// Don't show table header
    #[arg(long = "no-header")]
    pub no_header: bool,
    /// Show aliases for <TOOL>
    #[arg(positional, value_name = "TOOL")]
    pub tool: ::std::option::Option<::std::string::String>,
}

/// Add/update an alias for a tool/backend
///
/// This modifies the contents of ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tool-alias set maven asdf:mise-plugins/mise-maven\u{1b}[22m\n    $ \u{1b}[1mmise tool-alias set node lts-jod 22.0.0\u{1b}[22m\n"
)]
pub struct ToolAliasSetArgs {
    /// The tool/backend to set the alias for
    #[arg(positional, value_name = "TOOL")]
    pub tool: ::std::string::String,
    /// The alias to set
    #[arg(positional, value_name = "ALIAS")]
    pub alias: ::std::string::String,
    /// The value to set the alias to
    #[arg(positional, value_name = "VALUE")]
    pub value: ::std::option::Option<::std::string::String>,
}

/// Clears an alias for a tool/backend
///
/// This modifies the contents of ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tool-alias unset maven\u{1b}[22m\n    $ \u{1b}[1mmise tool-alias unset node lts-jod\u{1b}[22m\n"
)]
pub struct ToolAliasUnsetArgs {
    /// The tool/backend to remove the alias from
    #[arg(positional, value_name = "TOOL")]
    pub tool: ::std::string::String,
    /// The alias to remove
    #[arg(positional, value_name = "ALIAS")]
    pub alias: ::std::option::Option<::std::string::String>,
}

/// Manage tool version aliases.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct ToolAliasArgs {
    /// Filter aliases by tool
    #[arg(long = "tool", alias = "plugin", short = 'p', value_name = "TOOL")]
    pub tool: ::std::option::Option<::std::string::String>,
    /// Don't show table header
    #[arg(long = "no-header")]
    pub no_header: bool,
    #[arg(subcommand)]
    pub command: ::std::option::Option<ToolAliasCommands>,
}

#[derive(Subcommand)]
pub enum ToolAliasCommands {
    /// Show an alias for a tool
    #[arg(name = "get")]
    Get(Box<ToolAliasGetArgs>),
    #[arg(
        name = "ls",
        help = "List tool version aliases\nShows the aliases that can be specified.\nThese can come from user config or from plugins in `bin/list-aliases`.",
        long_help = "List tool version aliases\nShows the aliases that can be specified.\nThese can come from user config or from plugins in `bin/list-aliases`.\n\nFor user config, aliases are defined like the following in `~/.config/mise/config.toml`:\n\n    [tool_alias.node.versions]\n    lts = \"22.0.0\"",
        alias = "list"
    )]
    Ls(Box<ToolAliasLsArgs>),
    /// Add/update an alias for a tool/backend
    #[arg(name = "set", alias("add", "create"))]
    Set(Box<ToolAliasSetArgs>),
    /// Clears an alias for a tool/backend
    #[arg(name = "unset", alias("rm", "remove", "delete", "del"))]
    Unset(Box<ToolAliasUnsetArgs>),
}

/// [internal] simulates asdf for plugins that call "asdf" internally
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct AsdfArgs {
    /// all arguments
    #[arg(positional, value_name = "ARGS", double_dash = "automatic")]
    pub args: ::std::vec::Vec<::std::string::String>,
}

/// List built-in backends
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise backends ls\u{1b}[22m\n    aqua\n    asdf\n    cargo\n    core\n    dotnet\n    gem\n    go\n    npm\n    pipx\n    spm\n    ubi\n    vfox\n"
)]
pub struct BackendsLsArgs {}

/// Manage backends
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mDeprecation:\u{1b}[22m\u{1b}[24m\n\nThe `mise b` alias is deprecated and will be removed in mise 2027.4.0.\nUse `mise backends` instead.\n"
)]
pub struct BackendsArgs {
    #[arg(subcommand)]
    pub command: ::std::option::Option<BackendsCommands>,
}

#[derive(Subcommand)]
pub enum BackendsCommands {
    /// List built-in backends
    #[arg(name = "ls", alias = "list")]
    Ls(Box<BackendsLsArgs>),
}

/// List all the active runtime bin paths
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BinPathsArgs {
    /// Output executable names instead of bin directories
    #[arg(long = "bin-names")]
    pub bin_names: bool,
    /// Output executable entries in JSON format (implies --bin-names)
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to look up\ne.g.: ruby@3"
    )]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapApplyAccountPlanArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapApplyServicePlanArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapApplyFirewallPlanArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapApplySystemPlanArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapInspectSystemFilesArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapInspectFirewallPlanArgs {}

/// Apply configured Linux users and groups
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapAccountsApplyArgs {
    /// Print what would change without changing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

/// Show configured Linux user and group state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapAccountsStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 when any account is not converged
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage Linux users and groups from `[bootstrap.users]` and `[bootstrap.groups]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapAccountsArgs {
    #[arg(subcommand)]
    pub command: BootstrapAccountsCommands,
}

#[derive(Subcommand)]
pub enum BootstrapAccountsCommands {
    /// Apply configured Linux users and groups
    #[arg(name = "apply")]
    Apply(Box<BootstrapAccountsApplyArgs>),
    /// Show configured Linux user and group state
    #[arg(name = "status")]
    Status(Box<BootstrapAccountsStatusArgs>),
}

/// Apply configured Docker Compose project state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapComposeApplyArgs {
    /// Print what would change without changing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

/// Show configured Docker Compose project state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapComposeStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 when any Compose project is not converged
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage Docker Compose projects from `[bootstrap.compose]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapComposeArgs {
    #[arg(subcommand)]
    pub command: BootstrapComposeCommands,
}

#[derive(Subcommand)]
pub enum BootstrapComposeCommands {
    /// Apply configured Docker Compose project state
    #[arg(name = "apply")]
    Apply(Box<BootstrapComposeApplyArgs>),
    /// Show configured Docker Compose project state
    #[arg(name = "status")]
    Status(Box<BootstrapComposeStatusArgs>),
}

/// Add or update dotfiles in `[dotfiles]`
///
/// If the target is already managed, this updates its source from the live target. Otherwise it creates a `[dotfiles]` entry and seeds the source under `dotfiles.root` unless `--source` is provided.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles add ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles add --mode copy ~/.config/starship.toml\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles add --source dotfiles/gitconfig ~/.gitconfig\u{1b}[22m\n"
)]
pub struct BootstrapDotfilesAddArgs {
    /// Overwrite existing sources without prompting
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Write to the global config
    #[arg(long = "global", short = 'g', conflicts("--local", "--path"))]
    pub global: bool,
    /// Write to the local config instead of the global config
    #[arg(long = "local", short = 'l', conflicts("--global", "--path"))]
    pub local: bool,
    /// Dotfile mode to write
    #[arg(long = "mode", short = 'm', value_name = "MODE")]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Print the config/source updates without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Add the entry without applying it
    #[arg(long = "no-apply")]
    pub no_apply: bool,
    /// Write to this config file or directory
    #[arg(
        long = "path",
        short = 'p',
        conflicts("--global", "--local"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Source path to use for a single target
    #[arg(long = "source", short = 's', value_name = "PATH")]
    pub source: ::std::option::Option<::std::string::String>,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Targets to add or update
    #[arg(positional, value_name = "TARGET", required)]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Apply dotfiles from `[dotfiles]`
///
/// Applies configured whole-file entries and edits that aren't in their desired state. Whole-file entries may symlink, copy, or render templates. Edit entries manage a marker-delimited block or a single line in a file mise doesn't otherwise own.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles apply\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles apply --force --yes\u{1b}[22m\n"
)]
pub struct BootstrapDotfilesApplyArgs {
    /// Overwrite existing files that conflict with whole-file dotfile entries
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Print the actions that would run without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Only apply these targets
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Edit a managed dotfile source
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles edit ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles edit --apply ~/.config/starship.toml\u{1b}[22m\n"
)]
pub struct BootstrapDotfilesEditArgs {
    /// Apply this target after the editor exits
    #[arg(long = "apply")]
    pub apply: bool,
    /// Dotfile mode to use if the target is not yet managed
    #[arg(long = "mode", short = 'm', value_name = "MODE")]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Source path to use if the target is not yet managed
    #[arg(long = "source", short = 's', value_name = "PATH")]
    pub source: ::std::option::Option<::std::string::String>,
    /// Skip the confirmation prompt when adding an unmanaged target
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Target to edit
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::string::String,
}

/// Show the status of dotfiles from `[dotfiles]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles status\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status --json\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status --missing\u{1b}[22m # exit 1 if anything is out of sync\n"
)]
pub struct BootstrapDotfilesStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    #[arg(
        help = "Exit with code 1 if any configured dotfiles are not in their desired\nstate (missing, source missing, differs)",
        long = "missing"
    )]
    pub missing: bool,
    /// Only show these targets
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Remove dotfiles applied from `[dotfiles]`
///
/// Removes configured whole-file entries and edits while preserving files mise cannot identify as managed. Modified copies, templates, and plain-line edits require `--force`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles unapply\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles unapply ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles unapply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles unapply --force --yes\u{1b}[22m\n"
)]
pub struct BootstrapDotfilesUnapplyArgs {
    /// Remove modified or otherwise ambiguous managed files and lines
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Print the actions that would run without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Only unapply these targets
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Manage dotfiles from `[dotfiles]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapDotfilesArgs {
    #[arg(subcommand)]
    pub command: BootstrapDotfilesCommands,
}

#[derive(Subcommand)]
pub enum BootstrapDotfilesCommands {
    /// Add or update dotfiles in `[dotfiles]`
    #[arg(name = "add")]
    Add(Box<BootstrapDotfilesAddArgs>),
    /// Apply dotfiles from `[dotfiles]`
    #[arg(name = "apply")]
    Apply(Box<BootstrapDotfilesApplyArgs>),
    /// Edit a managed dotfile source
    #[arg(name = "edit")]
    Edit(Box<BootstrapDotfilesEditArgs>),
    /// Show the status of dotfiles from `[dotfiles]`
    #[arg(name = "status")]
    Status(Box<BootstrapDotfilesStatusArgs>),
    /// Remove dotfiles applied from `[dotfiles]`
    #[arg(name = "unapply")]
    Unapply(Box<BootstrapDotfilesUnapplyArgs>),
}

/// Apply configured privileged files and directories
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapFilesApplyArgs {
    /// Print what would change without changing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Prompt securely for missing bootstrap secret inputs
    #[arg(long = "prompt-secrets")]
    pub prompt_secrets: bool,
}

/// Show configured privileged file and directory state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapFilesStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 when any resource is not converged
    #[arg(long = "missing")]
    pub missing: bool,
    /// Prompt securely for missing bootstrap secret inputs
    #[arg(long = "prompt-secrets")]
    pub prompt_secrets: bool,
}

/// Manage privileged files and directories from `[bootstrap.files]` and `[bootstrap.directories]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapFilesArgs {
    #[arg(subcommand)]
    pub command: BootstrapFilesCommands,
}

#[derive(Subcommand)]
pub enum BootstrapFilesCommands {
    /// Apply configured privileged files and directories
    #[arg(name = "apply")]
    Apply(Box<BootstrapFilesApplyArgs>),
    /// Show configured privileged file and directory state
    #[arg(name = "status")]
    Status(Box<BootstrapFilesStatusArgs>),
}

/// Apply the configured Linux host firewall
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapFirewallApplyArgs {
    /// Print what would change without changing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

/// Show configured Linux host firewall state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapFirewallStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 when the firewall is not converged
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage the Linux host firewall from `[bootstrap.linux.firewall]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapFirewallArgs {
    #[arg(subcommand)]
    pub command: BootstrapFirewallCommands,
}

#[derive(Subcommand)]
pub enum BootstrapFirewallCommands {
    /// Apply the configured Linux host firewall
    #[arg(name = "apply")]
    Apply(Box<BootstrapFirewallApplyArgs>),
    /// Show configured Linux host firewall state
    #[arg(name = "status")]
    Status(Box<BootstrapFirewallStatusArgs>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLaunchdApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLaunchdStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured LaunchAgent is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage macOS LaunchAgents from `[bootstrap.macos.launchd.agents]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLaunchdArgs {
    #[arg(subcommand)]
    pub command: BootstrapLaunchdCommands,
}

#[derive(Subcommand)]
pub enum BootstrapLaunchdCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapLaunchdApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapLaunchdStatusArgs>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLinuxSystemdUnitsApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLinuxSystemdUnitsStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured systemd user service is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage systemd user services from `[bootstrap.linux.systemd.units]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLinuxSystemdUnitsArgs {
    #[arg(subcommand)]
    pub command: BootstrapLinuxSystemdUnitsCommands,
}

#[derive(Subcommand)]
pub enum BootstrapLinuxSystemdUnitsCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapLinuxSystemdUnitsApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapLinuxSystemdUnitsStatusArgs>),
}

/// Manage Linux bootstrap config from `[bootstrap.linux]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapLinuxArgs {
    #[arg(subcommand)]
    pub command: BootstrapLinuxCommands,
}

#[derive(Subcommand)]
pub enum BootstrapLinuxCommands {
    /// Manage systemd user services from `[bootstrap.linux.systemd.units]`
    #[arg(name = "systemd-units", alias_hidden = "systemd")]
    SystemdUnits(Box<BootstrapLinuxSystemdUnitsArgs>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosDefaultsApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosDefaultsStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured defaults are not in their desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage macOS defaults from `[bootstrap.macos.defaults]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosDefaultsArgs2 {
    #[arg(subcommand)]
    pub command: BootstrapMacosDefaultsCommands2,
}

#[derive(Subcommand)]
pub enum BootstrapMacosDefaultsCommands2 {
    #[arg(name = "apply")]
    Apply(Box<BootstrapMacosDefaultsApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapMacosDefaultsStatusArgs>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosLaunchdAgentsApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosLaunchdAgentsStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured LaunchAgent is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage macOS LaunchAgents from `[bootstrap.macos.launchd.agents]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosLaunchdAgentsArgs {
    #[arg(subcommand)]
    pub command: BootstrapMacosLaunchdAgentsCommands,
}

#[derive(Subcommand)]
pub enum BootstrapMacosLaunchdAgentsCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapMacosLaunchdAgentsApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapMacosLaunchdAgentsStatusArgs>),
}

/// Manage macOS bootstrap config from `[bootstrap.macos]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosArgs {
    #[arg(subcommand)]
    pub command: BootstrapMacosCommands,
}

#[derive(Subcommand)]
pub enum BootstrapMacosCommands {
    /// Manage macOS defaults from `[bootstrap.macos.defaults]`
    #[arg(name = "defaults")]
    Defaults(Box<BootstrapMacosDefaultsArgs2>),
    /// Manage macOS LaunchAgents from `[bootstrap.macos.launchd.agents]`
    #[arg(name = "launchd-agents", alias_hidden = "launchd")]
    LaunchdAgents(Box<BootstrapMacosLaunchdAgentsArgs>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosDefaultsApplyArgs2 {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosDefaultsStatusArgs2 {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured defaults are not in their desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage macOS defaults from `[bootstrap.macos.defaults]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMacosDefaultsArgs {
    #[arg(subcommand)]
    pub command: BootstrapMacosDefaultsCommands,
}

#[derive(Subcommand)]
pub enum BootstrapMacosDefaultsCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapMacosDefaultsApplyArgs2>),
    #[arg(name = "status")]
    Status(Box<BootstrapMacosDefaultsStatusArgs2>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMiseShellActivateApplyArgs {
    /// Print the actions that would run without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMiseShellActivateStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured shell activation is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage mise shell activation from `[bootstrap.mise_shell_activate]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapMiseShellActivateArgs {
    #[arg(subcommand)]
    pub command: BootstrapMiseShellActivateCommands,
}

#[derive(Subcommand)]
pub enum BootstrapMiseShellActivateCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapMiseShellActivateApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapMiseShellActivateStatusArgs>),
}

/// Apply system packages from `[bootstrap.packages]`
///
/// Checks which configured packages are missing and installs them with the system package manager. Built-in system managers may elevate with sudo when not running as root (see `system_packages.sudo`); package plugins never do.
///
/// Packages can also be given explicitly in `manager:package` form (e.g. `apk:zlib-dev`, `apt:curl`, `brew:jq`); they are installed whether or not they appear in the config. Explicit packages and `--manager` scope the run to packages only. `install` is accepted as an alias for this command.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages apply\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages apply apk:zlib-dev apt:curl brew:jq brew-cask:firefox flatpak:org.mozilla.firefox flatpak-user:org.gnome.Builder mas:497799835\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages apply --manager apt --yes\u{1b}[22m\n"
)]
pub struct BootstrapPackagesApplyArgs {
    /// Only install packages for this built-in or plugin manager
    #[arg(long = "manager", short = 'm', value_name = "MANAGER")]
    pub manager: ::std::option::Option<::std::string::String>,
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Refresh package manager metadata first (apk: `--update-cache`, apt: `apt-get update`)
    #[arg(long = "update")]
    pub update: bool,
    #[arg(
        positional,
        value_name = "PACKAGE",
        help = "Packages in `manager:package` form; defaults to everything configured in [bootstrap.packages]",
        long_help = "Packages in `manager:package` form; defaults to everything configured\nin [bootstrap.packages]"
    )]
    pub package: ::std::vec::Vec<::std::string::String>,
}

/// Add a Homebrew tap URL to [bootstrap.brew.taps]
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages brew tap railwaycat/emacsmacport\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages brew tap acme/tools https://github.com/acme/homebrew-tools.git\u{1b}[22m\n"
)]
pub struct BootstrapPackagesBrewTapArgs {
    /// Write to the local config instead of the global config
    #[arg(long = "local", short = 'l')]
    pub local: bool,
    /// Print the config change without writing it
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Write to this config file or directory
    #[arg(
        long = "path",
        long = "file",
        short = 'p',
        conflicts("--local"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Tap name, e.g. `owner/repo`
    #[arg(positional, value_name = "TAP")]
    pub tap: ::std::string::String,
    /// GitHub URL for the tap. Defaults to https://github.com/<owner>/homebrew-<repo>.git
    #[arg(positional, value_name = "URL")]
    pub url: ::std::option::Option<::std::string::String>,
}

/// Remove Homebrew tap URLs from [bootstrap.brew.taps]
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages brew untap railwaycat/emacsmacport\u{1b}[22m\n"
)]
pub struct BootstrapPackagesBrewUntapArgs {
    /// Write to the local config instead of the global config
    #[arg(long = "local", short = 'l')]
    pub local: bool,
    /// Print the config change without writing it
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Write to this config file or directory
    #[arg(
        long = "path",
        long = "file",
        short = 'p',
        conflicts("--local"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Tap name(s), e.g. `owner/repo`
    #[arg(positional, value_name = "TAPS", required)]
    pub taps: ::std::vec::Vec<::std::string::String>,
}

/// Manage Homebrew taps used by bootstrap packages
///
/// These commands edit `[bootstrap.brew.taps]` so tapped formulae and casks can be fetched directly by mise without a Homebrew installation.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapPackagesBrewArgs {
    #[arg(subcommand)]
    pub command: BootstrapPackagesBrewCommands,
}

#[derive(Subcommand)]
pub enum BootstrapPackagesBrewCommands {
    /// Add a Homebrew tap URL to [bootstrap.brew.taps]
    #[arg(name = "tap")]
    Tap(Box<BootstrapPackagesBrewTapArgs>),
    /// Remove Homebrew tap URLs from [bootstrap.brew.taps]
    #[arg(name = "untap", alias("remove", "rm"))]
    Untap(Box<BootstrapPackagesBrewUntapArgs>),
}

/// Import installed system packages into `[bootstrap.packages]`
///
/// Currently supports Homebrew formulae only. By default, imports linked formulae whose active keg receipt says they were installed on request. Pass `--all` to import every linked formula, including dependencies.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages import --manager brew\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages import --manager brew --all\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages import --manager brew --global\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages import --manager brew --dry-run\u{1b}[22m\n"
)]
pub struct BootstrapPackagesImportArgs {
    /// Write to the config file for this environment (mise.<ENV>.toml)
    #[arg(
        long = "env",
        short = 'e',
        conflicts("--global", "--path"),
        value_name = "ENV"
    )]
    pub env: ::std::option::Option<::std::string::String>,
    /// Write to the global config (~/.config/mise/config.toml)
    #[arg(long = "global", short = 'g', conflicts("--env", "--path"))]
    pub global: bool,
    /// Only import packages for this manager. Currently only `brew` is supported.
    #[arg(
        long = "manager",
        short = 'm',
        value_name = "MANAGER",
        choices("brew"),
        default = "brew"
    )]
    pub manager: ::std::option::Option<::std::string::String>,
    /// Import every linked formula, including dependencies
    #[arg(long = "all")]
    pub all: bool,
    /// Print the config change without writing config
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Write to this config file or directory
    #[arg(
        long = "path",
        long = "file",
        short = 'p',
        conflicts("--global"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
}

/// Prune installed system packages no longer declared in `[bootstrap.packages]`
///
/// Supports Homebrew formulae and conservatively removable, mise-owned casks. Pruning keeps packages needed by the current config or by trusted, loadable tracked configs.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages prune --manager brew\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages prune --manager brew --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages prune --manager brew --yes\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages prune --manager brew-cask --dry-run\u{1b}[22m\n"
)]
pub struct BootstrapPackagesPruneArgs {
    /// Only prune packages for this manager
    #[arg(
        long = "manager",
        short = 'm',
        value_name = "MANAGER",
        choices("brew", "brew-cask"),
        default = "brew"
    )]
    pub manager: ::std::option::Option<::std::string::String>,
    /// Print what would be removed without deleting anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

/// Show the status of system packages from `[bootstrap.packages]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages status\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages status --json\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages status --missing\u{1b}[22m # exit 1 if anything is out of sync\n"
)]
pub struct BootstrapPackagesStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured packages are not in their desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Upgrade installed bootstrap packages from `[bootstrap.packages]`
///
/// Refreshes package manager metadata and upgrades the configured packages that are already installed: apk/apt/dnf/pacman upgrade to the newest available version (apk, apt, and dnf honor a version pinned in config), brew pours the formula's current bottle and replaces the old keg, brew-cask installs the current cask artifact, flatpak and flatpak-user update applications and runtimes, and mas upgrades App Store apps. Packages that are not installed yet are skipped — use `mise bootstrap packages apply` for those.
///
/// Packages can also be given explicitly in `manager:package` form.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages upgrade\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages upgrade brew:postgresql@17\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages upgrade --manager brew-cask\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages upgrade --manager mas\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages upgrade --manager apt --yes\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages upgrade --dry-run\u{1b}[22m\n"
)]
pub struct BootstrapPackagesUpgradeArgs {
    /// Only upgrade packages for this built-in or plugin manager
    #[arg(long = "manager", short = 'm', value_name = "MANAGER")]
    pub manager: ::std::option::Option<::std::string::String>,
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    #[arg(
        positional,
        value_name = "PACKAGE",
        help = "Packages in `manager:package` form; defaults to everything configured in [bootstrap.packages]",
        long_help = "Packages in `manager:package` form; defaults to everything configured\nin [bootstrap.packages]"
    )]
    pub package: ::std::vec::Vec<::std::string::String>,
}

/// Add bootstrap packages to [bootstrap.packages] and install them
///
/// Like `mise use` for tools: writes `"manager:package" = "version"` entries to mise.toml (the local config by default, the global one with `-g`) and then installs whatever is missing.
///
/// Versions are pinned with `@`: `mise bootstrap packages use apt:curl@8.5.0-2`. Without `@` (or with `@latest`) no pin is written. brew formulae and casks version through their names instead (for example `brew:postgresql@17`, `brew-cask:temurin@17`), where `@` is part of the Homebrew name rather than a mise version selector. mas uses numeric ADAM IDs and does not support pins.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap packages use apk:zlib-dev apt:curl brew:jq brew-cask:firefox flatpak:org.mozilla.firefox flatpak-user:org.gnome.Builder mas:497799835\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages use -g brew:postgresql@17\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages use apt:curl@8.5.0-2\u{1b}[22m\n"
)]
pub struct BootstrapPackagesUseArgs {
    /// Write to the config file for this environment (mise.<ENV>.toml)
    #[arg(
        long = "env",
        short = 'e',
        conflicts("--global", "--path"),
        value_name = "ENV"
    )]
    pub env: ::std::option::Option<::std::string::String>,
    #[arg(
        help = "Write to the global config (~/.config/mise/config.toml) instead of the local one",
        long_help = "Write to the global config (~/.config/mise/config.toml) instead of the\nlocal one",
        long = "global",
        short = 'g'
    )]
    pub global: bool,
    /// Print the commands that would run without writing config or installing
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Write to this config file or directory
    #[arg(
        long = "path",
        long = "file",
        short = 'p',
        conflicts("--global"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Packages in `manager:package[@version]` form
    #[arg(positional, value_name = "PACKAGE", required)]
    pub package: ::std::vec::Vec<::std::string::String>,
}

/// Manage bootstrap system packages from `[bootstrap.packages]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapPackagesArgs {
    #[arg(subcommand)]
    pub command: BootstrapPackagesCommands,
}

#[derive(Subcommand)]
pub enum BootstrapPackagesCommands {
    /// Apply system packages from `[bootstrap.packages]`
    #[arg(name = "apply", alias = "i", alias_hidden = "install")]
    Apply(Box<BootstrapPackagesApplyArgs>),
    /// Manage Homebrew taps used by bootstrap packages
    #[arg(name = "brew")]
    Brew(Box<BootstrapPackagesBrewArgs>),
    /// Import installed system packages into `[bootstrap.packages]`
    #[arg(name = "import")]
    Import(Box<BootstrapPackagesImportArgs>),
    /// Prune installed system packages no longer declared in `[bootstrap.packages]`
    #[arg(name = "prune")]
    Prune(Box<BootstrapPackagesPruneArgs>),
    /// Show the status of system packages from `[bootstrap.packages]`
    #[arg(name = "status", alias = "ls")]
    Status(Box<BootstrapPackagesStatusArgs>),
    /// Upgrade installed bootstrap packages from `[bootstrap.packages]`
    #[arg(name = "upgrade", alias = "up")]
    Upgrade(Box<BootstrapPackagesUpgradeArgs>),
    /// Add bootstrap packages to [bootstrap.packages] and install them
    #[arg(name = "use", alias = "u")]
    Use(Box<BootstrapPackagesUseArgs>),
}

/// Show the changes declarative bootstrap resources would make
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapPlanArgs {
    /// Output a stable machine-readable plan in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit 2 when the plan contains changes, 0 when unchanged, and 1 on errors
    #[arg(long = "detailed-exitcode")]
    pub detailed_exitcode: bool,
    /// Prompt securely for missing bootstrap secret inputs
    #[arg(long = "prompt-secrets")]
    pub prompt_secrets: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapPluginsApplyArgs {
    /// Print what would happen without installing plugins
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapPluginsStatusArgs {
    /// Exit with code 1 if a declared plugin is missing
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage package manager plugins declared in `[bootstrap.plugins]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapPluginsArgs {
    #[arg(subcommand)]
    pub command: BootstrapPluginsCommands,
}

#[derive(Subcommand)]
pub enum BootstrapPluginsCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapPluginsApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapPluginsStatusArgs>),
}

/// Bootstrap one or more machines over OpenSSH
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapRemoteArgs {
    /// Select every configured inventory host
    #[arg(long = "all")]
    pub all: bool,
    /// Explicit remote shell command that installs mise and places it on PATH
    #[arg(
        long = "bootstrap-command",
        conflicts("--mise-bin", "--remote-mise"),
        value_name = "COMMAND"
    )]
    pub bootstrap_command: ::std::option::Option<::std::string::String>,
    /// SSH connection timeout in seconds
    #[arg(
        long = "connect-timeout",
        value_name = "CONNECT_TIMEOUT",
        default = "10"
    )]
    pub connect_timeout: ::std::option::Option<::std::string::String>,
    /// Dereference one source-relative symbolic link; repeat for multiple links
    #[arg(long = "copy-link", value_name = "PATH")]
    pub copy_link: ::std::vec::Vec<::std::string::String>,
    /// Dereference every symbolic link in the source archive
    #[arg(long = "copy-links")]
    pub copy_links: bool,
    /// Additional archive pattern to exclude; repeat for multiple patterns
    #[arg(long = "exclude", value_name = "PATTERN")]
    pub exclude: ::std::vec::Vec<::std::string::String>,
    /// Stop after the first failed target
    #[arg(long = "fail-fast")]
    pub fail_fast: bool,
    /// Allow remote dotfile conflicts to be replaced
    #[arg(long = "force-dotfiles")]
    pub force_dotfiles: bool,
    /// Ad-hoc SSH destination (`[user@]host`); repeat for multiple hosts
    #[arg(long = "host", value_name = "[USER@]HOST")]
    pub host: ::std::vec::Vec<::std::string::String>,
    /// SSH identity file override
    #[arg(long = "identity-file", short = 'i', value_name = "IDENTITY_FILE")]
    pub identity_file: ::std::option::Option<::std::string::String>,
    /// Print the remote bootstrap changes without applying them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Keep the remote staging directory for debugging
    #[arg(long = "keep-staging")]
    pub keep_staging: bool,
    /// Local mise binary to upload (escape hatch for custom architectures)
    #[arg(
        long = "mise-bin",
        conflicts("--remote-mise", "--bootstrap-command"),
        value_name = "MISE_BIN"
    )]
    pub mise_bin: ::std::option::Option<::std::string::String>,
    /// Run only one or more remote bootstrap parts
    #[arg(
        long = "only",
        conflicts("--skip"),
        delimiter = ',',
        value_name = "ONLY"
    )]
    pub only: ::std::vec::Vec<BootstrapRemoteOnlyValue>,
    /// SSH port override
    #[arg(long = "port", value_name = "PORT")]
    pub port: ::std::option::Option<::std::string::String>,
    /// Prompt securely for missing secret inputs on the remote host
    #[arg(long = "prompt-secrets")]
    pub prompt_secrets: bool,
    /// Config environments to load on the remote host; repeat or delimit with commas (for example, ci,dotfiles)
    #[arg(long = "remote-env", delimiter = ',', value_name = "ENV")]
    pub remote_env: ::std::vec::Vec<::std::string::String>,
    /// Existing mise executable name or path; relative paths use the staged project
    #[arg(
        long = "remote-mise",
        conflicts("--mise-bin", "--bootstrap-command"),
        value_name = "COMMAND"
    )]
    pub remote_mise: ::std::option::Option<::std::string::String>,
    /// Skip one or more remote bootstrap parts
    #[arg(long = "skip", delimiter = ',', value_name = "SKIP")]
    pub skip: ::std::vec::Vec<BootstrapRemoteSkipValue>,
    /// Local directory archived and sent to each target
    #[arg(long = "source", value_name = "SOURCE")]
    pub source: ::std::option::Option<::std::string::String>,
    /// OpenSSH `-o` option; repeat for multiple options
    #[arg(long = "ssh-option", value_name = "OPTION")]
    pub ssh_option: ::std::vec::Vec<::std::string::String>,
    /// Select configured hosts with this tag; repeat to match any tag
    #[arg(long = "tag", value_name = "TAG")]
    pub tag: ::std::vec::Vec<::std::string::String>,
    /// Refresh package manager metadata and update configured repos remotely
    #[arg(long = "update")]
    pub update: bool,
    /// Skip remote confirmation prompts
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Inventory host names from `[bootstrap.remote.hosts]`
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

#[derive(ValueEnum)]
pub enum BootstrapRemoteOnlyValue {
    #[arg(name = "plugins")]
    Plugins,
    #[arg(name = "packages")]
    Packages,
    #[arg(name = "accounts")]
    Accounts,
    #[arg(name = "files")]
    Files,
    #[arg(name = "services")]
    Services,
    #[arg(name = "firewall")]
    Firewall,
    #[arg(name = "compose")]
    Compose,
    #[arg(name = "repos")]
    Repos,
    #[arg(name = "dotfiles")]
    Dotfiles,
    #[arg(name = "mise-shell-activate", alias = "shell")]
    MiseShellActivate,
    #[arg(name = "macos-defaults", alias = "defaults")]
    MacosDefaults,
    #[arg(name = "macos-launchd-agents", alias = "launchd")]
    MacosLaunchdAgents,
    #[arg(name = "linux-systemd-units", alias = "systemd")]
    LinuxSystemdUnits,
    #[arg(name = "user")]
    User,
    #[arg(name = "tools")]
    Tools,
    #[arg(name = "task")]
    Task,
    #[arg(name = "final-hook")]
    FinalHook,
}

#[derive(ValueEnum)]
pub enum BootstrapRemoteSkipValue {
    #[arg(name = "plugins")]
    Plugins,
    #[arg(name = "packages")]
    Packages,
    #[arg(name = "accounts")]
    Accounts,
    #[arg(name = "files")]
    Files,
    #[arg(name = "services")]
    Services,
    #[arg(name = "firewall")]
    Firewall,
    #[arg(name = "compose")]
    Compose,
    #[arg(name = "repos")]
    Repos,
    #[arg(name = "dotfiles")]
    Dotfiles,
    #[arg(name = "mise-shell-activate", alias = "shell")]
    MiseShellActivate,
    #[arg(name = "macos-defaults", alias = "defaults")]
    MacosDefaults,
    #[arg(name = "macos-launchd-agents", alias = "launchd")]
    MacosLaunchdAgents,
    #[arg(name = "linux-systemd-units", alias = "systemd")]
    LinuxSystemdUnits,
    #[arg(name = "user")]
    User,
    #[arg(name = "tools")]
    Tools,
    #[arg(name = "task")]
    Task,
    #[arg(name = "final-hook")]
    FinalHook,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapReposApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapReposExecArgs {
    /// Continue running in other repos after a command fails
    #[arg(long = "continue-on-error", short = 'c')]
    pub continue_on_error: bool,
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Run only in matching configured or expanded paths
    #[arg(positional, value_name = "PATH")]
    pub path: ::std::vec::Vec<::std::string::String>,
    /// Command and arguments to run in each repo
    #[arg(positional, value_name = "COMMAND", required, double_dash = "required")]
    pub command: ::std::vec::Vec<::std::string::String>,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapReposStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured repo is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapReposUpdateArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Update only matching configured or expanded paths
    #[arg(positional, value_name = "PATH")]
    pub path: ::std::vec::Vec<::std::string::String>,
}

/// Manage git repo checkouts from `[bootstrap.repos]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapReposArgs {
    #[arg(subcommand)]
    pub command: BootstrapReposCommands,
}

#[derive(Subcommand)]
pub enum BootstrapReposCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapReposApplyArgs>),
    #[arg(name = "exec")]
    Exec(Box<BootstrapReposExecArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapReposStatusArgs>),
    #[arg(name = "update")]
    Update(Box<BootstrapReposUpdateArgs>),
}

/// Show whether declared bootstrap secret inputs are available
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapSecretsStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if a declared secret input is unavailable
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Inspect bootstrap secret inputs without revealing their values
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapSecretsArgs {
    #[arg(subcommand)]
    pub command: BootstrapSecretsCommands,
}

#[derive(Subcommand)]
pub enum BootstrapSecretsCommands {
    /// Show whether declared bootstrap secret inputs are available
    #[arg(name = "status")]
    Status(Box<BootstrapSecretsStatusArgs>),
}

/// Apply configured Linux system service state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapServicesApplyArgs {
    /// Print what would change without changing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

/// Show configured Linux system service state
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapServicesStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 when any service is not converged
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage Linux system services from `[bootstrap.services]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapServicesArgs {
    #[arg(subcommand)]
    pub command: BootstrapServicesCommands,
}

#[derive(Subcommand)]
pub enum BootstrapServicesCommands {
    /// Apply configured Linux system service state
    #[arg(name = "apply")]
    Apply(Box<BootstrapServicesApplyArgs>),
    /// Show configured Linux system service state
    #[arg(name = "status")]
    Status(Box<BootstrapServicesStatusArgs>),
}

/// Show the aggregate bootstrap status
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured bootstrap state is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
    /// Prompt securely for missing bootstrap secret inputs
    #[arg(long = "prompt-secrets")]
    pub prompt_secrets: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapSystemdApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapSystemdStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured systemd user service is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage systemd user services from `[bootstrap.linux.systemd.units]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapSystemdArgs {
    #[arg(subcommand)]
    pub command: BootstrapSystemdCommands,
}

#[derive(Subcommand)]
pub enum BootstrapSystemdCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapSystemdApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapSystemdStatusArgs>),
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapUserApplyArgs {
    /// Print the commands that would run without running them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapUserStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Exit with code 1 if any configured user setting is not in its desired state
    #[arg(long = "missing")]
    pub missing: bool,
}

/// Manage current-user bootstrap settings from `[bootstrap.user]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct BootstrapUserArgs {
    #[arg(subcommand)]
    pub command: BootstrapUserCommands,
}

#[derive(Subcommand)]
pub enum BootstrapUserCommands {
    #[arg(name = "apply")]
    Apply(Box<BootstrapUserApplyArgs>),
    #[arg(name = "status")]
    Status(Box<BootstrapUserStatusArgs>),
}

/// Set up a machine for the current config in one command
///
/// Runs the bootstrap steps for the current config in order:
///
/// 0. `mise bootstrap accounts apply` — converge `[bootstrap.users]` and
///    `[bootstrap.groups]` (Linux)
/// 1. `mise bootstrap plugins apply` — install `[bootstrap.plugins]`
///    1.7. `[bootstrap.hooks.pre-packages]` — optional setup hook
/// 2. Install built-in-manager entries from `[bootstrap.packages]` 3. `mise bootstrap files apply` — converge `[bootstrap.files]` and
///    `[bootstrap.directories]`
/// 4. `mise bootstrap services apply` — converge `[bootstrap.services]`
///    systemd system services (Linux)
/// 5. `mise bootstrap firewall apply` — converge `[bootstrap.linux.firewall]`
///    host firewall policy and rules (Linux)
/// 6. `mise bootstrap compose apply` — converge `[bootstrap.compose]`
///    Docker Compose projects
/// 7. `mise bootstrap repos apply` — clone/converge `[bootstrap.repos]`
///    surrounded by `pre-repos`/`post-repos` hooks
/// 8. `mise bootstrap dotfiles apply` — apply dotfiles from `[dotfiles]`
///    surrounded by `pre-dotfiles`/`post-dotfiles` hooks
/// 9. `mise bootstrap mise-shell-activate apply` — configure shell activation
///    from `[bootstrap.mise_shell_activate]`
/// 10. `mise bootstrap macos defaults apply` — write
///     `[bootstrap.macos.defaults]` entries (macOS)
///     surrounded by `pre-defaults`/`post-defaults` hooks
/// 11. `mise bootstrap macos launchd-agents apply` — install/load
///     `[bootstrap.macos.launchd.agents]`
/// 12. `mise bootstrap linux systemd-units apply` — install/start
///     `[bootstrap.linux.systemd.units]`
/// 13. `mise bootstrap user apply` — set `[bootstrap.user].login_shell`
///     (Unix)
///     surrounded by `pre-user`/`post-user` hooks
/// 14. `mise install` — install missing tools from `[tools]`
///     surrounded by `pre-tools`/`post-tools` hooks; package-plugin entries
///     from `[bootstrap.packages]` install afterward, followed by
///     `[bootstrap.hooks.post-packages]`
/// 15. `mise run bootstrap` — if a task named `bootstrap` is defined 16. `[bootstrap.hooks.final]` — optional final hook
///
/// The declarative steps converge — anything already in its desired state is skipped, so re-running is safe. The `bootstrap` task runs on every invocation; keep it idempotent. Use it for any project-specific setup that doesn't fit the declarative sections (seeding databases, auth flows, etc.) — it runs with the installed tools on PATH.
///
/// Use `--skip <part>` to skip named parts, or `--only <part>` to run just named parts. Both flags can be repeated or comma-separated, but they cannot be used together.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap\u{1b}[22m                    # packages + repos + dotfiles + tools + bootstrap task\n    $ \u{1b}[1mmise bootstrap --force-dotfiles\u{1b}[22m   # replace conflicting dotfile targets\n    $ \u{1b}[1mmise bootstrap --skip tools,task\u{1b}[22m  # skip tool installation and the bootstrap task\n    $ \u{1b}[1mmise bootstrap --only tools\u{1b}[22m       # run just tool installation\n    $ \u{1b}[1mmise bootstrap status --missing\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap packages apply --yes\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap repos status\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap repos apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap mise-shell-activate apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap macos defaults status\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap macos launchd-agents apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap linux systemd-units apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap user apply --dry-run\u{1b}[22m\n"
)]
pub struct BootstrapArgs {
    /// Print what would happen without installing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip confirmation prompts
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Overwrite existing files that conflict with whole-file dotfile entries
    #[arg(long = "force-dotfiles")]
    pub force_dotfiles: bool,
    /// Run only one or more bootstrap parts
    ///
    /// Can be passed multiple times or as a comma-separated list. Cannot be used with `--skip`.
    #[arg(
        long = "only",
        conflicts("--skip"),
        delimiter = ',',
        value_name = "ONLY"
    )]
    pub only: ::std::vec::Vec<BootstrapOnlyValue>,
    /// Prompt securely for missing bootstrap secret inputs
    #[arg(long = "prompt-secrets")]
    pub prompt_secrets: bool,
    /// Skip one or more bootstrap parts
    ///
    /// Can be passed multiple times or as a comma-separated list.
    #[arg(long = "skip", delimiter = ',', value_name = "SKIP")]
    pub skip: ::std::vec::Vec<BootstrapSkipValue>,
    /// Refresh package manager metadata and update configured repos
    #[arg(long = "update")]
    pub update: bool,
    #[arg(subcommand)]
    pub command: ::std::option::Option<BootstrapCommands>,
}

#[derive(ValueEnum)]
pub enum BootstrapOnlyValue {
    #[arg(name = "plugins")]
    Plugins,
    #[arg(name = "packages")]
    Packages,
    #[arg(name = "accounts")]
    Accounts,
    #[arg(name = "files")]
    Files,
    #[arg(name = "services")]
    Services,
    #[arg(name = "firewall")]
    Firewall,
    #[arg(name = "compose")]
    Compose,
    #[arg(name = "repos")]
    Repos,
    #[arg(name = "dotfiles")]
    Dotfiles,
    #[arg(name = "mise-shell-activate", alias = "shell")]
    MiseShellActivate,
    #[arg(name = "macos-defaults", alias = "defaults")]
    MacosDefaults,
    #[arg(name = "macos-launchd-agents", alias = "launchd")]
    MacosLaunchdAgents,
    #[arg(name = "linux-systemd-units", alias = "systemd")]
    LinuxSystemdUnits,
    #[arg(name = "user")]
    User,
    #[arg(name = "tools")]
    Tools,
    #[arg(name = "task")]
    Task,
    #[arg(name = "final-hook")]
    FinalHook,
}

#[derive(ValueEnum)]
pub enum BootstrapSkipValue {
    #[arg(name = "plugins")]
    Plugins,
    #[arg(name = "packages")]
    Packages,
    #[arg(name = "accounts")]
    Accounts,
    #[arg(name = "files")]
    Files,
    #[arg(name = "services")]
    Services,
    #[arg(name = "firewall")]
    Firewall,
    #[arg(name = "compose")]
    Compose,
    #[arg(name = "repos")]
    Repos,
    #[arg(name = "dotfiles")]
    Dotfiles,
    #[arg(name = "mise-shell-activate", alias = "shell")]
    MiseShellActivate,
    #[arg(name = "macos-defaults", alias = "defaults")]
    MacosDefaults,
    #[arg(name = "macos-launchd-agents", alias = "launchd")]
    MacosLaunchdAgents,
    #[arg(name = "linux-systemd-units", alias = "systemd")]
    LinuxSystemdUnits,
    #[arg(name = "user")]
    User,
    #[arg(name = "tools")]
    Tools,
    #[arg(name = "task")]
    Task,
    #[arg(name = "final-hook")]
    FinalHook,
}

#[derive(Subcommand)]
pub enum BootstrapCommands {
    #[arg(name = "__apply-account-plan", hide)]
    ApplyAccountPlan(Box<BootstrapApplyAccountPlanArgs>),
    #[arg(name = "__apply-service-plan", hide)]
    ApplyServicePlan(Box<BootstrapApplyServicePlanArgs>),
    #[arg(name = "__apply-firewall-plan", hide)]
    ApplyFirewallPlan(Box<BootstrapApplyFirewallPlanArgs>),
    #[arg(name = "__apply-system-plan", hide)]
    ApplySystemPlan(Box<BootstrapApplySystemPlanArgs>),
    #[arg(name = "__inspect-system-files", hide)]
    InspectSystemFiles(Box<BootstrapInspectSystemFilesArgs>),
    #[arg(name = "__inspect-firewall-plan", hide)]
    InspectFirewallPlan(Box<BootstrapInspectFirewallPlanArgs>),
    /// Manage Linux users and groups from `[bootstrap.users]` and `[bootstrap.groups]`
    #[arg(name = "accounts")]
    Accounts(Box<BootstrapAccountsArgs>),
    /// Manage Docker Compose projects from `[bootstrap.compose]`
    #[arg(name = "compose")]
    Compose(Box<BootstrapComposeArgs>),
    /// Manage dotfiles from `[dotfiles]`
    #[arg(name = "dotfiles")]
    Dotfiles(Box<BootstrapDotfilesArgs>),
    /// Manage privileged files and directories from `[bootstrap.files]` and `[bootstrap.directories]`
    #[arg(name = "files")]
    Files(Box<BootstrapFilesArgs>),
    /// Manage the Linux host firewall from `[bootstrap.linux.firewall]`
    #[arg(name = "firewall")]
    Firewall(Box<BootstrapFirewallArgs>),
    /// Manage macOS LaunchAgents from `[bootstrap.macos.launchd.agents]`
    #[arg(name = "launchd", hide)]
    Launchd(Box<BootstrapLaunchdArgs>),
    /// Manage Linux bootstrap config from `[bootstrap.linux]`
    #[arg(name = "linux")]
    Linux(Box<BootstrapLinuxArgs>),
    /// Manage macOS bootstrap config from `[bootstrap.macos]`
    #[arg(name = "macos")]
    Macos(Box<BootstrapMacosArgs>),
    /// Manage macOS defaults from `[bootstrap.macos.defaults]`
    #[arg(name = "macos-defaults", hide)]
    MacosDefaults(Box<BootstrapMacosDefaultsArgs>),
    /// Manage mise shell activation from `[bootstrap.mise_shell_activate]`
    #[arg(name = "mise-shell-activate", alias_hidden = "shell")]
    MiseShellActivate(Box<BootstrapMiseShellActivateArgs>),
    /// Manage bootstrap system packages from `[bootstrap.packages]`
    #[arg(name = "packages")]
    Packages(Box<BootstrapPackagesArgs>),
    /// Show the changes declarative bootstrap resources would make
    #[arg(name = "plan")]
    Plan(Box<BootstrapPlanArgs>),
    /// Manage package manager plugins declared in `[bootstrap.plugins]`
    #[arg(name = "plugins")]
    Plugins(Box<BootstrapPluginsArgs>),
    /// Bootstrap one or more machines over OpenSSH
    #[arg(name = "remote")]
    Remote(Box<BootstrapRemoteArgs>),
    /// Manage git repo checkouts from `[bootstrap.repos]`
    #[arg(name = "repos")]
    Repos(Box<BootstrapReposArgs>),
    /// Inspect bootstrap secret inputs without revealing their values
    #[arg(name = "secrets")]
    Secrets(Box<BootstrapSecretsArgs>),
    /// Manage Linux system services from `[bootstrap.services]`
    #[arg(name = "services")]
    Services(Box<BootstrapServicesArgs>),
    /// Show the aggregate bootstrap status
    #[arg(name = "status", alias = "ls")]
    Status(Box<BootstrapStatusArgs>),
    /// Manage systemd user services from `[bootstrap.linux.systemd.units]`
    #[arg(name = "systemd", hide)]
    Systemd(Box<BootstrapSystemdArgs>),
    /// Manage current-user bootstrap settings from `[bootstrap.user]`
    #[arg(name = "user")]
    User(Box<BootstrapUserArgs>),
}

/// Deletes all cache files in mise
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct CacheClearArgs {
    /// Mark all cache files as old
    #[arg(long = "outdate", hide)]
    pub outdate: bool,
    /// Clear output cache entries for a task name or pattern
    #[arg(long = "task", conflicts("TOOL", "--outdate"), value_name = "TASK")]
    pub task: ::std::option::Option<::std::string::String>,
    #[arg(
        positional,
        value_name = "TOOL",
        help = "Tool(s) to clear cache for e.g.: node, python",
        long_help = "Tool(s) to clear cache for\ne.g.: node, python"
    )]
    pub tool: ::std::vec::Vec<::std::string::String>,
}

/// Show the cache directory path
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct CachePathArgs {}

/// Removes stale mise cache files
///
/// By default, this command will remove files that have not been accessed in 30 days. Change this with the MISE_CACHE_PRUNE_AGE environment variable.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct CachePruneArgs {
    /// Show pruned files
    #[arg(long = "verbose", short = 'v', count)]
    pub verbose: u8,
    /// Just show what would be pruned
    #[arg(long = "dry-run")]
    pub dry_run: bool,
    #[arg(
        positional,
        value_name = "TOOL",
        help = "Tool(s) to prune cache for e.g.: node, python",
        long_help = "Tool(s) to prune cache for\ne.g.: node, python"
    )]
    pub tool: ::std::vec::Vec<::std::string::String>,
}

/// Inspect output cache entries for a task
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct CacheTaskArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Task name or pattern to inspect
    #[arg(positional, value_name = "TASK")]
    pub task: ::std::string::String,
}

/// Manage the mise cache
///
/// Run `mise cache` with no args to view the current cache directory.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct CacheArgs {
    #[arg(subcommand)]
    pub command: ::std::option::Option<CacheCommands>,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Deletes all cache files in mise
    #[arg(name = "clear", alias = "c", alias_hidden = "clean")]
    Clear(Box<CacheClearArgs>),
    /// Show the cache directory path
    #[arg(name = "path", alias = "dir")]
    Path(Box<CachePathArgs>),
    /// Removes stale mise cache files
    #[arg(name = "prune", alias = "p")]
    Prune(Box<CachePruneArgs>),
    /// Inspect output cache entries for a task
    #[arg(name = "task")]
    Task(Box<CacheTaskArgs>),
}

/// Generate shell completions
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise completion bash --include-bash-completion-lib > ~/.local/share/bash-completion/completions/mise\u{1b}[22m\n    $ \u{1b}[1mmise completion zsh  > /usr/local/share/zsh/site-functions/_mise\u{1b}[22m\n    $ \u{1b}[1mmise completion fish > ~/.config/fish/completions/mise.fish\u{1b}[22m\n    $ \u{1b}[1mmise completion powershell >> $PROFILE\u{1b}[22m\n"
)]
pub struct CompletionArgs {
    /// Shell type to generate completions for
    #[arg(long = "shell", short = 's', hide, value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Include the bash completion library in the bash completion script
    ///
    /// This is required for completions to work in bash, but it is not included by default you may source it separately or enable this flag to enable it in the script.
    #[arg(long = "include-bash-completion-lib")]
    pub include_bash_completion_lib: bool,
    #[arg(
        help = "Always use usage for completions.\nCurrently, usage is the default for fish and bash but not zsh since it has a few quirks\nto work out first.",
        long_help = "Always use usage for completions.\nCurrently, usage is the default for fish and bash but not zsh since it has a few quirks\nto work out first.\n\nThis requires the `usage` CLI to be installed.\nhttps://usage.jdx.dev",
        long = "usage",
        hide
    )]
    pub usage: bool,
    /// Shell type to generate completions for
    #[arg(positional, value_name = "SHELL", required_unless("--shell"))]
    pub shell_2: ::std::option::Option<::std::string::String>,
}

/// Display the value of a setting in a mise.toml file
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise toml get tools.python\u{1b}[22m\n    3.12\n"
)]
pub struct ConfigGetArgs {
    /// The path to the mise.toml file to read
    ///
    /// Can be a file path or directory. If a directory is provided, the config file in that directory is used.
    ///
    /// If not provided, the nearest mise.toml file will be used
    #[arg(long = "file", long = "path", short = 'f', value_name = "FILE")]
    pub file: ::std::option::Option<::std::string::String>,
    /// The path of the config to display
    #[arg(positional, value_name = "KEY")]
    pub key: ::std::option::Option<::std::string::String>,
}

/// List config files currently in use
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise config ls\u{1b}[22m\n    Path                        Tools\n    ~/.config/mise/config.toml  pitchfork\n    ~/src/mise/mise.toml        actionlint, bun, cargo-binstall, cargo:cargo-insta\n"
)]
pub struct ConfigLsArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Do not print table header
    #[arg(long = "no-header", alias = "no-headers")]
    pub no_header: bool,
    /// List all tracked config files
    #[arg(long = "tracked-configs")]
    pub tracked_configs: bool,
}

/// Set the value of a setting in a mise.toml file
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise config set tools.python 3.12\u{1b}[22m\n    $ \u{1b}[1mmise config set settings.always_keep_download true\u{1b}[22m\n    $ \u{1b}[1mmise config set env.TEST_ENV_VAR ABC\u{1b}[22m\n    $ \u{1b}[1mmise config set settings.disable_tools node,rust\u{1b}[22m\n\n    # Type for `settings` is inferred\n    $ \u{1b}[1mmise config set settings.jobs 4\u{1b}[22m\n"
)]
pub struct ConfigSetArgs {
    /// The path to the mise.toml file to edit
    ///
    /// Can be a file path or directory. If a directory is provided, the config file in that directory is used.
    ///
    /// If not provided, the nearest mise.toml file will be used
    #[arg(long = "file", long = "path", short = 'f', value_name = "FILE")]
    pub file: ::std::option::Option<::std::string::String>,
    #[arg(long = "type", short = 't', value_name = "TYPE", default = "infer")]
    pub type_: ::std::option::Option<ConfigSetTypeValue>,
    /// The path of the config to display
    #[arg(positional, value_name = "KEY")]
    pub key: ::std::string::String,
    /// The value to set the key to (optional if provided as KEY=VALUE)
    #[arg(positional, value_name = "VALUE")]
    pub value: ::std::option::Option<::std::string::String>,
}

#[derive(ValueEnum)]
pub enum ConfigSetTypeValue {
    #[arg(name = "infer")]
    Infer,
    #[arg(name = "string")]
    String,
    #[arg(name = "integer")]
    Integer,
    #[arg(name = "float")]
    Float,
    #[arg(name = "bool")]
    Bool,
    #[arg(name = "list")]
    List,
    #[arg(name = "set")]
    Set,
}

/// Manage config files
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct ConfigArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Do not print table header
    #[arg(long = "no-header", alias = "no-headers")]
    pub no_header: bool,
    /// List all tracked config files
    #[arg(long = "tracked-configs")]
    pub tracked_configs: bool,
    #[arg(subcommand)]
    pub command: ::std::option::Option<ConfigCommands>,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Display the value of a setting in a mise.toml file
    #[arg(name = "get")]
    Get(Box<ConfigGetArgs>),
    /// List config files currently in use
    #[arg(name = "ls", alias = "list")]
    Ls(Box<ConfigLsArgs>),
    /// Set the value of a setting in a mise.toml file
    #[arg(name = "set")]
    Set(Box<ConfigSetArgs>),
}

/// Shows current active and installed runtime versions
///
/// This is similar to `mise ls --current`, but this only shows the runtime and/or version. It's designed to fit into scripts more easily.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # outputs `.tool-versions` compatible format\n    $ \u{1b}[1mmise current\u{1b}[22m\n    python 3.11.0 3.10.0\n    shfmt 3.6.0\n    shellcheck 0.9.0\n    node 20.0.0\n\n    $ \u{1b}[1mmise current node\u{1b}[22m\n    20.0.0\n\n    # can output multiple versions\n    $ \u{1b}[1mmise current python\u{1b}[22m\n    3.11.0 3.10.0\n"
)]
pub struct CurrentArgs {
    #[arg(
        positional,
        value_name = "PLUGIN",
        help = "Plugin to show versions of e.g.: ruby, node, cargo:eza, npm:prettier, etc.",
        long_help = "Plugin to show versions of\ne.g.: ruby, node, cargo:eza, npm:prettier, etc."
    )]
    pub plugin: ::std::option::Option<::std::string::String>,
}

/// Disable mise for current shell session
///
/// This can be used to temporarily disable mise in a shell session.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise deactivate\u{1b}[22m\n"
)]
pub struct DeactivateArgs {}

/// Output direnv function to use mise inside direnv
///
/// See https://mise.jdx.dev/direnv.html for more information
///
/// Because this generates the idiomatic files based on currently installed plugins, you should run this command after installing new plugins. Otherwise direnv may not know to update environment variables when idiomatic file versions change.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise direnv activate > ~/.config/direnv/lib/use_mise.sh\u{1b}[22m\n    $ \u{1b}[1mecho 'use mise' > .envrc\u{1b}[22m\n    $ \u{1b}[1mdirenv allow\u{1b}[22m\n"
)]
pub struct DirenvActivateArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DirenvEnvrcArgs {}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DirenvExecArgs {}

/// Output direnv function to use mise inside direnv
///
/// See https://mise.jdx.dev/direnv.html for more information
///
/// Because this generates the idiomatic files based on currently installed plugins, you should run this command after installing new plugins. Otherwise direnv may not know to update environment variables when idiomatic file versions change.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DirenvArgs {
    #[arg(subcommand)]
    pub command: ::std::option::Option<DirenvCommands>,
}

#[derive(Subcommand)]
pub enum DirenvCommands {
    /// Output direnv function to use mise inside direnv
    #[arg(name = "activate", hide)]
    Activate(Box<DirenvActivateArgs>),
    #[arg(
        name = "envrc",
        help = "[internal] This is an internal command that writes an envrc file\nfor direnv to consume.",
        hide
    )]
    Envrc(Box<DirenvEnvrcArgs>),
    #[arg(
        name = "exec",
        help = "[internal] This is an internal command that writes an envrc file\nfor direnv to consume.",
        hide
    )]
    Exec(Box<DirenvExecArgs>),
}

/// Add or update dotfiles in `[dotfiles]`
///
/// If the target is already managed, this updates its source from the live target. Otherwise it creates a `[dotfiles]` entry and seeds the source under `dotfiles.root` unless `--source` is provided.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles add ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles add --mode copy ~/.config/starship.toml\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles add --source dotfiles/gitconfig ~/.gitconfig\u{1b}[22m\n"
)]
pub struct DotfilesAddArgs {
    /// Overwrite existing sources without prompting
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Write to the global config
    #[arg(long = "global", short = 'g', conflicts("--local", "--path"))]
    pub global: bool,
    /// Write to the local config instead of the global config
    #[arg(long = "local", short = 'l', conflicts("--global", "--path"))]
    pub local: bool,
    /// Dotfile mode to write
    #[arg(long = "mode", short = 'm', value_name = "MODE")]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Print the config/source updates without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Add the entry without applying it
    #[arg(long = "no-apply")]
    pub no_apply: bool,
    /// Write to this config file or directory
    #[arg(
        long = "path",
        short = 'p',
        conflicts("--global", "--local"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Source path to use for a single target
    #[arg(long = "source", short = 's', value_name = "PATH")]
    pub source: ::std::option::Option<::std::string::String>,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Targets to add or update
    #[arg(positional, value_name = "TARGET", required)]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Apply dotfiles from `[dotfiles]`
///
/// Applies configured whole-file entries and edits that aren't in their desired state. Whole-file entries may symlink, copy, or render templates. Edit entries manage a marker-delimited block or a single line in a file mise doesn't otherwise own.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles apply\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles apply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles apply --force --yes\u{1b}[22m\n"
)]
pub struct DotfilesApplyArgs {
    /// Overwrite existing files that conflict with whole-file dotfile entries
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Print the actions that would run without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Only apply these targets
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Edit a managed dotfile source
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles edit ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles edit --apply ~/.config/starship.toml\u{1b}[22m\n"
)]
pub struct DotfilesEditArgs {
    /// Apply this target after the editor exits
    #[arg(long = "apply")]
    pub apply: bool,
    /// Dotfile mode to use if the target is not yet managed
    #[arg(long = "mode", short = 'm', value_name = "MODE")]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Source path to use if the target is not yet managed
    #[arg(long = "source", short = 's', value_name = "PATH")]
    pub source: ::std::option::Option<::std::string::String>,
    /// Skip the confirmation prompt when adding an unmanaged target
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Target to edit
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::string::String,
}

/// Show the status of dotfiles from `[dotfiles]`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles status\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status --json\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles status --missing\u{1b}[22m # exit 1 if anything is out of sync\n"
)]
pub struct DotfilesStatusArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    #[arg(
        help = "Exit with code 1 if any configured dotfiles are not in their desired\nstate (missing, source missing, differs)",
        long = "missing"
    )]
    pub missing: bool,
    /// Only show these targets
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Remove dotfiles applied from `[dotfiles]`
///
/// Removes configured whole-file entries and edits while preserving files mise cannot identify as managed. Modified copies, templates, and plain-line edits require `--force`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise bootstrap dotfiles unapply\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles unapply ~/.zshrc\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles unapply --dry-run\u{1b}[22m\n    $ \u{1b}[1mmise bootstrap dotfiles unapply --force --yes\u{1b}[22m\n"
)]
pub struct DotfilesUnapplyArgs {
    /// Remove modified or otherwise ambiguous managed files and lines
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Print the actions that would run without writing anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Skip the confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Only unapply these targets
    #[arg(positional, value_name = "TARGET")]
    pub target: ::std::vec::Vec<::std::string::String>,
}

/// Manage dotfiles from `[dotfiles]` (deprecated)
///
/// Use `mise bootstrap dotfiles` instead.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DotfilesArgs {
    #[arg(subcommand)]
    pub command: DotfilesCommands,
}

#[derive(Subcommand)]
pub enum DotfilesCommands {
    /// Add or update dotfiles in `[dotfiles]`
    #[arg(name = "add", hide)]
    Add(Box<DotfilesAddArgs>),
    /// Apply dotfiles from `[dotfiles]`
    #[arg(name = "apply", hide)]
    Apply(Box<DotfilesApplyArgs>),
    /// Edit a managed dotfile source
    #[arg(name = "edit", hide)]
    Edit(Box<DotfilesEditArgs>),
    /// Show the status of dotfiles from `[dotfiles]`
    #[arg(name = "status", hide, alias = "ls")]
    Status(Box<DotfilesStatusArgs>),
    /// Remove dotfiles applied from `[dotfiles]`
    #[arg(name = "unapply", hide)]
    Unapply(Box<DotfilesUnapplyArgs>),
}

/// Print the current PATH entries mise is providing
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    Get the current PATH entries mise is providing\n    $ mise doctor path\n    /home/user/.local/share/mise/installs/node/24.0.0/bin\n    /home/user/.local/share/mise/installs/rust/1.90.0/bin\n    /home/user/.local/share/mise/installs/python/3.10.0/bin\n"
)]
pub struct DoctorPathArgs {
    /// Print all entries including those not provided by mise
    #[arg(long = "full", short = 'f')]
    pub full: bool,
}

/// Check mise installation for possible problems
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise doctor\u{1b}[22m\n    [WARN] plugin node is not installed\n"
)]
pub struct DoctorArgs {
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    #[arg(subcommand)]
    pub command: ::std::option::Option<DoctorCommands>,
}

#[derive(Subcommand)]
pub enum DoctorCommands {
    /// Print the current PATH entries mise is providing
    #[arg(name = "path", alias_hidden = "paths")]
    Path(Box<DoctorPathArgs>),
}

/// Starts a new shell with the mise environment built from the current configuration
///
/// This is an alternative to `mise activate` that allows you to explicitly start a mise session. It will have the tools and environment variables in the configs loaded. Note that changing directories will not update the mise environment.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise en .\u{1b}[22m\n    $ \u{1b}[1mnode -v\u{1b}[22m\n    v20.0.0\n\n    Skip loading bashrc:\n    $ \u{1b}[1mmise en -s \"bash --norc\"\u{1b}[22m\n\n    Skip loading zshrc:\n    $ \u{1b}[1mmise en -s \"zsh -f\"\u{1b}[22m\n"
)]
pub struct EnArgs {
    /// Shell to start
    ///
    /// Defaults to $SHELL
    #[arg(long = "shell", short = 's', value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Directory to start the shell in
    #[arg(positional, value_name = "DIR", default = ".")]
    pub dir: ::std::option::Option<::std::string::String>,
}

/// Exports env vars to activate mise a single time
///
/// Use this if you don't want to permanently install mise. It's not necessary to use this if you have `mise activate` in your shell rc file.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1meval \"$(mise env -s bash)\"\u{1b}[22m\n    $ \u{1b}[1meval \"$(mise env -s zsh)\"\u{1b}[22m\n    $ \u{1b}[1mmise env -s fish | source\u{1b}[22m\n    $ \u{1b}[1mexecx($(mise env -s xonsh))\u{1b}[22m\n"
)]
pub struct EnvArgs {
    /// Output in dotenv format
    #[arg(long = "dotenv", short = 'D', overrides("--shell"))]
    pub dotenv: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J', overrides("--shell"))]
    pub json: bool,
    /// Shell type to generate environment variables for
    #[arg(long = "shell", short = 's', overrides("--json"), value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Output in JSON format with additional information (source, tool)
    #[arg(long = "json-extended", overrides("--shell"))]
    pub json_extended: bool,
    /// Only show redacted environment variables
    #[arg(long = "redacted")]
    pub redacted: bool,
    /// Only show values of environment variables
    #[arg(long = "values")]
    pub values: bool,
    /// Tool(s) to use
    #[arg(positional, value_name = "TOOL@VERSION")]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Execute a command with tool(s) set
///
/// use this to avoid modifying the shell session or running ad-hoc commands with mise tools set.
///
/// Tools will be loaded from mise.toml, though they can be overridden with <RUNTIME> args Note that only the plugin specified will be overridden, so if a `mise.toml` file includes "node 20" but you run `mise exec python@3.11`; it will still load node@20.
///
/// The "--" separates runtimes from the commands to pass along to the subprocess.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise exec node@20 -- node ./app.js\u{1b}[22m  # launch app.js using node-20.x\n    $ \u{1b}[1mmise x node@20 -- node ./app.js\u{1b}[22m     # shorter alias\n\n    # Specify command as a string:\n    $ \u{1b}[1mmise exec node@20 python@3.11 --command \"node -v && python -V\"\u{1b}[22m\n\n    # Run a command in a different directory:\n    $ \u{1b}[1mmise x -C /path/to/project node@20 -- node ./app.js\u{1b}[22m\n"
)]
pub struct ExecArgs {
    /// Command string to execute
    #[arg(
        long = "command",
        short = 'c',
        conflicts("COMMAND"),
        value_name = "COMMAND"
    )]
    pub command: ::std::option::Option<::std::string::String>,
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1\n[default: 4]",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    #[arg(
        help = "Allow specific env var through (implies --deny-env for everything else)\nSupports wildcards, e.g. --allow-env='MYAPP_*'",
        long = "allow-env",
        value_name = "VAR"
    )]
    pub allow_env: ::std::vec::Vec<::std::string::String>,
    #[arg(
        help = "Allow network to specific host (implies --deny-net for everything else)\nmacOS only in v1; on Linux falls back to allowing all network",
        long = "allow-net",
        value_name = "HOST"
    )]
    pub allow_net: ::std::vec::Vec<::std::string::String>,
    /// Allow reads from specific path (implies --deny-read for everything else)
    #[arg(long = "allow-read", value_name = "PATH")]
    pub allow_read: ::std::vec::Vec<::std::string::String>,
    /// Allow writes to specific path (implies --deny-write for everything else)
    #[arg(long = "allow-write", value_name = "PATH")]
    pub allow_write: ::std::vec::Vec<::std::string::String>,
    /// Block reads, writes, network, and env vars
    #[arg(long = "deny-all")]
    pub deny_all: bool,
    /// Block env var inheritance (only PATH, HOME, USER, SHELL, TERM, LANG pass through)
    #[arg(long = "deny-env")]
    pub deny_env: bool,
    /// Block all network access
    #[arg(long = "deny-net")]
    pub deny_net: bool,
    /// Block filesystem reads (system libs and tool dirs still accessible)
    #[arg(long = "deny-read")]
    pub deny_read: bool,
    /// Block all filesystem writes
    #[arg(long = "deny-write")]
    pub deny_write: bool,
    /// Bypass the environment cache and recompute the environment
    #[arg(long = "fresh-env")]
    pub fresh_env: bool,
    /// Skip automatic dependency preparation
    #[arg(long = "no-deps")]
    pub no_deps: bool,
    #[arg(
        help = "Connect backend install command stdin/stdout/stderr directly to the terminal Implies --jobs=1",
        long_help = "Connect backend install command stdin/stdout/stderr directly to the terminal\nImplies --jobs=1",
        long = "raw",
        overrides("--jobs")
    )]
    pub raw: bool,
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to start e.g.: node@20 python@3.10",
        long_help = "Tool(s) to start\ne.g.: node@20 python@3.10"
    )]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
    /// Command string to execute (same as --command)
    #[arg(
        positional,
        value_name = "COMMAND",
        conflicts("--command"),
        required_unless("--command"),
        double_dash = "required"
    )]
    pub command_2: ::std::vec::Vec<::std::string::String>,
}

/// Formats mise.toml
///
/// Sorts keys and cleans up whitespace in mise.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise fmt\u{1b}[22m\n"
)]
pub struct FmtArgs {
    /// Format all files from the current directory
    #[arg(long = "all", short = 'a')]
    pub all: bool,
    /// Check if the configs are formatted, no formatting is done
    #[arg(long = "check", short = 'c')]
    pub check: bool,
    #[arg(
        help = "Read config from stdin and write its formatted version into stdout",
        long_help = "Read config from stdin and write its formatted version into\nstdout",
        long = "stdin",
        short = 's'
    )]
    pub stdin: bool,
}

/// Generate a script to download+execute mise
///
/// This is designed to be used in a project where contributors may not have mise installed.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise generate bootstrap --write ./bin/mise\u{1b}[22m\n    $ \u{1b}[1m./bin/mise install\u{1b}[22m                                    \u{1b}[2m# downloads mise to .mise if not already installed\u{1b}[22m\n\n    \u{1b}[2m# add a launcher for contributors who clone the project on Windows\u{1b}[22m\n    $ \u{1b}[1mmise generate bootstrap --write ./bin/mise --windows\u{1b}[22m  \u{1b}[2m# also writes bin/mise.cmd\u{1b}[22m\n    $ \u{1b}[1m.\\bin\\mise.cmd install\u{1b}[22m\n"
)]
pub struct GenerateBootstrapArgs {
    /// Sandboxes mise internal directories like MISE_DATA_DIR and MISE_CACHE_DIR into a `.mise` directory in the project
    ///
    /// This is necessary if users may use a different version of mise outside the project.
    #[arg(long = "localize", short = 'l')]
    pub localize: bool,
    /// Specify mise version to fetch
    #[arg(long = "version", short = 'V', value_name = "VERSION")]
    pub version: ::std::option::Option<::std::string::String>,
    /// instead of outputting the script to stdout, write to a file and make it executable
    #[arg(
        long = "write",
        short = 'w',
        value_optional,
        default_missing = "./bin/mise",
        value_name = "WRITE"
    )]
    pub write: ::std::option::Option<::std::string::String>,
    /// Directory to put localized data into
    #[arg(
        long = "localized-dir",
        value_name = "LOCALIZED_DIR",
        default = ".mise"
    )]
    pub localized_dir: ::std::option::Option<::std::string::String>,
    /// Also write a Windows launcher, `<WRITE>.cmd`
    ///
    /// Windows cannot execute the `#!/usr/bin/env bash` script, so a contributor who clones the project on Windows has nothing to run without this.
    ///
    /// Generated on every host, not only on Windows: the file is committed, and whoever runs it on Windows is not the person who generated it. Requires `--write`, since stdout cannot carry two files.
    #[arg(long = "windows", requires("--write"))]
    pub windows: bool,
}

/// Generate a mise.toml file
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise generate config\u{1b}[22m             \u{1b}[2m# generate mise.toml interactively\u{1b}[22m\n    $ \u{1b}[1mmise generate config .mise.toml\u{1b}[22m  \u{1b}[2m# generate a specific file\u{1b}[22m\n    $ \u{1b}[1mmise generate config -g\u{1b}[22m          \u{1b}[2m# generate the global config file\u{1b}[22m\n    $ \u{1b}[1mmise generate config -y\u{1b}[22m          \u{1b}[2m# skip interactive editor\u{1b}[22m\n    $ \u{1b}[1mmise generate config -n\u{1b}[22m          \u{1b}[2m# preview without writing\u{1b}[22m\n"
)]
pub struct GenerateConfigArgs {
    /// Generate the global config file (~/.config/mise/config.toml)
    #[arg(long = "global", short = 'g', conflicts("PATH"))]
    pub global: bool,
    /// Show what would be generated without writing to file
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Path to a .tool-versions file to import tools from
    #[arg(long = "tool-versions", short = 't', value_name = "TOOL_VERSIONS")]
    pub tool_versions: ::std::option::Option<::std::string::String>,
    /// Path to the config file to create
    #[arg(positional, value_name = "PATH")]
    pub path: ::std::option::Option<::std::string::String>,
}

/// Generate a devcontainer to execute mise
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise generate devcontainer\u{1b}[22m\n"
)]
pub struct GenerateDevcontainerArgs {
    /// The image to use for the devcontainer
    #[arg(long = "image", short = 'i', value_name = "IMAGE")]
    pub image: ::std::option::Option<::std::string::String>,
    /// Bind the mise-data-volume to the devcontainer
    #[arg(long = "mount-mise-data", short = 'm')]
    pub mount_mise_data: bool,
    /// The name of the devcontainer
    #[arg(long = "name", short = 'n', value_name = "NAME")]
    pub name: ::std::option::Option<::std::string::String>,
    /// write to .devcontainer/devcontainer.json
    #[arg(long = "write", short = 'w')]
    pub write: bool,
}

/// Generate a git pre-commit hook
///
/// This command generates a git pre-commit hook that runs a mise task like `mise run pre-commit` when you commit changes to your repository.
///
/// Staged files are passed to the task as `STAGED`.
///
/// For more advanced pre-commit functionality, see mise's sister project: https://hk.jdx.dev/
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise generate git-pre-commit --write --task=pre-commit\u{1b}[22m\n    $ \u{1b}[1mgit commit -m \"feat: add new feature\"\u{1b}[22m \u{1b}[2m# runs `mise run pre-commit`\u{1b}[22m\n\n    \u{1b}[2m# config lives in a subdirectory, so the hook has to change into it first\u{1b}[22m\n    $ \u{1b}[1mmise generate git-pre-commit --write -- -C subdir\u{1b}[22m\n"
)]
pub struct GenerateGitPreCommitArgs {
    /// The task to run when the pre-commit hook is triggered
    #[arg(
        long = "task",
        short = 't',
        value_name = "TASK",
        default = "pre-commit"
    )]
    pub task: ::std::option::Option<::std::string::String>,
    /// write to .git/hooks/pre-commit and make it executable
    #[arg(long = "write", short = 'w')]
    pub write: bool,
    /// Which hook to generate (saves to .git/hooks/$hook)
    #[arg(long = "hook", value_name = "HOOK", default = "pre-commit")]
    pub hook: ::std::option::Option<::std::string::String>,
    /// mise flags to embed in the generated hook, given after `--`
    ///
    /// These are inserted between `mise` and `run`, so the hook carries the same context you would pass on the command line. Useful when the config is not at the repository root, since git runs hooks from the top level: `-- -C subdir` makes the hook find it.
    #[arg(positional, value_name = "MISE_ARG", double_dash = "required")]
    pub mise_arg: ::std::vec::Vec<::std::string::String>,
}

/// Generate a GitHub Action workflow file
///
/// This command generates a GitHub Action workflow file that runs a mise task like `mise run ci` when you push changes to your repository.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise generate github-action --write --task=ci\u{1b}[22m\n    $ \u{1b}[1mgit commit -m \"feat: add new feature\"\u{1b}[22m\n    $ \u{1b}[1mgit push\u{1b}[22m \u{1b}[2m# runs `mise run ci` on GitHub\u{1b}[22m\n"
)]
pub struct GenerateGithubActionArgs {
    /// The task to run when the workflow is triggered
    #[arg(long = "task", short = 't', value_name = "TASK", default = "ci")]
    pub task: ::std::option::Option<::std::string::String>,
    /// write to .github/workflows/$name.yml
    #[arg(long = "write", short = 'w')]
    pub write: bool,
    /// the name of the workflow to generate
    #[arg(long = "name", value_name = "NAME", default = "ci")]
    pub name: ::std::option::Option<::std::string::String>,
}

/// Generate documentation for tasks in a project
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise generate task-docs\u{1b}[22m\n"
)]
pub struct GenerateTaskDocsArgs {
    /// inserts the documentation into an existing file
    ///
    /// This will look for a special comment, `<!-- mise-tasks -->`, and replace it with the generated documentation. It will replace everything between the comment and the next comment, `<!-- /mise-tasks -->` so it can be run multiple times on the same file to update the documentation. The file must already contain both comments; mise errors instead of modifying the file if they are missing.
    #[arg(long = "inject", short = 'i')]
    pub inject: bool,
    /// write only an index of tasks, intended for use with `--multi`
    #[arg(long = "index", short = 'I')]
    pub index: bool,
    /// render each task as a separate document, requires `--output` to be a directory
    #[arg(long = "multi", short = 'm')]
    pub multi: bool,
    /// writes the generated docs to a file/directory
    #[arg(long = "output", short = 'o', value_name = "OUTPUT")]
    pub output: ::std::option::Option<::std::string::String>,
    /// root directory to search for tasks
    #[arg(long = "root", short = 'r', value_name = "ROOT")]
    pub root: ::std::option::Option<::std::string::String>,
    #[arg(long = "style", short = 's', value_name = "STYLE", default = "simple")]
    pub style: ::std::option::Option<GenerateTaskDocsStyleValue>,
}

#[derive(ValueEnum)]
pub enum GenerateTaskDocsStyleValue {
    #[arg(name = "simple")]
    Simple,
    #[arg(name = "detailed")]
    Detailed,
}

/// Generates shims to run mise tasks
///
/// By default, this will build shims like ./bin/<task>. These can be paired with `mise generate bootstrap` so contributors to a project can execute mise tasks without installing mise into their system. When a parent and nested task both exist, the parent stub is written to `<parent>/_default`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tasks add test -- echo 'running tests'\u{1b}[22m\n    $ \u{1b}[1mmise generate task-stubs\u{1b}[22m\n    $ \u{1b}[1m./bin/test\u{1b}[22m\n    running tests\n"
)]
pub struct GenerateTaskStubsArgs {
    /// Directory to create task stubs inside of
    #[arg(long = "dir", short = 'd', value_name = "DIR", default = "bin")]
    pub dir: ::std::option::Option<::std::string::String>,
    /// Path to a mise bin to use when running the task stub.
    ///
    /// Use `--mise-bin=./bin/mise` to use a mise bin generated from `mise generate bootstrap`
    #[arg(
        long = "mise-bin",
        short = 'm',
        value_name = "MISE_BIN",
        default = "mise"
    )]
    pub mise_bin: ::std::option::Option<::std::string::String>,
}

/// Generate a tool stub for HTTP-based tools
///
/// This command generates tool stubs that can automatically download and execute tools from HTTP URLs. It can detect checksums, file sizes, and binary paths automatically by downloading and analyzing the tool.
///
/// When generating stubs with platform-specific URLs, the command will append new platforms to existing stub files rather than overwriting them. This allows you to incrementally build cross-platform tool stubs.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    Generate a tool stub for a single URL:\n    $ \u{1b}[1mmise generate tool-stub ./bin/gh --url \"https://github.com/cli/cli/releases/download/v2.96.0/gh_2.96.0_linux_amd64.tar.gz\"\u{1b}[22m\n\n    Generate a tool stub with platform-specific URLs:\n    $ \u{1b}[1mmise generate tool-stub ./bin/rg \\\n        --platform-url linux-x64:https://github.com/BurntSushi/ripgrep/releases/download/14.0.3/ripgrep-14.0.3-x86_64-unknown-linux-musl.tar.gz \\\n        --platform-url darwin-arm64:https://github.com/BurntSushi/ripgrep/releases/download/14.0.3/ripgrep-14.0.3-aarch64-apple-darwin.tar.gz\u{1b}[22m\n\n    Append additional platforms to an existing stub:\n    $ \u{1b}[1mmise generate tool-stub ./bin/rg \\\n        --platform-url linux-x64:https://example.com/rg-linux.tar.gz\u{1b}[22m\n    $ \u{1b}[1mmise generate tool-stub ./bin/rg \\\n        --platform-url darwin-arm64:https://example.com/rg-darwin.tar.gz\u{1b}[22m\n    # The stub now contains both platforms\n\n    Use auto-detection for platform from URL:\n    $ \u{1b}[1mmise generate tool-stub ./bin/node \\\n        --platform-url https://nodejs.org/dist/v22.17.1/node-v22.17.1-darwin-arm64.tar.gz\u{1b}[22m\n    # Platform 'macos-arm64' will be auto-detected from the URL\n\n    Generate with platform-specific binary paths:\n    $ \u{1b}[1mmise generate tool-stub ./bin/tool \\\n        --platform-url linux-x64:https://example.com/tool-linux.tar.gz \\\n        --platform-url windows-x64:https://example.com/tool-windows.zip \\\n        --platform-bin windows-x64:tool.exe\u{1b}[22m\n\n    Generate without downloading (faster):\n    $ \u{1b}[1mmise generate tool-stub ./bin/tool --url \"https://example.com/tool.tar.gz\" --skip-download\u{1b}[22m\n\n    Fetch checksums for an existing stub:\n    $ \u{1b}[1mmise generate tool-stub ./bin/jq --fetch\u{1b}[22m\n    # This will read the existing stub and download files to fill in any missing checksums/sizes\n\n    Generate a bootstrap stub that installs mise if needed:\n    $ \u{1b}[1mmise generate tool-stub ./bin/tool --url \"https://example.com/tool.tar.gz\" --bootstrap\u{1b}[22m\n    # The stub will check for mise and install it automatically before running the tool\n\n    Generate a bootstrap stub with a pinned mise version:\n    $ \u{1b}[1mmise generate tool-stub ./bin/tool --url \"https://example.com/tool.tar.gz\" --bootstrap --bootstrap-version 2025.1.0\u{1b}[22m\n\n    Lock an existing tool stub with pinned version and platform URLs/checksums:\n    $ \u{1b}[1mmise generate tool-stub ./bin/node --lock\u{1b}[22m\n\n    Bump the version in a locked stub:\n    $ \u{1b}[1mmise generate tool-stub ./bin/node --lock --version 22\u{1b}[22m\n    # Resolves the latest node 22.x, pins it, and updates platform URLs/checksums\n"
)]
pub struct GenerateToolStubArgs {
    /// Binary path within the extracted archive
    ///
    /// If not specified and the archive is downloaded, will auto-detect the most likely binary
    #[arg(long = "bin", short = 'b', value_name = "BIN")]
    pub bin: ::std::option::Option<::std::string::String>,
    /// Wrap stub in a bootstrap script that installs mise if not already present
    ///
    /// When enabled, generates a bash script that: 1. Checks if mise is installed at the expected path 2. If not, downloads and installs mise using the embedded installer 3. Executes the tool stub using mise
    #[arg(long = "bootstrap")]
    pub bootstrap: bool,
    /// Specify mise version for the bootstrap script
    ///
    /// By default, uses the latest version from the install script. Use this to pin to a specific version (e.g., "2025.1.0").
    #[arg(
        long = "bootstrap-version",
        requires("--bootstrap"),
        value_name = "BOOTSTRAP_VERSION"
    )]
    pub bootstrap_version: ::std::option::Option<::std::string::String>,
    /// Checksum algorithm to use when downloading artifacts
    ///
    /// Accepts `blake3` or `sha256` and defaults to `blake3`. Cannot be used with `--lock` or `--skip-download` because those modes do not calculate checksums.
    #[arg(
        long = "checksum-algorithm",
        conflicts("--lock", "--skip-download"),
        value_name = "CHECKSUM_ALGORITHM",
        default = "blake3"
    )]
    pub checksum_algorithm: ::std::option::Option<GenerateToolStubChecksumAlgorithmValue>,
    /// Fetch checksums and sizes for an existing tool stub file
    ///
    /// This reads an existing stub file and fills in any missing checksum/size fields by downloading the files. URLs must already be present in the stub.
    #[arg(
        long = "fetch",
        conflicts(
            "--url",
            "--platform-url",
            "--version",
            "--bin",
            "--platform-bin",
            "--skip-download",
            "--lock"
        )
    )]
    pub fetch: bool,
    /// HTTP backend type to use
    #[arg(long = "http", value_name = "HTTP", default = "http")]
    pub http: ::std::option::Option<::std::string::String>,
    #[arg(
        help = "Resolve and embed lockfile data (exact version + platform URLs/checksums) into an existing stub file for reproducible installs without runtime API calls",
        long_help = "Resolve and embed lockfile data (exact version + platform URLs/checksums)\ninto an existing stub file for reproducible installs without runtime API calls",
        long = "lock",
        conflicts(
            "--url",
            "--platform-url",
            "--bin",
            "--platform-bin",
            "--fetch",
            "--skip-download"
        )
    )]
    pub lock: bool,
    /// Platform-specific binary paths in the format platform:path
    ///
    /// Examples: --platform-bin windows-x64:tool.exe --platform-bin linux-x64:bin/tool
    #[arg(long = "platform-bin", value_name = "PLATFORM_BIN")]
    pub platform_bin: ::std::vec::Vec<::std::string::String>,
    /// Platform-specific URLs in the format platform:url or just url (auto-detect platform)
    ///
    /// When the output file already exists, new platforms will be appended to the existing platforms table. Existing platform URLs will be updated if specified again.
    ///
    /// If only a URL is provided (without platform:), the platform will be automatically detected from the URL filename.
    ///
    /// Examples: --platform-url linux-x64:https://... --platform-url https://nodejs.org/dist/v22.17.1/node-v22.17.1-darwin-arm64.tar.gz
    #[arg(long = "platform-url", value_name = "PLATFORM_URL")]
    pub platform_url: ::std::vec::Vec<::std::string::String>,
    /// Skip downloading for checksum and binary path detection (faster but less informative)
    #[arg(long = "skip-download")]
    pub skip_download: bool,
    /// URL for downloading the tool
    ///
    /// Example: https://github.com/owner/repo/releases/download/v2.0.0/tool-linux-x64.tar.gz
    #[arg(long = "url", short = 'u', value_name = "URL")]
    pub url: ::std::option::Option<::std::string::String>,
    /// Version of the tool
    #[arg(long = "version", value_name = "VERSION", default = "latest")]
    pub version: ::std::option::Option<::std::string::String>,
    /// Output file path for the tool stub
    #[arg(positional, value_name = "OUTPUT")]
    pub output: ::std::string::String,
}

#[derive(ValueEnum)]
pub enum GenerateToolStubChecksumAlgorithmValue {
    #[arg(name = "blake3")]
    Blake3,
    #[arg(name = "sha256")]
    Sha256,
}

/// Generate files for various tools/services
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct GenerateArgs {
    #[arg(subcommand)]
    pub command: GenerateCommands,
}

#[derive(Subcommand)]
pub enum GenerateCommands {
    /// Generate a script to download+execute mise
    #[arg(name = "bootstrap")]
    Bootstrap(Box<GenerateBootstrapArgs>),
    /// Generate a mise.toml file
    #[arg(name = "config")]
    Config(Box<GenerateConfigArgs>),
    /// Generate a devcontainer to execute mise
    #[arg(name = "devcontainer")]
    Devcontainer(Box<GenerateDevcontainerArgs>),
    /// Generate a git pre-commit hook
    #[arg(name = "git-pre-commit", alias = "pre-commit")]
    GitPreCommit(Box<GenerateGitPreCommitArgs>),
    /// Generate a GitHub Action workflow file
    #[arg(name = "github-action")]
    GithubAction(Box<GenerateGithubActionArgs>),
    /// Generate documentation for tasks in a project
    #[arg(name = "task-docs")]
    TaskDocs(Box<GenerateTaskDocsArgs>),
    /// Generates shims to run mise tasks
    #[arg(name = "task-stubs")]
    TaskStubs(Box<GenerateTaskStubsArgs>),
    /// Generate a tool stub for HTTP-based tools
    #[arg(name = "tool-stub")]
    ToolStub(Box<GenerateToolStubArgs>),
}

/// Display the GitHub token mise will use for a given host
///
/// Shows which token source mise would use, useful for debugging authentication issues. The token is masked by default.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise github token\u{1b}[22m\n    github.com: ghp_…xxxx (source: GITHUB_TOKEN)\n\n    $ \u{1b}[1mmise github token --unmask\u{1b}[22m\n    github.com: ghp_xxxxxxxxxxxx (source: GITHUB_TOKEN)\n\n    $ \u{1b}[1mmise github token github.mycompany.com\u{1b}[22m\n    github.mycompany.com: (none)\n"
)]
pub struct GithubTokenArgs {
    /// Force native GitHub OAuth device flow instead of normal token resolution
    #[arg(long = "oauth")]
    pub oauth: bool,
    /// Print only the token value
    #[arg(long = "raw")]
    pub raw: bool,
    #[arg(
        help = "Mint a fresh OAuth token even if the cached one has not expired, via the refresh-token grant or a new device-code flow",
        long_help = "Mint a fresh OAuth token even if the cached one has not\nexpired, via the refresh-token grant or a new device-code flow",
        long = "refresh",
        requires("--oauth")
    )]
    pub refresh: bool,
    /// Show the full unmasked token
    #[arg(long = "unmask")]
    pub unmask: bool,
    /// GitHub hostname
    #[arg(positional, value_name = "HOST", default = "github.com")]
    pub host: ::std::option::Option<::std::string::String>,
}

/// GitHub related commands
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct GithubArgs {
    #[arg(subcommand)]
    pub command: GithubCommands,
}

#[derive(Subcommand)]
pub enum GithubCommands {
    /// Display the GitHub token mise will use for a given host
    #[arg(name = "token", hide)]
    Token(Box<GithubTokenArgs>),
}

/// Sets/gets the global tool version(s)
///
/// Displays the contents of global config after writing. The file is `$HOME/.config/mise/config.toml` by default. It can be changed with `$MISE_GLOBAL_CONFIG_FILE`. If `$MISE_GLOBAL_CONFIG_FILE` is set to anything that ends in `.toml`, it will be parsed as `mise.toml`. Otherwise, it will be parsed as a `.tool-versions` file.
///
/// Use MISE_ASDF_COMPAT=1 to default the global config to ~/.tool-versions
///
/// Use `mise local` to set a tool version locally in the current directory.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n    # set the current version of node to 20.x\n    # will use a fuzzy version (e.g.: 20) in .tool-versions file\n    $ \u{1b}[1mmise global --fuzzy node@20\u{1b}[22m\n\n    # set the current version of node to 20.x\n    # will use a precise version (e.g.: 20.0.0) in .tool-versions file\n    $ \u{1b}[1mmise global --pin node@20\u{1b}[22m\n\n    # show the current version of node in ~/.tool-versions\n    $ \u{1b}[1mmise global node\u{1b}[22m\n    20.0.0\n"
)]
pub struct GlobalArgs {
    #[arg(
        help = "Save fuzzy version to `~/.tool-versions`\ne.g.: `mise global --fuzzy node@20` will save `node 20` to ~/.tool-versions\nthis is the default behavior unless MISE_ASDF_COMPAT=1",
        long = "fuzzy",
        overrides("--pin")
    )]
    pub fuzzy: bool,
    /// Get the path of the global config file
    #[arg(long = "path")]
    pub path: bool,
    #[arg(
        help = "Save exact version to `~/.tool-versions`\ne.g.: `mise global --pin node@20` will save `node 20.0.0` to ~/.tool-versions",
        long = "pin",
        overrides("--fuzzy")
    )]
    pub pin: bool,
    /// Remove the tool(s) from ~/.tool-versions
    #[arg(long = "remove", alias("rm", "unset"), value_name = "TOOL")]
    pub remove: ::std::vec::Vec<::std::string::String>,
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to add to .tool-versions\ne.g.: node@20\nIf this is a single tool with no version, the current value of the global\n.tool-versions will be displayed"
    )]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// [internal] called by activate hook to update env vars directory change
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct HookEnvArgs {
    /// Skip early exit check
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Hide warnings such as when a tool is not installed
    #[arg(long = "quiet", short = 'q')]
    pub quiet: bool,
    /// Shell type to generate script for
    #[arg(long = "shell", short = 's', value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Reason for calling hook-env (e.g., "precmd", "chpwd")
    #[arg(long = "reason", hide, value_name = "REASON")]
    pub reason: ::std::option::Option<HookEnvReasonValue>,
    /// Show "mise: <TOOL>@<VERSION>" message when changing directories
    #[arg(long = "status", hide)]
    pub status: bool,
}

#[derive(ValueEnum)]
pub enum HookEnvReasonValue {
    #[arg(name = "precmd")]
    Precmd,
    #[arg(name = "chpwd")]
    Chpwd,
}

/// [internal] called by shell when a command is not found
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct HookNotFoundArgs {
    /// Shell type to generate script for
    #[arg(long = "shell", short = 's', value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Attempted bin to run
    #[arg(positional, value_name = "BIN")]
    pub bin: ::std::string::String,
}

/// Removes mise CLI and all related data
///
/// Skips config directory by default.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct ImplodeArgs {
    /// List directories that would be removed without actually removing them
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Also remove config directory
    #[arg(long = "config")]
    pub config: bool,
}

/// Edit mise.toml interactively
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise edit\u{1b}[22m             \u{1b}[2m# edit mise.toml interactively\u{1b}[22m\n    $ \u{1b}[1mmise edit .mise.toml\u{1b}[22m  \u{1b}[2m# edit a specific file\u{1b}[22m\n    $ \u{1b}[1mmise edit -g\u{1b}[22m          \u{1b}[2m# edit the global config file\u{1b}[22m\n    $ \u{1b}[1mmise edit -y\u{1b}[22m          \u{1b}[2m# skip interactive editor\u{1b}[22m\n    $ \u{1b}[1mmise edit -n\u{1b}[22m          \u{1b}[2m# preview without writing\u{1b}[22m\n"
)]
pub struct EditArgs {
    /// Edit the global config file (~/.config/mise/config.toml)
    #[arg(long = "global", short = 'g', conflicts("PATH"))]
    pub global: bool,
    /// Show what would be generated without writing to file
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Path to a .tool-versions file to import tools from
    #[arg(long = "tool-versions", short = 't', value_name = "TOOL_VERSIONS")]
    pub tool_versions: ::std::option::Option<::std::string::String>,
    /// Path to the config file to create
    #[arg(positional, value_name = "PATH")]
    pub path: ::std::option::Option<::std::string::String>,
}

/// Install a tool version
///
/// Installs a tool version to `~/.local/share/mise/installs/<TOOL>/<VERSION>` Installing alone will not activate the tools so they won't be in PATH. To install and/or activate in one command, use `mise use` which will create a `mise.toml` file in the current directory to activate this tool when inside the directory. Alternatively, run `mise exec <TOOL>@<VERSION> -- <COMMAND>` to execute a tool without creating config files.
///
/// Tools will be installed in parallel. To disable, set `--jobs=1` or `MISE_JOBS=1`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise install node@20.0.0\u{1b}[22m  # install specific node version\n    $ \u{1b}[1mmise install node@20\u{1b}[22m      # install fuzzy node version\n    $ \u{1b}[1mmise install node\u{1b}[22m         # install version specified in mise.toml\n    $ \u{1b}[1mmise install\u{1b}[22m              # installs everything specified in mise.toml\n    $ \u{1b}[1mmise install --include-task-tools\u{1b}[22m # also install tools required by tasks\n"
)]
pub struct InstallArgs {
    #[arg(
        help = "Force reinstall even if already installed\nWith no tools specified, reinstall all configured tools",
        long = "force",
        short = 'f'
    )]
    pub force: bool,
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1\n[default: 4]",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Show what would be installed without actually installing
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Show installation output
    ///
    /// This argument will print backend output such as download, configuration, and compilation output.
    #[arg(long = "verbose", short = 'v', count)]
    pub verbose: u8,
    /// Like --dry-run but exits with code 1 if there are tools to install
    ///
    /// This is useful for scripts to check if tools need to be installed.
    #[arg(long = "dry-run-code")]
    pub dry_run_code: bool,
    /// Also install tools required by tasks in the current scope
    ///
    /// This prepares task tools without running task commands or dependencies. Combine with --monorepo to include tasks from every configured root.
    #[arg(long = "include-task-tools")]
    pub include_task_tools: bool,
    /// Only install versions released before this date or older than this duration
    ///
    /// Supports absolute dates like "2024-06-01" and relative durations like "90d" or "1y".
    #[arg(
        long = "minimum-release-age",
        alias = "before",
        value_name = "MINIMUM_RELEASE_AGE"
    )]
    pub minimum_release_age: ::std::option::Option<::std::string::String>,
    /// Install tools from every [monorepo].config_roots config root
    ///
    /// Uses the active MISE_ENV and requires monorepo_root = true plus explicit [monorepo].config_roots in the monorepo root config.
    #[arg(long = "monorepo", env = "MISE_MONOREPO")]
    pub monorepo: bool,
    #[arg(
        help = "Connect backend install command stdin/stdout/stderr directly to the terminal Implies --jobs=1",
        long_help = "Connect backend install command stdin/stdout/stderr directly to the terminal\nImplies --jobs=1",
        long = "raw",
        overrides("--jobs")
    )]
    pub raw: bool,
    /// Install tool(s) to a shared directory
    ///
    /// Installs to the specified directory instead of the default install location. May require elevated permissions depending on the path.
    #[arg(long = "shared", conflicts("--system"), value_name = "SHARED")]
    pub shared: ::std::option::Option<::std::string::String>,
    /// Install tool(s) to the system-wide shared directory
    ///
    /// Installs to /usr/local/share/mise/installs (or MISE_SYSTEM_DATA_DIR/installs). May require elevated permissions (e.g. sudo).
    #[arg(long = "system", conflicts("--shared"))]
    pub system: bool,
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to install e.g.: node@20",
        long_help = "Tool(s) to install\ne.g.: node@20"
    )]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Install a tool version to a specific path
///
/// Used for building a tool to a directory for use outside of mise
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # install node@20.0.0 into ./mynode\n    $ \u{1b}[1mmise install-into node@20.0.0 ./mynode && ./mynode/bin/node -v\u{1b}[22m\n    20.0.0\n"
)]
pub struct InstallIntoArgs {
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool to install e.g.: node@20",
        long_help = "Tool to install\ne.g.: node@20"
    )]
    pub tool_version: ::std::string::String,
    /// Path to install the tool into
    #[arg(positional, value_name = "PATH")]
    pub path: ::std::string::String,
}

/// Gets the latest available version for a plugin
///
/// Supports prefixes such as `node@20` to get the latest version of node 20.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise latest node@20\u{1b}[22m  # get the latest version of node 20\n    20.0.0\n\n    $ \u{1b}[1mmise latest node\u{1b}[22m     # get the latest stable version of node\n    20.0.0\n\n    $ \u{1b}[1mmise latest node --minimum-release-age 2024-01-01\u{1b}[22m  # latest stable node released before 2024-01-01\n"
)]
pub struct LatestArgs {
    /// Show latest installed instead of available version
    #[arg(long = "installed", short = 'i')]
    pub installed: bool,
    /// Only consider versions released before this date or older than this duration
    ///
    /// Supports absolute dates like "2024-06-01" and relative durations like "90d" or "1y". Overrides per-tool `minimum_release_age` options and the global `minimum_release_age` setting.
    #[arg(
        long = "minimum-release-age",
        alias = "before",
        conflicts("--installed"),
        value_name = "MINIMUM_RELEASE_AGE"
    )]
    pub minimum_release_age: ::std::option::Option<::std::string::String>,
    /// Tool to get the latest version of
    #[arg(positional, value_name = "TOOL@VERSION")]
    pub tool_version: ::std::string::String,
    #[arg(
        positional,
        value_name = "ASDF_VERSION",
        help = "The version prefix to use when querying the latest version same as the first argument after the \"@\" used for asdf compatibility",
        long_help = "The version prefix to use when querying the latest version\nsame as the first argument after the \"@\"\nused for asdf compatibility",
        hide
    )]
    pub asdf_version: ::std::option::Option<::std::string::String>,
}

/// Symlinks a tool version into mise
///
/// Use this for adding installs either custom compiled outside mise or built with a different tool.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # build node-20.0.0 with node-build and link it into mise\n    $ \u{1b}[1mnode-build 20.0.0 ~/.nodes/20.0.0\u{1b}[22m\n    $ \u{1b}[1mmise link node@20.0.0 ~/.nodes/20.0.0\u{1b}[22m\n\n    # have mise use the node version provided by Homebrew\n    $ \u{1b}[1mbrew install node\u{1b}[22m\n    $ \u{1b}[1mmise link node@brew $(brew --prefix node)\u{1b}[22m\n    $ \u{1b}[1mmise use node@brew\u{1b}[22m\n"
)]
pub struct LinkArgs {
    /// Overwrite an existing tool version if it exists
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Tool name and version to create a symlink for
    #[arg(positional, value_name = "TOOL@VERSION")]
    pub tool_version: ::std::string::String,
    #[arg(
        positional,
        value_name = "PATH",
        help = "The local path to the tool version\ne.g.: ~/.nvm/versions/node/v20.0.0"
    )]
    pub path: ::std::string::String,
}

/// Sets/gets tool version in local .tool-versions or mise.toml
///
/// Use this to set a tool's version when within a directory Use `mise global` to set a tool version globally This uses `.tool-version` by default unless there is a `mise.toml` file or if `MISE_USE_TOML` is set. A future v2 release of mise will default to using `mise.toml`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n    # set the current version of node to 20.x for the current directory\n    # will use a precise version (e.g.: 20.0.0) in .tool-versions file\n    $ \u{1b}[1mmise local node@20\u{1b}[22m\n\n    # set node to 20.x for the current project (recurses up to find .tool-versions)\n    $ \u{1b}[1mmise local -p node@20\u{1b}[22m\n\n    # set the current version of node to 20.x for the current directory\n    # will use a fuzzy version (e.g.: 20) in .tool-versions file\n    $ \u{1b}[1mmise local --fuzzy node@20\u{1b}[22m\n\n    # removes node from .tool-versions\n    $ \u{1b}[1mmise local --remove=node\u{1b}[22m\n\n    # show the current version of node in .tool-versions\n    $ \u{1b}[1mmise local node\u{1b}[22m\n    20.0.0\n"
)]
pub struct LocalArgs {
    #[arg(
        help = "Recurse up to find a .tool-versions file rather than using the current directory only\nby default this command will only set the tool in the current directory (\"$PWD/.tool-versions\")",
        long = "parent",
        short = 'p'
    )]
    pub parent: bool,
    #[arg(
        help = "Save fuzzy version to `.tool-versions` e.g.: `mise local --fuzzy node@20` will save `node 20` to .tool-versions This is the default behavior unless MISE_ASDF_COMPAT=1",
        long_help = "Save fuzzy version to `.tool-versions`\ne.g.: `mise local --fuzzy node@20` will save `node 20` to .tool-versions\nThis is the default behavior unless MISE_ASDF_COMPAT=1",
        long = "fuzzy",
        overrides("--pin")
    )]
    pub fuzzy: bool,
    /// Get the path of the config file
    #[arg(long = "path")]
    pub path: bool,
    #[arg(
        help = "Save exact version to `.tool-versions`\ne.g.: `mise local --pin node@20` will save `node 20.0.0` to .tool-versions",
        long = "pin",
        overrides("--fuzzy")
    )]
    pub pin: bool,
    /// Remove the tool(s) from .tool-versions
    #[arg(long = "remove", alias("rm", "unset"), value_name = "TOOL")]
    pub remove: ::std::vec::Vec<::std::string::String>,
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to add to .tool-versions/mise.toml\ne.g.: node@20\nif this is a single tool with no version,\nthe current value of .tool-versions/mise.toml will be displayed"
    )]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Update lockfile checksums and URLs for all specified platforms
///
/// Updates checksums and download URLs for all platforms already specified in the lockfile. If no lockfile exists, shows what would be created based on the current configuration, including tools declared by tasks. This allows you to refresh lockfile data for platforms other than the one you're currently on. Operates on the lockfile in the current config root. Use TOOL arguments to target specific tools.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise lock\u{1b}[22m                       # update lockfile for all common platforms\n    $ \u{1b}[1mmise lock node python\u{1b}[22m           # update only node and python\n    $ \u{1b}[1mmise lock --platform linux-x64\u{1b}[22m  # update only linux-x64 platform\n    $ \u{1b}[1mmise lock --dry-run\u{1b}[22m             # show what would be updated\n    $ \u{1b}[1mmise lock --bump\u{1b}[22m                # re-resolve selectors like \"latest\" or \"20\" to the latest matching versions\n    $ \u{1b}[1mmise lock --bump --dry-run --json\u{1b}[22m   # list available updates as JSON without writing\n    $ \u{1b}[1mmise lock --minimum-release-age 2024-01-01\u{1b}[22m   # lock latest/fuzzy versions released before 2024-01-01\n    $ \u{1b}[1mmise lock --local\u{1b}[22m               # update mise.local.lock for local configs\n    $ \u{1b}[1mmise lock --global\u{1b}[22m              # update only global config lockfiles\n"
)]
pub struct LockArgs {
    #[arg(
        help = "Target only global config lockfiles (~/.config/mise/mise.lock and system config)\nBy default, only the active project config root is locked",
        long = "global",
        short = 'g'
    )]
    pub global: bool,
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Show what would be updated without making changes
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    #[arg(
        help = "Comma-separated list of platforms to target\ne.g.: linux-x64,macos-arm64,windows-x64\nIf not specified, all platforms already in lockfile will be updated",
        long = "platform",
        short = 'p',
        delimiter = ',',
        value_name = "PLATFORM"
    )]
    pub platform: ::std::vec::Vec<::std::string::String>,
    /// Re-resolve fuzzy version selectors against the latest available versions
    ///
    /// By default, `mise lock` refreshes metadata for the currently locked versions. With this flag, selectors like "latest", "lts", or prefixes like "20" are re-resolved against the latest matching remote versions, so the lockfile advances without installing anything. Config files are never modified: exactly pinned versions resolve to themselves and stay unchanged (use `mise upgrade --bump` to rewrite pins in mise.toml).
    #[arg(long = "bump")]
    pub bump: bool,
    /// Output version changes as JSON
    ///
    /// Prints an array of objects describing lockfile version changes: name, backend, lockfile, old_versions, new_versions. Version lists keep config/lockfile order; they are not sorted. Only version-level changes are reported: checksum/URL refreshes for unchanged versions produce no entries, so plain `mise lock --json` typically prints `[]` while still updating the lockfile. Suppresses the human-readable output. Combine with `--dry-run` to detect available updates without writing the lockfile.
    #[arg(long = "json")]
    pub json: bool,
    #[arg(
        help = "Update mise.local.lock instead of mise.lock\nUse for tools defined in .local.toml configs",
        long = "local"
    )]
    pub local: bool,
    /// Only lock versions released before this age or date
    ///
    /// Supports absolute dates like "2024-06-01" and relative durations like "90d" or "1y". This only affects fuzzy version matches like "20" or "latest". Explicitly pinned versions like "22.5.0" are not filtered. Existing matching lockfile entries are preserved and are not downgraded solely by this flag.
    #[arg(
        long = "minimum-release-age",
        alias = "before",
        value_name = "MINIMUM_RELEASE_AGE"
    )]
    pub minimum_release_age: ::std::option::Option<::std::string::String>,
    #[arg(
        positional,
        value_name = "TOOL",
        help = "Tool(s) to update in lockfile\ne.g.: node python\nIf not specified, all configured and task-specific tools will be updated"
    )]
    pub tool: ::std::vec::Vec<::std::string::String>,
}

/// List installed and active tool versions
///
/// This command lists tools that mise "knows about". These may be tools that are currently installed, or those that are in a config file (active) but may or may not be installed.
///
/// It's a useful command to get the current state of your tools.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise ls\u{1b}[22m\n    node    20.0.0 ~/src/myapp/.tool-versions latest\n    python  3.11.0 ~/.tool-versions           3.10\n    python  3.10.0\n\n    $ \u{1b}[1mmise ls --current\u{1b}[22m\n    node    20.0.0 ~/src/myapp/.tool-versions 20\n    python  3.11.0 ~/.tool-versions           3.11.0\n\n    $ \u{1b}[1mmise ls --json\u{1b}[22m\n    {\n      \"node\": [\n        {\n          \"version\": \"20.0.0\",\n          \"install_path\": \"/Users/jdx/.mise/installs/node/20.0.0\",\n          \"source\": {\n            \"type\": \"mise.toml\",\n            \"path\": \"/Users/jdx/mise.toml\"\n          }\n        }\n      ],\n      \"python\": [...]\n    }\n\n    $ \u{1b}[1mmise ls --all-sources\u{1b}[22m\n    node    20.0.0  ~/src/myapp/mise.toml  20\n                    ~/.config/mise/config.toml  latest\n"
)]
pub struct LsArgs {
    /// Only show tool versions currently specified in a mise.toml
    #[arg(long = "current", short = 'c')]
    pub current: bool,
    /// Only show tool versions currently specified in the global mise.toml
    #[arg(long = "global", short = 'g', conflicts("--local"))]
    pub global: bool,
    #[arg(
        help = "Only show tool versions that are installed (Hides tools defined in mise.toml but not installed)",
        long_help = "Only show tool versions that are installed\n(Hides tools defined in mise.toml but not installed)",
        long = "installed",
        short = 'i'
    )]
    pub installed: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Only show tool versions currently specified in the local mise.toml
    #[arg(long = "local", short = 'l', conflicts("--global"))]
    pub local: bool,
    /// Display missing tool versions
    #[arg(long = "missing", short = 'm', conflicts("--installed"))]
    pub missing: bool,
    /// Don't fetch information such as outdated versions
    #[arg(long = "offline", short = 'o', hide)]
    pub offline: bool,
    #[arg(long = "plugin", short = 'p', hide, value_name = "PLUGIN")]
    pub plugin: ::std::option::Option<::std::string::String>,
    /// Display all tracked config sources for tools
    #[arg(
        long = "all-sources",
        conflicts("--current", "--global", "--local", "--prunable")
    )]
    pub all_sources: bool,
    /// List tools from every [monorepo].config_roots config root
    ///
    /// Uses the active MISE_ENV and requires monorepo_root = true plus explicit [monorepo].config_roots in the monorepo root config.
    #[arg(
        long = "monorepo",
        env = "MISE_MONOREPO",
        conflicts("--all-sources", "--prunable")
    )]
    pub monorepo: bool,
    /// Don't display headers
    #[arg(long = "no-header", alias = "no-headers", conflicts("--json"))]
    pub no_header: bool,
    /// Display whether a version is outdated
    #[arg(long = "outdated")]
    pub outdated: bool,
    /// Display versions matching this prefix
    #[arg(long = "prefix", requires("INSTALLED_TOOL"), value_name = "PREFIX")]
    pub prefix: ::std::option::Option<::std::string::String>,
    /// List only tools that can be pruned with `mise prune`
    #[arg(long = "prunable")]
    pub prunable: bool,
    /// Only show tool versions from [TOOL]
    #[arg(positional, value_name = "INSTALLED_TOOL", conflicts("--plugin"))]
    pub installed_tool: ::std::vec::Vec<::std::string::String>,
}

/// List runtime versions available for install.
///
/// Note that the results may be cached, run `mise cache clean` to clear the cache and get fresh results.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise ls-remote node\u{1b}[22m\n    18.0.0\n    20.0.0\n\n    $ \u{1b}[1mmise ls-remote node@20\u{1b}[22m\n    20.0.0\n    20.1.0\n\n    $ \u{1b}[1mmise ls-remote node 20\u{1b}[22m\n    20.0.0\n    20.1.0\n\n    $ \u{1b}[1mmise ls-remote node --minimum-release-age 2024-01-01\u{1b}[22m\n    20.0.0\n\n    $ \u{1b}[1mmise ls-remote github:cli/cli --json\u{1b}[22m\n    [{\"version\":\"2.62.0\",\"created_at\":\"2024-11-14T15:40:35Z\",\"prerelease\":false},{\"version\":\"2.61.0\",\"created_at\":\"2024-10-23T19:22:15Z\",\"prerelease\":false}]\n"
)]
pub struct LsRemoteArgs {
    /// Show all installed plugins and versions
    #[arg(long = "all", conflicts("TOOL@VERSION", "PREFIX"))]
    pub all: bool,
    /// Only show versions released before this age or date
    ///
    /// Supports absolute dates like "2024-06-01" and relative durations like "90d" or "1y".
    #[arg(
        long = "minimum-release-age",
        alias = "before",
        value_name = "MINIMUM_RELEASE_AGE"
    )]
    pub minimum_release_age: ::std::option::Option<::std::string::String>,
    /// Output in JSON format (includes version metadata like created_at timestamps when available)
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Disable checking the mise-versions host
    #[arg(long = "no-versions-host")]
    pub no_versions_host: bool,
    #[arg(
        help = "Include pre-release versions in the output for backends that report\nupstream prerelease metadata or opt in to regex-based prerelease\ndetection. Equivalent to setting `MISE_PRERELEASES=1` or the\n`prereleases` setting for the duration of this command.",
        long = "prerelease"
    )]
    pub prerelease: bool,
    /// Fail if release metadata fetches fail
    ///
    /// Requires --json and --no-versions-host.
    ///
    /// This prevents metadata consumers from accepting empty fallback results when a backend's metadata-producing upstream request fails.
    #[arg(long = "strict-metadata", requires("--json", "--no-versions-host"))]
    pub strict_metadata: bool,
    /// Tool to get versions for
    #[arg(positional, value_name = "TOOL@VERSION", required_unless("--all"))]
    pub tool_version: ::std::option::Option<::std::string::String>,
    #[arg(
        positional,
        value_name = "PREFIX",
        help = "The version prefix to use when querying the latest version\nsame as the first argument after the \"@\""
    )]
    pub prefix: ::std::option::Option<::std::string::String>,
}

/// Run Model Context Protocol (MCP) server
///
/// This command starts an MCP server that exposes mise functionality to AI assistants over stdin/stdout using JSON-RPC protocol.
///
/// The MCP server provides access to: - Installed and available tools - Task definitions and execution - Environment variables - Configuration information - Task execution via the run_task tool
///
/// Resources available: - mise://tools - List all tools (use ?include_inactive=true to include inactive tools) - mise://tasks - List all tasks with their configurations - mise://env - List all environment variables - mise://config - Show configuration files and project root
///
/// Tools available: - list_commands - Every mise command, with its declared effect on the world - install_tool - Install a tool with an optional version (not yet implemented) - run_task - Execute a mise task with optional arguments
///
/// Note: This is primarily intended for integration with AI assistants like Claude, Cursor, or other tools that support the Model Context Protocol.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Start the MCP server (typically used by AI assistant tools)\n    $ \u{1b}[1mmise mcp\u{1b}[22m\n\n    # Example integration with Claude Desktop (add to claude_desktop_config.json):\n    {\n      \"mcpServers\": {\n        \"mise\": {\n          \"command\": \"mise\",\n          \"args\": [\"mcp\"],\n          \"env\": {}\n        }\n      }\n    }\n\n    # Interactive testing with JSON-RPC commands:\n    $ \u{1b}[1mecho '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1.0\"}}}' | mise mcp\u{1b}[22m\n\n    # Resources you can query:\n    - \u{1b}[1mmise://tools\u{1b}[22m - List active tools\n    - \u{1b}[1mmise://tools?include_inactive=true\u{1b}[22m - List all installed tools\n    - \u{1b}[1mmise://tasks\u{1b}[22m - List all tasks\n    - \u{1b}[1mmise://env\u{1b}[22m - List environment variables\n    - \u{1b}[1mmise://config\u{1b}[22m - Show configuration info\n\n    # Tools available:\n    - \u{1b}[1mlist_commands\u{1b}[22m - Every mise command and what running it does\n      Example: {\"include_hidden\": false}\n    - \u{1b}[1minstall_tool\u{1b}[22m - Install a tool (not yet implemented)\n    - \u{1b}[1mrun_task\u{1b}[22m - Execute a mise task with optional arguments\n      Example: {\"task\": \"build\", \"args\": [\"--verbose\"]}\n"
)]
pub struct McpArgs {}

/// [experimental] Build an OCI image from the current mise.toml
///
/// Each tool version becomes its own content-addressable OCI layer. Bumping a tool version only invalidates that tool's layer — other tools, the base image, and config are reused unchanged. The output directory conforms to the OCI image-layout spec and can be consumed by `skopeo`, `crane`, or `podman load`.
///
/// Requires `mise settings experimental=true` (or `MISE_EXPERIMENTAL=1`).
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    Build with defaults (debian:bookworm-slim base):\n    $ \u{1b}[1mmise oci build\u{1b}[22m\n\n    Build with a specific base image and tag:\n    $ \u{1b}[1mmise oci build --from ubuntu:24.04 --tag myorg/dev:latest -o ./img\u{1b}[22m\n\n    Inspect the result with skopeo:\n    $ \u{1b}[1mskopeo inspect oci:./mise-oci\u{1b}[22m\n\n    Push to a registry:\n    $ \u{1b}[1mmise oci push --image-dir ./mise-oci ghcr.io/me/dev:latest\u{1b}[22m\n\n\u{1b}[1m\u{1b}[4mNotes:\u{1b}[22m\u{1b}[24m\n\n    - The image only contains tools from the project's mise config (and\n      any configs at-or-below the project root). Tools from\n      `~/.config/mise/config.toml` are not included; pass --include-global\n      to package them too.\n    - asdf and vfox plugins are not supported in v1; use a different backend\n      (core, aqua, ubi, github, cargo, npm, go, pipx, spm, http) for each tool.\n    - The host mise binary is embedded at /usr/local/bin/mise by default;\n      build on the same OS/arch as your target image (or pass --no-mise).\n"
)]
pub struct OciBuildArgs {
    /// Copy a host file, directory, or symlink into the image (repeatable, HOST:IMAGE)
    #[arg(long = "copy", value_name = "HOST_PATH:IMAGE_PATH")]
    pub copy: ::std::vec::Vec<::std::string::String>,
    /// Output directory for the OCI image layout
    #[arg(
        long = "output",
        short = 'o',
        value_name = "OUTPUT",
        default = "./mise-oci"
    )]
    pub output: ::std::option::Option<::std::string::String>,
    /// Base image reference (overrides [oci].from and the oci.default_from setting)
    #[arg(long = "from", value_name = "FROM")]
    pub from: ::std::option::Option<::std::string::String>,
    /// Also include tools from the global / system config (default: project-only)
    ///
    /// By default `mise oci build` only packages tools declared in the project's mise config (and any parent configs at-or-below the project root, e.g. a monorepo root config). Personal dev tools in `~/.config/mise/config.toml` are excluded so they don't bake into a project image. Pass `--include-global` to revert to the old "merge all loaded configs" behavior.
    #[arg(long = "include-global")]
    pub include_global: bool,
    /// Tag to record in the image index (the org.opencontainers.image.ref.name annotation)
    #[arg(long = "tag", short = 't', value_name = "TAG")]
    pub tag: ::std::option::Option<::std::string::String>,
    /// Where to place tool installs inside the image (default: /mise)
    #[arg(long = "mount-point", value_name = "MOUNT_POINT")]
    pub mount_point: ::std::option::Option<::std::string::String>,
    /// Do not embed the currently-running mise binary at /usr/local/bin/mise
    #[arg(long = "no-mise")]
    pub no_mise: bool,
    /// UID[:GID] to assign to every tar entry in generated layers
    ///
    /// Overrides [oci].user_id / [oci].group_id. Defaults to 0:0. If GID is omitted, it defaults to UID. This affects file ownership only; [oci].user controls the image USER directive.
    #[arg(long = "owner", value_name = "UID[:GID]")]
    pub owner: ::std::option::Option<::std::string::String>,
}

/// [experimental] Build an OCI image and push it to a registry
///
/// Pushes with mise's built-in registry client — no skopeo/crane/docker required. If `--image-dir` is not passed, builds fresh from the current mise.toml first. Only blobs the registry doesn't already have are uploaded, so repeat pushes of mostly-unchanged toolsets are cheap.
///
/// Tool layers whose tool, version, mount point, and file owner match the previously pushed image (or `--cache-from`) are reused without being rebuilt — those tools don't even need to be installed locally. Pass `--no-cache` to force a full local rebuild.
///
/// Credentials are read from the same places docker and podman use: `$REGISTRY_AUTH_FILE`, `$XDG_RUNTIME_DIR/containers/auth.json`, `~/.config/containers/auth.json`, and `~/.docker/config.json` (including credential helpers) — so `docker login` / `podman login` is all the setup needed.
///
/// Requires `mise settings experimental=true` (or `MISE_EXPERIMENTAL=1`).
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    Build and push to GHCR:\n    $ \u{1b}[1mmise oci push ghcr.io/me/devenv:latest\u{1b}[22m\n\n    Push an image built earlier:\n    $ \u{1b}[1mmise oci build -o ./img\u{1b}[22m\n    $ \u{1b}[1mmise oci push --image-dir ./img ghcr.io/me/devenv:v1\u{1b}[22m\n\n\u{1b}[1m\u{1b}[4mAuth:\u{1b}[22m\u{1b}[24m\n\n    Credentials are resolved the same way docker/podman resolve them:\n    \u{1b}[1m$REGISTRY_AUTH_FILE\u{1b}[22m, \u{1b}[1m$XDG_RUNTIME_DIR/containers/auth.json\u{1b}[22m,\n    \u{1b}[1m~/.config/containers/auth.json\u{1b}[22m, then \u{1b}[1m~/.docker/config.json\u{1b}[22m\n    (inline auths and credential helpers). Log in with either:\n    $ \u{1b}[1mdocker login ghcr.io\u{1b}[22m\n    $ \u{1b}[1mpodman login ghcr.io\u{1b}[22m\n"
)]
pub struct OciPushArgs {
    /// Reuse unchanged tool layers from this image instead of the destination ref
    ///
    /// Must live in the same repository as the destination. Useful when each push gets a unique tag (e.g. per-commit tags in CI): `--cache-from ghcr.io/me/dev:latest ghcr.io/me/dev:$SHA`.
    #[arg(
        long = "cache-from",
        conflicts("--no-cache", "--image-dir"),
        value_name = "REF"
    )]
    pub cache_from: ::std::option::Option<::std::string::String>,
    /// Base image for the build (ignored with --image-dir)
    #[arg(long = "from", value_name = "FROM")]
    pub from: ::std::option::Option<::std::string::String>,
    /// Push an already-built OCI image layout (skip the build step)
    #[arg(
        long = "image-dir",
        conflicts("--from", "--mount-point", "--no-mise", "--owner", "--include-global"),
        value_name = "IMAGE_DIR"
    )]
    pub image_dir: ::std::option::Option<::std::string::String>,
    /// Also include tools from the global / system config (default: project-only)
    ///
    /// See `mise oci build --help` for details.
    #[arg(long = "include-global")]
    pub include_global: bool,
    /// Override in-image mount point (ignored with --image-dir)
    #[arg(long = "mount-point", value_name = "MOUNT_POINT")]
    pub mount_point: ::std::option::Option<::std::string::String>,
    /// Don't reuse tool layers from the previously pushed image
    #[arg(long = "no-cache")]
    pub no_cache: bool,
    /// Don't embed the mise binary (ignored with --image-dir)
    #[arg(long = "no-mise")]
    pub no_mise: bool,
    /// UID[:GID] to assign to every tar entry when building (conflicts with --image-dir)
    ///
    /// Overrides [oci].user_id / [oci].group_id. Defaults to 0:0. If GID is omitted, it defaults to UID. This affects file ownership only; [oci].user controls the image USER directive.
    #[arg(long = "owner", value_name = "UID[:GID]")]
    pub owner: ::std::option::Option<::std::string::String>,
    /// Maintain the tag as a multi-arch image index
    ///
    /// Pushes this build's manifest by digest and points the tag at an OCI image index containing one entry per platform, preserving entries other architectures pushed. Run `mise oci push --update-index` from one runner per platform to assemble a multi-arch tag.
    #[arg(long = "update-index")]
    pub update_index: bool,
    /// Destination registry reference (e.g. `ghcr.io/me/devenv:latest`)
    #[arg(positional, value_name = "REF")]
    pub ref_: ::std::string::String,
}

/// [experimental] Build an OCI image from the current mise.toml and run a command in it
///
/// Equivalent to `mise oci build` followed by `docker run` / `podman run`. The built image is loaded into the local container engine (podman pulls the OCI layout natively; docker receives it via `docker load`) and the given command is executed inside it with stdin/stdout/stderr inherited.
///
/// Requires `mise settings experimental=true` (or `MISE_EXPERIMENTAL=1`) and one of: `podman`, `docker`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    Build the current mise.toml and drop into bash:\n    $ \u{1b}[1mmise oci run -it -- bash\u{1b}[22m\n\n    Run a one-shot command with env + volume (note: `-v` is reserved\n    for --verbose, so use `--volume`):\n    $ \u{1b}[1mmise oci run -e DEBUG=1 --volume $PWD:/work -w /work -- npm test\u{1b}[22m\n\n    Re-use a previously built layout (skip the build step):\n    $ \u{1b}[1mmise oci build -o ./img && mise oci run --image-dir ./img -- node -e 'console.log(process.version)'\u{1b}[22m\n\n\u{1b}[1m\u{1b}[4mEngines:\u{1b}[22m\u{1b}[24m\n\n    Prefers \u{1b}[1mpodman\u{1b}[22m (loads OCI layouts natively). Falls back to \u{1b}[1mdocker\u{1b}[22m\n    (loaded via \u{1b}[1mdocker load\u{1b}[22m). Pass \u{1b}[1m--engine podman\u{1b}[22m or \u{1b}[1m--engine docker\u{1b}[22m to override.\n"
)]
pub struct OciRunArgs {
    /// Container engine to use (`auto`, `podman`, or `docker`)
    #[arg(long = "engine", value_name = "ENGINE", default = "auto")]
    pub engine: ::std::option::Option<OciRunEngineValue>,
    /// Base image reference for the build (ignored with --image-dir)
    #[arg(long = "from", value_name = "FROM")]
    pub from: ::std::option::Option<::std::string::String>,
    /// Use an already-built OCI image layout instead of building fresh
    #[arg(
        long = "image-dir",
        conflicts("--from", "--mount-point", "--no-mise", "--owner", "--include-global"),
        value_name = "IMAGE_DIR"
    )]
    pub image_dir: ::std::option::Option<::std::string::String>,
    /// Also include tools from the global / system config (default: project-only)
    ///
    /// See `mise oci build --help` for details.
    #[arg(long = "include-global")]
    pub include_global: bool,
    /// Keep the loaded image in the engine's storage after the run
    ///
    /// By default, both the container (`--rm`) and the loaded image are removed when the command exits, so repeated `mise oci run` calls don't accumulate images in podman / docker storage. Pass `--keep` to retain the image under the tag mise used (`mise-oci:run-*` for docker; the pulled image ID for podman).
    #[arg(long = "keep")]
    pub keep: bool,
    /// Override in-image mount point (ignored with --image-dir)
    #[arg(long = "mount-point", value_name = "MOUNT_POINT")]
    pub mount_point: ::std::option::Option<::std::string::String>,
    /// Don't embed the mise binary (ignored with --image-dir)
    #[arg(long = "no-mise")]
    pub no_mise: bool,
    /// UID[:GID] to assign to every tar entry when building (conflicts with --image-dir)
    ///
    /// Overrides [oci].user_id / [oci].group_id. Defaults to 0:0. If GID is omitted, it defaults to UID. This affects file ownership only; [oci].user controls the image USER directive.
    #[arg(long = "owner", value_name = "UID[:GID]")]
    pub owner: ::std::option::Option<::std::string::String>,
    /// Bind-mount a host path (repeatable, `HOST:CONTAINER[:MODE]`)
    ///
    /// Note: unlike `docker run -v`, there's no `-v` short flag here because mise reserves `-v` for --verbose. Use `--volume` or `--mount`.
    #[arg(long = "volume", alias = "mount", value_name = "HOST:CONTAINER")]
    pub volume: ::std::vec::Vec<::std::string::String>,
    /// Set environment variable in the container (repeatable, `KEY=VAL`)
    #[arg(long = "env", short = 'e', value_name = "KEY=VAL")]
    pub env: ::std::vec::Vec<::std::string::String>,
    /// Run interactively (pass `-i` to the engine)
    #[arg(long = "interactive", short = 'i')]
    pub interactive: bool,
    /// Allocate a TTY (pass `-t` to the engine)
    #[arg(long = "tty", short = 't')]
    pub tty: bool,
    /// Working directory inside the container
    #[arg(long = "workdir", short = 'w', value_name = "WORKDIR")]
    pub workdir: ::std::option::Option<::std::string::String>,
    /// Command and arguments to run inside the container (after `--`)
    #[arg(positional, value_name = "CMD", double_dash = "required")]
    pub cmd: ::std::vec::Vec<::std::string::String>,
}

#[derive(ValueEnum)]
pub enum OciRunEngineValue {
    #[arg(name = "auto")]
    Auto,
    #[arg(name = "podman")]
    Podman,
    #[arg(name = "docker")]
    Docker,
}

/// [experimental] Build OCI container images from a mise.toml
///
/// Each tool becomes its own OCI layer, so bumping any single tool version only invalidates one content-addressable blob — unlike a Dockerfile where changing an early `RUN` invalidates every layer above it.
///
/// This command is experimental and requires `mise settings experimental=true` (or `MISE_EXPERIMENTAL=1`). Behavior, flags, and output layout may change in future releases.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct OciArgs {
    #[arg(subcommand)]
    pub command: OciCommands,
}

#[derive(Subcommand)]
pub enum OciCommands {
    /// [experimental] Build an OCI image from the current mise.toml
    #[arg(name = "build")]
    Build(Box<OciBuildArgs>),
    /// [experimental] Build an OCI image and push it to a registry
    #[arg(name = "push")]
    Push(Box<OciPushArgs>),
    /// [experimental] Build an OCI image from the current mise.toml and run a command in it
    #[arg(name = "run")]
    Run(Box<OciRunArgs>),
}

/// Shows outdated tool versions
///
/// See `mise upgrade` to upgrade these versions.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mDeprecation:\u{1b}[22m\u{1b}[24m\n\nThe `-l` shorthand for `--bump` is deprecated and will be removed in mise 2027.8.5.\nAfter removal, `-l` will become shorthand for `--local`. Use `-b` or `--bump` instead.\n\n\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise outdated\u{1b}[22m\n    Plugin  Requested  Current  Latest\n    python  3.11       3.11.0   3.11.1\n    node    20         20.0.0   20.1.0\n\n    $ \u{1b}[1mmise outdated node\u{1b}[22m\n    Plugin  Requested  Current  Latest\n    node    20         20.0.0   20.1.0\n\n    $ \u{1b}[1mmise outdated --json\u{1b}[22m\n    {\"python\": {\"requested\": \"3.11\", \"current\": \"3.11.0\", \"latest\": \"3.11.1\"}, ...}\n\n    $ \u{1b}[1mmise outdated --local\u{1b}[22m\n    Plugin  Requested  Current  Latest\n    node    20         20.0.0   20.1.0\n"
)]
pub struct OutdatedArgs {
    /// Compares against the latest versions available, not what matches the current config
    ///
    /// For example, if you have `node = "20"` in your config by default `mise outdated` will only show other 20.x versions, not 21.x or 22.x versions.
    ///
    /// Using this flag, if there are 21.x or newer versions it will display those instead of 20.x.
    #[arg(long = "bump", short = 'b')]
    pub bump: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Deprecated shorthand for --bump
    #[arg(short = 'l', hide)]
    pub l: bool,
    /// Show outdated tools including installed-but-inactive tools not present in the current config
    ///
    /// By default, `mise outdated` only shows tools that come from the current config.
    #[arg(long = "inactive", conflicts("--local"))]
    pub inactive: bool,
    /// Only show outdated tools defined in local config files
    ///
    /// This will only show tools that are defined in project-local mise.toml and will skip tools defined in the global config (~/.config/mise/config.toml).
    #[arg(long = "local")]
    pub local: bool,
    /// Placeholder for future monorepo outdated checks; `mise outdated --monorepo` is not implemented yet.
    #[arg(long = "monorepo")]
    pub monorepo: bool,
    /// Don't show table header
    #[arg(long = "no-header")]
    pub no_header: bool,
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to show outdated versions for\ne.g.: node@20 python@3.10\nIf not specified, all tools in global and local configs will be shown"
    )]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Show the individuals supporting mise as Patron-tier members
///
/// Lists the individuals on the Patron tier from <https://jdx.dev/patrons.json>. The list refreshes daily; supporting terminals will render each patron's name as a clickable link via OSC 8 hyperlinks.
///
/// To appear here, become a patron at <https://jdx.dev/sponsors.html>.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise patrons\u{1b}[22m\n    $ \u{1b}[1mmise patrons -J\u{1b}[22m\n    $ \u{1b}[1mmise patrons --refresh\u{1b}[22m"
)]
pub struct PatronsArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Bypass the local cache and re-fetch
    #[arg(long = "refresh")]
    pub refresh: bool,
}

/// Install a plugin
///
/// note that mise can automatically install plugins when you install a tool e.g.: `mise install cmake@3.30` will autoinstall the cmake plugin
///
/// This behavior can be modified in ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # install the poetry via shorthand\n    $ \u{1b}[1mmise plugins install poetry\u{1b}[22m\n\n    # install the poetry plugin using a specific git url\n    $ \u{1b}[1mmise plugins install poetry https://github.com/mise-plugins/mise-poetry.git\u{1b}[22m\n\n    # install the poetry plugin using the git url only\n    # (poetry is inferred from the url)\n    $ \u{1b}[1mmise plugins install https://github.com/mise-plugins/mise-poetry.git\u{1b}[22m\n\n    # install the poetry plugin using a specific ref\n    $ \u{1b}[1mmise plugins install poetry https://github.com/mise-plugins/mise-poetry.git#11d0c1e\u{1b}[22m\n"
)]
pub struct PluginsInstallArgs {
    #[arg(
        help = "Install all missing plugins\nThis will only install plugins that have matching shorthands.\ni.e.: they don't need the full git repo url",
        long = "all",
        short = 'a',
        conflicts("NEW_PLUGIN", "--force")
    )]
    pub all: bool,
    /// Reinstall even if plugin exists
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1",
        long = "jobs",
        short = 'j',
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Show installation output
    #[arg(long = "verbose", short = 'v', count)]
    pub verbose: u8,
    #[arg(
        positional,
        value_name = "NEW_PLUGIN",
        help = "The name of the plugin to install\ne.g.: cmake, poetry\nCan specify multiple plugins: `mise plugins install cmake poetry`",
        required_unless("--all")
    )]
    pub new_plugin: ::std::option::Option<::std::string::String>,
    /// The git url of the plugin
    #[arg(positional, value_name = "GIT_URL")]
    pub git_url: ::std::option::Option<::std::string::String>,
    #[arg(positional, value_name = "REST", hide)]
    pub rest: ::std::vec::Vec<::std::string::String>,
}

/// Symlinks a plugin into mise
///
/// This is used for developing a plugin.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # essentially just `ln -s ./vfox-cmake ~/.local/share/mise/plugins/cmake`\n    $ \u{1b}[1mmise plugins link cmake ./vfox-cmake\u{1b}[22m\n\n    # infer plugin name as \"cmake\"\n    $ \u{1b}[1mmise plugins link ./vfox-cmake\u{1b}[22m\n"
)]
pub struct PluginsLinkArgs {
    /// Overwrite existing plugin
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    #[arg(
        positional,
        value_name = "NAME",
        help = "The name of the plugin\ne.g.: cmake, poetry"
    )]
    pub name: ::std::string::String,
    #[arg(
        positional,
        value_name = "DIR",
        help = "The local path to the plugin\ne.g.: ./vfox-cmake"
    )]
    pub dir: ::std::option::Option<::std::string::String>,
}

/// List installed plugins
///
/// Can also show remotely available plugins to install.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise plugins ls\u{1b}[22m\n    cmake\n    poetry\n\n    $ \u{1b}[1mmise plugins ls --urls\u{1b}[22m\n    cmake     https://github.com/mise-plugins/vfox-cmake.git\n    poetry    https://github.com/mise-plugins/vfox-poetry.git\n"
)]
pub struct PluginsLsArgs {
    #[arg(
        help = "List all available remote plugins\nSame as `mise plugins ls-remote`",
        long = "all",
        short = 'a',
        hide
    )]
    pub all: bool,
    #[arg(
        help = "The built-in plugins only\nNormally these are not shown",
        long = "core",
        short = 'c',
        hide,
        conflicts("--all")
    )]
    pub core: bool,
    #[arg(
        help = "Show plugins with available updates\nChecks the remote for newer versions and only displays plugins that are outdated",
        long = "outdated",
        short = 'o'
    )]
    pub outdated: bool,
    #[arg(
        help = "Show the git url for each plugin\ne.g.: https://github.com/mise-plugins/vfox-cmake.git",
        long = "urls",
        alias = "url",
        short = 'u'
    )]
    pub urls: bool,
    #[arg(
        help = "Show the git refs for each plugin\ne.g.: main 1234abc",
        long = "refs",
        hide
    )]
    pub refs: bool,
    /// List installed plugins
    #[arg(long = "user", hide, conflicts("--all"))]
    pub user: bool,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct PluginsLsRemoteArgs {
    #[arg(
        help = "Show the git url for each plugin e.g.: https://github.com/mise-plugins/mise-poetry.git",
        long_help = "Show the git url for each plugin\ne.g.: https://github.com/mise-plugins/mise-poetry.git",
        long = "urls",
        short = 'u'
    )]
    pub urls: bool,
    #[arg(
        help = "Only show the name of each plugin by default it will show a \"*\" next to installed plugins",
        long_help = "Only show the name of each plugin\nby default it will show a \"*\" next to installed plugins",
        long = "only-names"
    )]
    pub only_names: bool,
}

/// Removes a plugin
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise plugins uninstall cmake\u{1b}[22m\n"
)]
pub struct PluginsUninstallArgs {
    /// Remove all plugins
    #[arg(long = "all", short = 'a', conflicts("PLUGIN"))]
    pub all: bool,
    /// Also remove the plugin's installs, downloads, and cache
    #[arg(long = "purge", short = 'p')]
    pub purge: bool,
    /// Plugin(s) to remove
    #[arg(positional, value_name = "PLUGIN")]
    pub plugin: ::std::vec::Vec<::std::string::String>,
}

/// Updates a plugin to the latest version
///
/// note: this updates the plugin itself, not the runtime versions
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise plugins update\u{1b}[22m              # update all plugins\n    $ \u{1b}[1mmise plugins update cmake\u{1b}[22m       # update only cmake\n    $ \u{1b}[1mmise plugins update cmake#beta\u{1b}[22m  # specify a ref\n"
)]
pub struct PluginsUpdateArgs {
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1\nDefault: 4",
        long = "jobs",
        short = 'j',
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Plugin(s) to update
    #[arg(positional, value_name = "PLUGIN")]
    pub plugin: ::std::vec::Vec<::std::string::String>,
}

/// Manage plugins
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct PluginsArgs {
    /// list all available remote plugins
    ///
    /// same as `mise plugins ls-remote`
    #[arg(long = "all", short = 'a', hide)]
    pub all: bool,
    #[arg(
        help = "The built-in plugins only\nNormally these are not shown",
        long = "core",
        short = 'c',
        conflicts("--all")
    )]
    pub core: bool,
    #[arg(
        help = "Show the git url for each plugin\ne.g.: https://github.com/mise-plugins/vfox-cmake.git",
        long = "urls",
        alias = "url",
        short = 'u'
    )]
    pub urls: bool,
    #[arg(
        help = "Show the git refs for each plugin\ne.g.: main 1234abc",
        long = "refs",
        hide
    )]
    pub refs: bool,
    /// List installed plugins
    ///
    /// This is the default behavior but can be used with --core to show core and user plugins
    #[arg(long = "user", conflicts("--all"))]
    pub user: bool,
    #[arg(subcommand)]
    pub command: ::std::option::Option<PluginsCommands>,
}

#[derive(Subcommand)]
pub enum PluginsCommands {
    /// Install a plugin
    #[arg(name = "install", alias("i", "a", "add"))]
    Install(Box<PluginsInstallArgs>),
    /// Symlinks a plugin into mise
    #[arg(name = "link", alias = "ln")]
    Link(Box<PluginsLinkArgs>),
    /// List installed plugins
    #[arg(name = "ls", alias = "list")]
    Ls(Box<PluginsLsArgs>),
    /// List all available remote plugins
    #[arg(
        name = "ls-remote",
        help = "List all available remote plugins",
        long_help = "\nList all available remote plugins\n\nThe full list is here: https://github.com/jdx/mise/blob/main/registry/\n\nExamples:\n\n    $ mise plugins ls-remote\n",
        alias("list-remote", "list-all")
    )]
    LsRemote(Box<PluginsLsRemoteArgs>),
    /// Removes a plugin
    #[arg(name = "uninstall", alias("remove", "rm"))]
    Uninstall(Box<PluginsUninstallArgs>),
    /// Updates a plugin to the latest version
    #[arg(name = "update", alias("up", "upgrade"))]
    Update(Box<PluginsUpdateArgs>),
}

/// Add a dependency
///
/// Adds one or more packages to the project using the appropriate package manager. Package specs use the format `ecosystem:package`, e.g., `npm:react` or `npm:@types/react@19`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DepsAddArgs {
    /// Add as a development dependency
    #[arg(long = "dev", short = 'D')]
    pub dev: bool,
    /// Package(s) to add (e.g., npm:react, npm:@types/react@19)
    #[arg(positional, value_name = "PACKAGES", required)]
    pub packages: ::std::vec::Vec<::std::string::String>,
}

/// Install all project dependencies
///
/// Checks if dependency lockfiles are newer than installed outputs and runs install commands if needed.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DepsInstallArgs {
    /// Show why a provider is fresh or stale (requires a provider argument)
    #[arg(long = "explain")]
    pub explain: bool,
    /// Force run all deps steps even if outputs are fresh
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Only check if deps install is needed, don't run commands
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Show what deps providers are available
    #[arg(long = "list")]
    pub list: bool,
    /// Install dependencies from every [monorepo].config_roots config root
    ///
    /// Requires monorepo_root = true plus explicit [monorepo].config_roots in the monorepo root config. Providers are named like //apps/api:uv.
    #[arg(long = "monorepo", env = "MISE_MONOREPO")]
    pub monorepo: bool,
    /// Run specific deps rule(s) only
    #[arg(long = "only", value_name = "ONLY")]
    pub only: ::std::vec::Vec<::std::string::String>,
    /// Skip specific deps rule(s)
    #[arg(long = "skip", value_name = "SKIP")]
    pub skip: ::std::vec::Vec<::std::string::String>,
    /// Provider to operate on (runs only this provider, or use with --explain)
    #[arg(positional, value_name = "PROVIDER")]
    pub provider: ::std::option::Option<::std::string::String>,
}

/// Remove a dependency
///
/// Removes one or more packages from the project using the appropriate package manager. Package specs use the format `ecosystem:package`, e.g., `npm:lodash`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct DepsRemoveArgs {
    /// Package(s) to remove (e.g., npm:lodash)
    #[arg(positional, value_name = "PACKAGES", required)]
    pub packages: ::std::vec::Vec<::std::string::String>,
}

/// [experimental] Manage project dependencies
///
/// Runs all applicable dependency install steps for the current project. This checks if dependency lockfiles are newer than installed outputs (e.g., package-lock.json vs node_modules/) and runs install commands if needed.
///
/// Providers with `auto = true` are automatically invoked before `mise x` and `mise run` unless skipped with the --no-deps flag.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise deps\u{1b}[22m                    # Install all project dependencies\n    $ \u{1b}[1mmise deps install\u{1b}[22m            # Same as bare `mise deps`\n    $ \u{1b}[1mmise deps install --force\u{1b}[22m    # Force reinstall even if fresh\n    $ \u{1b}[1mmise deps install --dry-run\u{1b}[22m  # Show what would run\n    $ \u{1b}[1mmise deps --monorepo\u{1b}[22m         # Install deps from explicit monorepo config roots\n    $ \u{1b}[1mmise deps add npm:react\u{1b}[22m      # Add a dependency\n    $ \u{1b}[1mmise deps add -D npm:vitest\u{1b}[22m  # Add a dev dependency\n    $ \u{1b}[1mmise deps remove npm:lodash\u{1b}[22m  # Remove a dependency\n\n\u{1b}[1m\u{1b}[4mConfiguration:\u{1b}[22m\u{1b}[24m\n\n```toml\n# Built-in npm provider (auto-detects lockfile)\n[deps.npm]\nauto = true              # Auto-run before mise x/run\n\n# Custom provider\n[deps.codegen]\nauto = true\nsources = [\"schema/*.graphql\"]\noutputs = [\"src/generated/\"]\nrun = \"npm run codegen\"\n\n[deps]\ndisable = [\"npm\"]        # Disable specific providers at runtime\n```\n"
)]
pub struct DepsArgs {
    /// Show why a provider is fresh or stale (requires a provider argument)
    #[arg(long = "explain")]
    pub explain: bool,
    /// Force run all deps steps even if outputs are fresh
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Only check if deps install is needed, don't run commands
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Show what deps providers are available
    #[arg(long = "list")]
    pub list: bool,
    /// Install dependencies from every [monorepo].config_roots config root
    ///
    /// Requires monorepo_root = true plus explicit [monorepo].config_roots in the monorepo root config. Providers are named like //apps/api:uv.
    #[arg(long = "monorepo", env = "MISE_MONOREPO")]
    pub monorepo: bool,
    /// Run specific deps rule(s) only
    #[arg(long = "only", value_name = "ONLY")]
    pub only: ::std::vec::Vec<::std::string::String>,
    /// Skip specific deps rule(s)
    #[arg(long = "skip", value_name = "SKIP")]
    pub skip: ::std::vec::Vec<::std::string::String>,
    /// Provider to operate on (runs only this provider, or use with --explain)
    #[arg(positional, value_name = "PROVIDER")]
    pub provider: ::std::option::Option<::std::string::String>,
    #[arg(subcommand)]
    pub command: ::std::option::Option<DepsCommands>,
}

#[derive(Subcommand)]
pub enum DepsCommands {
    /// Add a dependency
    #[arg(name = "add")]
    Add(Box<DepsAddArgs>),
    /// Install all project dependencies
    #[arg(name = "install")]
    Install(Box<DepsInstallArgs>),
    /// Remove a dependency
    #[arg(name = "remove")]
    Remove(Box<DepsRemoveArgs>),
}

/// Delete unused versions of tools
///
/// mise tracks which config files have been used in ~/.local/state/mise/tracked-configs Versions which are no longer the latest specified in any of those configs are deleted. Versions installed only with environment variables `MISE_<TOOL>_VERSION` will be deleted, as will versions only referenced on the command line `mise exec <TOOL>@<VERSION>`.
///
/// Tool stubs that have been executed are tracked in ~/.local/state/mise/tracked-stubs. Versions still referenced by a tracked stub are not deleted.
///
/// You can list prunable tools with `mise ls --prunable`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise prune --dry-run\u{1b}[22m\n    rm -rf ~/.local/share/mise/versions/node/20.0.0\n    rm -rf ~/.local/share/mise/versions/node/20.0.1\n"
)]
pub struct PruneArgs {
    /// Do not actually delete anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Prune only tracked and trusted configuration links that point to nonexistent configurations
    #[arg(long = "configs")]
    pub configs: bool,
    /// Like --dry-run but exits with code 1 if there are tools to prune
    ///
    /// This is useful for scripts to check if tools need to be pruned.
    #[arg(long = "dry-run-code")]
    pub dry_run_code: bool,
    /// Placeholder for future monorepo pruning; `mise prune --monorepo` is not implemented yet.
    #[arg(long = "monorepo")]
    pub monorepo: bool,
    /// Prune only unused versions of tools
    #[arg(long = "tools")]
    pub tools: bool,
    /// Prune only these tools
    #[arg(positional, value_name = "INSTALLED_TOOL")]
    pub installed_tool: ::std::vec::Vec<::std::string::String>,
}

/// List available tools to install
///
/// This command lists the tools available in the registry as shorthand names.
///
/// For example, `poetry` is shorthand for `asdf:mise-plugins/mise-poetry`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise registry\u{1b}[22m\n    node    core:node\n    poetry  asdf:mise-plugins/mise-poetry\n    ubi     cargo:ubi-cli\n\n    $ \u{1b}[1mmise registry poetry\u{1b}[22m\n    asdf:mise-plugins/mise-poetry\n"
)]
pub struct RegistryArgs {
    /// Show only tools for this backend
    #[arg(long = "backend", short = 'b', value_name = "BACKEND")]
    pub backend: ::std::option::Option<::std::string::String>,
    /// Print all tools with descriptions for shell completions
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// Hide aliased tools
    #[arg(long = "hide-aliased")]
    pub hide_aliased: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Include security features for each tool's backends in JSON output.
    ///
    /// Requires --json. Security info is de-duplicated across all of a tool's backends. This can add noticeable time for large listings since each backend's security info is resolved individually.
    #[arg(long = "security", requires("--json"))]
    pub security: bool,
    /// Show only the specified tool's full name
    #[arg(positional, value_name = "NAME")]
    pub name: ::std::option::Option<::std::string::String>,
}

/// internal command to generate markdown from help
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct RenderHelpArgs {}

/// Creates new shims based on bin paths from currently installed tools.
///
/// This creates new shims in ~/.local/share/mise/shims for CLIs that have been added. mise will try to do this automatically for commands like `npm i -g` but there are other ways to install things (like using yarn or pnpm for node) that mise does not know about and so it will be necessary to call this explicitly.
///
/// If you think mise should automatically call this for a particular command, please open an issue on the mise repo. You can also set up a shell function to reshim automatically (it's really fast so you don't need to worry about overhead):
///
///     npm() {
///       command npm "$@"
///       mise reshim
///     }
///
/// Note that this creates shims for _all_ installed tools, not just the ones that are currently active in mise.toml.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise reshim\u{1b}[22m\n    $ \u{1b}[1m~/.local/share/mise/shims/node -v\u{1b}[22m\n    v20.0.0\n"
)]
pub struct ReshimArgs {
    /// Removes all shims before reshimming
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    #[arg(positional, value_name = "TOOL", hide)]
    pub tool: ::std::option::Option<::std::string::String>,
    #[arg(positional, value_name = "VERSION", hide)]
    pub version: ::std::option::Option<::std::string::String>,
}

/// Run task(s)
///
/// This command will run a task, or multiple tasks in parallel. Tasks may have dependencies on other tasks or on source files. If source is configured on a task, it will only run if the source files have changed.
///
/// Tasks can be defined in mise.toml or as standalone scripts. In mise.toml, tasks take this form:
///
///     [tasks.build]
///     run = "npm run build"
///     sources = ["src/**/*.ts"]
///     outputs = ["dist/**/*.js"]
///
/// Alternatively, tasks can be defined as standalone scripts. These must be located in `mise-tasks`, `.mise-tasks`, `.mise/tasks`, `mise/tasks` or `.config/mise/tasks`. The name of the script will be the name of the tasks.
///
///     $ cat .mise/tasks/build<<EOF
///     #!/usr/bin/env bash
///     npm run build
///     EOF
///     $ mise run build
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Runs the \"lint\" tasks. This needs to either be defined in mise.toml\n    # or as a standalone script. See the project README for more information.\n    $ \u{1b}[1mmise run lint\u{1b}[22m\n\n    # Forces the \"build\" tasks to run even if its sources are up-to-date.\n    $ \u{1b}[1mmise run --force build\u{1b}[22m\n\n    # Run \"test\" with stdin/stdout/stderr all connected to the current terminal.\n    # This forces `--jobs=1` to prevent interleaving of output.\n    $ \u{1b}[1mmise run --raw test\u{1b}[22m\n\n    # Runs the \"lint\", \"test\", and \"check\" tasks in parallel.\n    $ \u{1b}[1mmise run lint ::: test ::: check\u{1b}[22m\n\n    # Execute multiple tasks each with their own arguments.\n    $ \u{1b}[1mmise run cmd1 arg1 arg2 ::: cmd2 arg1 arg2\u{1b}[22m\n",
    restart_token = ":::",
    disable_help_flag
)]
pub struct RunArgs {
    /// Run matching tasks only for projects affected by Git changes
    #[arg(long = "affected")]
    pub affected: bool,
    #[arg(
        help = "Git base revision for --affected\nDefaults to MISE_AFFECTED_BASE, CI metadata, or HEAD~1",
        long = "affected-base",
        requires("--affected"),
        value_name = "REV"
    )]
    pub affected_base: ::std::option::Option<::std::string::String>,
    /// Explain why projects and tasks were selected by --affected
    #[arg(
        long = "affected-explain",
        conflicts("--affected-json"),
        requires("--affected")
    )]
    pub affected_explain: bool,
    #[arg(
        help = "Git head revision for --affected\nDefaults to MISE_AFFECTED_HEAD, CI metadata, or HEAD",
        long = "affected-head",
        requires("--affected"),
        value_name = "REV"
    )]
    pub affected_head: ::std::option::Option<::std::string::String>,
    /// Output affected projects and tasks as JSON without running tasks
    #[arg(
        long = "affected-json",
        conflicts("--affected-explain"),
        requires("--affected")
    )]
    pub affected_json: bool,
    /// Open the interactive selector with all tasks from the entire monorepo
    #[arg(long = "all", conflicts("--affected"))]
    pub all: bool,
    /// Continue running tasks even if one fails
    #[arg(long = "continue-on-error", short = 'c')]
    pub continue_on_error: bool,
    /// Change to this directory before executing the command
    #[arg(long = "cd", short = 'C', value_name = "CD")]
    pub cd: ::std::option::Option<::std::string::String>,
    /// Force the tasks to run even if outputs are up to date
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    #[arg(
        help = "Number of tasks to run in parallel\nValues below 1 are treated as 1\n[default: 4]\nConfigure with `jobs` config or `MISE_JOBS` env var",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Don't actually run the task(s), just print them in order of execution
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Change how tasks information is output when running tasks
    ///
    /// - `prefix` - Print stdout/stderr by line, prefixed with the task's label - `interleave` - Print directly to stdout/stderr instead of by line - `replacing` - Stdout is replaced each time, stderr is printed as is - `timed` - Only show stdout lines if they are displayed for more than 1 second - `keep-order` - Print stdout/stderr by line, prefixed with the task's label, but keep the order of the output - `quiet` - Don't show extra output - `silent` - Don't show any output including stdout and stderr from the task except for errors
    #[arg(
        long = "output",
        short = 'o',
        env = "MISE_TASK_OUTPUT",
        value_name = "OUTPUT"
    )]
    pub output: ::std::option::Option<::std::string::String>,
    /// Don't show extra output
    #[arg(long = "quiet", short = 'q', env = "MISE_QUIET")]
    pub quiet: bool,
    #[arg(
        help = "Read/write directly to stdin/stdout/stderr instead of by line\nRedactions are not applied with this option\nConfigure with `raw` config or `MISE_RAW` env var",
        long = "raw",
        short = 'r'
    )]
    pub raw: bool,
    /// Shell to use to run toml tasks
    ///
    /// Defaults to `sh -c -o errexit -o pipefail` on unix, and `cmd /c` on Windows Can also be set with the setting `MISE_UNIX_DEFAULT_INLINE_SHELL_ARGS` or `MISE_WINDOWS_DEFAULT_INLINE_SHELL_ARGS` Or it can be overridden with the `shell` property on a task.
    #[arg(long = "shell", short = 's', value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Don't show any output except for errors
    #[arg(long = "silent", short = 'S', env = "MISE_SILENT")]
    pub silent: bool,
    #[arg(
        help = "Tool(s) to run in addition to what is in mise.toml files e.g.: node@20 python@3.10",
        long_help = "Tool(s) to run in addition to what is in mise.toml files\ne.g.: node@20 python@3.10",
        long = "tool",
        short = 't',
        value_name = "TOOL@VERSION"
    )]
    pub tool: ::std::vec::Vec<::std::string::String>,
    #[arg(
        help = "Allow specific env var through (implies --deny-env for everything else)\nSupports wildcards, e.g. --allow-env='MYAPP_*'",
        long = "allow-env",
        value_name = "VAR"
    )]
    pub allow_env: ::std::vec::Vec<::std::string::String>,
    /// Allow network to specific host (implies --deny-net for everything else)
    #[arg(long = "allow-net", value_name = "HOST")]
    pub allow_net: ::std::vec::Vec<::std::string::String>,
    /// Allow reads from specific path (implies --deny-read for everything else)
    #[arg(long = "allow-read", value_name = "PATH")]
    pub allow_read: ::std::vec::Vec<::std::string::String>,
    /// Allow writes to specific path (implies --deny-write for everything else)
    #[arg(long = "allow-write", value_name = "PATH")]
    pub allow_write: ::std::vec::Vec<::std::string::String>,
    /// Block reads, writes, network, and env vars
    #[arg(long = "deny-all")]
    pub deny_all: bool,
    /// Block env var inheritance (only PATH, HOME, USER, SHELL, TERM, LANG pass through)
    #[arg(long = "deny-env")]
    pub deny_env: bool,
    /// Block all network access
    #[arg(long = "deny-net")]
    pub deny_net: bool,
    /// Block filesystem reads (system libs and tool dirs still accessible)
    #[arg(long = "deny-read")]
    pub deny_read: bool,
    /// Block all filesystem writes
    #[arg(long = "deny-write")]
    pub deny_write: bool,
    /// Bypass the environment cache and recompute the environment
    #[arg(long = "fresh-env")]
    pub fresh_env: bool,
    /// Do not use cache on remote tasks
    #[arg(long = "no-cache", env = "MISE_TASK_REMOTE_NO_CACHE")]
    pub no_cache: bool,
    /// Skip automatic dependency preparation
    #[arg(long = "no-deps")]
    pub no_deps: bool,
    /// Hides elapsed time after each task completes
    ///
    /// Default to always hide with `MISE_TASK_TIMINGS=0`
    #[arg(long = "no-timings", alias = "no-timing")]
    pub no_timings: bool,
    /// Run only the specified tasks skipping all dependencies
    #[arg(long = "skip-deps", env = "MISE_TASK_SKIP_DEPENDS")]
    pub skip_deps: bool,
    /// Skip installing tools before running tasks
    ///
    /// Can also be set persistently with the `task.run_auto_install` setting or `MISE_TASK_RUN_AUTO_INSTALL=false` env var
    #[arg(long = "skip-tools")]
    pub skip_tools: bool,
    /// Set task output cache access for this run
    ///
    /// - `read-write` - Read cached results and write new results - `read-only` - Read cached results without writing new results - `write-only` - Write new results without reading cached results - `off` - Disable task output caching - `local-only` - Read and write only the local cache; currently equivalent to `read-write`
    #[arg(
        long = "task-cache",
        env = "MISE_TASK_CACHE",
        value_name = "TASK_CACHE",
        default = "read-write"
    )]
    pub task_cache: ::std::option::Option<RunTaskCacheValue>,
    /// Explain the inputs that produced each task's output cache key
    #[arg(long = "task-cache-explain")]
    pub task_cache_explain: bool,
    /// Output cache-key input details as JSON Lines without running tasks
    #[arg(
        long = "task-cache-explain-json",
        conflicts("--task-cache-explain"),
        requires("--dry-run")
    )]
    pub task_cache_explain_json: bool,
    /// Report task output cache hits, restored bytes, and time saved
    #[arg(long = "task-cache-stats", conflicts("--dry-run"))]
    pub task_cache_stats: bool,
    #[arg(
        help = "Timeout for the task to complete\ne.g.: 30s, 5m",
        long = "timeout",
        value_name = "TIMEOUT"
    )]
    pub timeout: ::std::option::Option<::std::string::String>,
    /// Shows elapsed time after each task completes
    ///
    /// Default to always show with `MISE_TASK_TIMINGS=1`
    #[arg(long = "timings", alias = "timing", hide)]
    pub timings: bool,
}

#[derive(ValueEnum)]
pub enum RunTaskCacheValue {
    /// Read cached results and write new results.
    #[arg(name = "read-write")]
    ReadWrite,
    /// Read cached results without writing new results.
    #[arg(name = "read-only")]
    ReadOnly,
    /// Write new results without reading cached results.
    #[arg(name = "write-only")]
    WriteOnly,
    /// Disable task output caching for this run.
    #[arg(name = "off")]
    Off,
    /// Read and write only the local cache.
    #[arg(name = "local-only")]
    LocalOnly,
}

/// Search for tools in the registry
///
/// This command searches a tool in the registry.
///
/// By default, it will show all tools that fuzzy match the search term. For non-fuzzy matches, use the `--match-type` flag.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise search jq\u{1b}[22m\n    Tool  Description\n    jq    Command-line JSON processor. https://github.com/jqlang/jq\n    jqp   A TUI playground to experiment with jq. https://github.com/noahgorstein/jqp\n    jiq   jid on jq - interactive JSON query tool using jq expressions. https://github.com/fiatjaf/jiq\n    gojq  Pure Go implementation of jq. https://github.com/itchyny/gojq\n\n    $ \u{1b}[1mmise search --interactive\u{1b}[22m\n    Tool\n    Search a tool\n    ❯ jq    Command-line JSON processor. https://github.com/jqlang/jq\n      jqp   A TUI playground to experiment with jq. https://github.com/noahgorstein/jqp\n      jiq   jid on jq - interactive JSON query tool using jq expressions. https://github.com/fiatjaf/jiq\n      gojq  Pure Go implementation of jq. https://github.com/itchyny/gojq\n    /jq \n    esc clear filter • enter confirm\n"
)]
pub struct SearchArgs {
    /// Show interactive search
    #[arg(
        long = "interactive",
        short = 'i',
        conflicts("--match-type", "--no-header")
    )]
    pub interactive: bool,
    /// Match type: equal, contains, or fuzzy
    #[arg(
        long = "match-type",
        short = 'm',
        value_name = "MATCH_TYPE",
        default = "fuzzy"
    )]
    pub match_type: ::std::option::Option<SearchMatchTypeValue>,
    /// Don't display headers
    #[arg(long = "no-header", alias = "no-headers")]
    pub no_header: bool,
    /// The tool to search for
    #[arg(positional, value_name = "NAME")]
    pub name: ::std::option::Option<::std::string::String>,
}

#[derive(ValueEnum)]
pub enum SearchMatchTypeValue {
    #[arg(name = "equal")]
    Equal,
    #[arg(name = "contains")]
    Contains,
    #[arg(name = "fuzzy")]
    Fuzzy,
}

/// Updates mise itself.
///
/// Uses the GitHub Releases API to find the latest release and binary. By default, this will also update any installed plugins. Uses mise's GitHub token resolution chain for authenticated requests.
///
/// Packagers can disable this command so that mise is updated through the package manager instead. See https://mise.jdx.dev/contributing.html#packaging-and-self-update-instructions
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct SelfUpdateArgs {
    /// Update even if already up to date
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    /// Skip confirmation prompt
    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
    /// Disable auto-updating plugins
    #[arg(long = "no-plugins")]
    pub no_plugins: bool,
    /// Update to a specific version
    #[arg(positional, value_name = "VERSION")]
    pub version: ::std::option::Option<::std::string::String>,
}

/// Set environment variables in mise.toml
///
/// By default, this command modifies `mise.toml` in the current directory. If multiple config files exist (e.g., both `mise.toml` and `mise.local.toml`), the lowest precedence file (`mise.toml`) will be used. See https://mise.jdx.dev/configuration.html#target-file-for-write-operations
///
/// Use `-E <env>` to create/modify environment-specific config files like `mise.<env>.toml`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise set NODE_ENV=production\u{1b}[22m\n\n    $ \u{1b}[1mmise set NODE_ENV\u{1b}[22m\n    production\n\n    $ \u{1b}[1mmise set -E staging NODE_ENV=staging\u{1b}[22m\n    # creates or modifies mise.staging.toml\n\n    $ \u{1b}[1mmise set\u{1b}[22m\n    key       value       source\n    NODE_ENV  production  ~/.config/mise/config.toml\n\n    $ \u{1b}[1mmise set --prompt PASSWORD\u{1b}[22m\n    Enter value for PASSWORD: [hidden input]\n\n    \u{1b}[1m\u{1b}[4mMultiline Values (--stdin):\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mcat private.key | mise set --stdin MY_KEY\u{1b}[22m\n\n    $ \u{1b}[1mprintf \"line1\\nline2\" | mise set --stdin MY_KEY\u{1b}[22m\n\n    \u{1b}[1m\u{1b}[4m[experimental] Age Encryption:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise set --age-encrypt API_KEY=secret\u{1b}[22m\n\n    $ \u{1b}[1mmise set --age-encrypt --prompt API_KEY\u{1b}[22m\n    Enter value for API_KEY: [hidden input]\n"
)]
pub struct SetArgs {
    /// Create/modify an environment-specific config file like .mise.<env>.toml
    #[arg(
        long = "env",
        short = 'E',
        overrides("--global", "--file"),
        value_name = "ENV"
    )]
    pub env: ::std::option::Option<::std::string::String>,
    /// Set the environment variable in the global config file
    #[arg(long = "global", short = 'g', overrides("--file", "--env"))]
    pub global: bool,
    /// [experimental] Encrypt the value with age before storing
    #[arg(long = "age-encrypt", requires("ENV_VAR"))]
    pub age_encrypt: bool,
    /// [experimental] Age identity file for encryption
    ///
    /// Defaults to ~/.config/mise/age.txt if it exists
    #[arg(long = "age-key-file", requires("--age-encrypt"), value_name = "PATH")]
    pub age_key_file: ::std::option::Option<::std::string::String>,
    /// [experimental] Age recipient (x25519 public key) for encryption
    ///
    /// Can be used multiple times. Requires --age-encrypt.
    #[arg(
        long = "age-recipient",
        requires("--age-encrypt"),
        value_name = "RECIPIENT"
    )]
    pub age_recipient: ::std::vec::Vec<::std::string::String>,
    /// [experimental] SSH recipient (public key or path) for age encryption
    ///
    /// Can be used multiple times. Requires --age-encrypt.
    #[arg(
        long = "age-ssh-recipient",
        requires("--age-encrypt"),
        value_name = "PATH_OR_PUBKEY"
    )]
    pub age_ssh_recipient: ::std::vec::Vec<::std::string::String>,
    /// Render completions
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// The TOML file to update
    ///
    /// Can be a file path or directory. If a directory is provided, will create/use mise.toml in that directory. Defaults to [`MISE_DEFAULT_CONFIG_FILENAME`](https://mise.jdx.dev/configuration.html#mise_default_config_filename) environment variable, or `mise.toml`. Use [`MISE_GLOBAL_CONFIG_FILE`](https://mise.jdx.dev/configuration.html#mise_global_config_file) to choose a different global config path.
    #[arg(long = "file", long = "path", value_name = "FILE")]
    pub file: ::std::option::Option<::std::string::String>,
    /// Show raw values instead of redacting secrets
    #[arg(long = "no-redact")]
    pub no_redact: bool,
    /// Prompt for environment variable values
    #[arg(long = "prompt")]
    pub prompt: bool,
    /// Remove the environment variable from config file
    ///
    /// Can be used multiple times.
    #[arg(
        long = "remove",
        long = "rm",
        long = "unset",
        hide,
        value_name = "ENV_KEY"
    )]
    pub remove: ::std::vec::Vec<::std::string::String>,
    /// Read the value from stdin (for multiline input)
    ///
    /// When using --stdin, provide a single key without a value. The value will be read from stdin until EOF.
    #[arg(long = "stdin", conflicts("--prompt"), requires("ENV_VAR"))]
    pub stdin: bool,
    #[arg(
        positional,
        value_name = "ENV_VAR",
        help = "Environment variable(s) to set\ne.g.: NODE_ENV=production"
    )]
    pub env_var: ::std::vec::Vec<::std::string::String>,
}

/// Adds a setting to the configuration file
///
/// Used with an array setting, this will append the value to the array. This modifies the contents of ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise settings add disable_hints python_multi\u{1b}[22m\n"
)]
pub struct SettingsAddArgs {
    /// Use the local config file instead of the global one
    #[arg(long = "local", short = 'l')]
    pub local: bool,
    /// The setting to set
    #[arg(positional, value_name = "SETTING")]
    pub setting: ::std::string::String,
    /// The value to set (optional if provided as KEY=VALUE)
    #[arg(positional, value_name = "VALUE")]
    pub value: ::std::option::Option<::std::string::String>,
}

/// Show a current setting
///
/// This is the contents of a single entry in ~/.config/mise/config.toml
///
/// Note that aliases are also stored in this file but managed separately with `mise tool-alias get`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise settings get idiomatic_version_file\u{1b}[22m\n    true\n"
)]
pub struct SettingsGetArgs {
    /// Use the local config file instead of the global one
    #[arg(long = "local", short = 'l')]
    pub local: bool,
    /// The setting to show
    #[arg(positional, value_name = "SETTING")]
    pub setting: ::std::string::String,
}

/// Show current settings
///
/// This is the contents of ~/.config/mise/config.toml
///
/// Note that aliases are also stored in this file but managed separately with `mise tool-alias`
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise settings ls\u{1b}[22m\n    idiomatic_version_file = false\n    ...\n\n    $ \u{1b}[1mmise settings ls python\u{1b}[22m\n    default_packages_file = \"~/.default-python-packages\"\n    ...\n",
    group("output")
)]
pub struct SettingsLsArgs {
    /// List all settings
    #[arg(long = "all", short = 'a')]
    pub all: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J', group = "output")]
    pub json: bool,
    /// Use the local config file instead of the global one
    #[arg(long = "local", short = 'l', global)]
    pub local: bool,
    /// Output in TOML format
    #[arg(long = "toml", short = 'T', group = "output")]
    pub toml: bool,
    /// Print all settings with descriptions for shell completions
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// Output in JSON format with sources
    #[arg(long = "json-extended", group = "output")]
    pub json_extended: bool,
    /// Name of setting
    #[arg(positional, value_name = "SETTING")]
    pub setting: ::std::option::Option<::std::string::String>,
}

/// Add/update a setting
///
/// This modifies the contents of ~/.config/mise/config.toml by default. With `--local`, modifies the local config file instead. See https://mise.jdx.dev/configuration.html#target-file-for-write-operations
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise settings idiomatic_version_file=true\u{1b}[22m\n"
)]
pub struct SettingsSetArgs {
    /// Use the local config file instead of the global one
    #[arg(long = "local", short = 'l')]
    pub local: bool,
    /// The setting to set
    #[arg(positional, value_name = "SETTING")]
    pub setting: ::std::string::String,
    /// The value to set (optional if provided as KEY=VALUE)
    #[arg(positional, value_name = "VALUE")]
    pub value: ::std::option::Option<::std::string::String>,
}

/// Clears a setting
///
/// This modifies the contents of ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise settings unset idiomatic_version_file\u{1b}[22m\n"
)]
pub struct SettingsUnsetArgs {
    /// Use the local config file instead of the global one
    #[arg(long = "local", short = 'l')]
    pub local: bool,
    /// The setting to remove
    #[arg(positional, value_name = "KEY")]
    pub key: ::std::string::String,
}

/// Manage settings
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # list all settings\n    $ \u{1b}[1mmise settings\u{1b}[22m\n\n    # get the value of the setting \"always_keep_download\"\n    $ \u{1b}[1mmise settings always_keep_download\u{1b}[22m\n\n    # set the value of the setting \"always_keep_download\" to \"true\"\n    $ \u{1b}[1mmise settings always_keep_download=true\u{1b}[22m\n\n    # set the value of the setting \"node.mirror_url\" to \"https://npmmirror.com/mirrors/node/\"\n    $ \u{1b}[1mmise settings node.mirror_url https://npmmirror.com/mirrors/node/\u{1b}[22m\n",
    group("output")
)]
pub struct SettingsArgs {
    /// List all settings
    #[arg(long = "all", short = 'a')]
    pub all: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J', group = "output")]
    pub json: bool,
    /// Use the local config file instead of the global one
    #[arg(long = "local", short = 'l', global)]
    pub local: bool,
    /// Output in TOML format
    #[arg(long = "toml", short = 'T', group = "output")]
    pub toml: bool,
    /// Print all settings with descriptions for shell completions
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// Output in JSON format with sources
    #[arg(long = "json-extended", group = "output")]
    pub json_extended: bool,
    /// Name of setting
    #[arg(positional, value_name = "SETTING")]
    pub setting: ::std::option::Option<::std::string::String>,
    /// Setting value to set
    #[arg(positional, value_name = "VALUE", conflicts("--all"))]
    pub value: ::std::option::Option<::std::string::String>,
    #[arg(subcommand)]
    pub command: ::std::option::Option<SettingsCommands>,
}

#[derive(Subcommand)]
pub enum SettingsCommands {
    /// Adds a setting to the configuration file
    #[arg(name = "add")]
    Add(Box<SettingsAddArgs>),
    /// Show a current setting
    #[arg(name = "get")]
    Get(Box<SettingsGetArgs>),
    /// Show current settings
    #[arg(name = "ls", alias = "list")]
    Ls(Box<SettingsLsArgs>),
    /// Add/update a setting
    #[arg(name = "set", alias = "create")]
    Set(Box<SettingsSetArgs>),
    /// Clears a setting
    #[arg(name = "unset", alias("rm", "remove", "delete", "del"))]
    Unset(Box<SettingsUnsetArgs>),
}

/// Sets a tool version for the current session.
///
/// Only works in a session where mise is already activated.
///
/// This works by setting environment variables for the current shell session such as `MISE_NODE_VERSION=20` which is "eval"ed as a shell function created by `mise activate`.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise shell node@20\u{1b}[22m\n    $ \u{1b}[1mnode -v\u{1b}[22m\n    v20.0.0\n"
)]
pub struct ShellArgs {
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1\n[default: 4]",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Removes a previously set version
    #[arg(long = "unset", short = 'u')]
    pub unset: bool,
    #[arg(
        help = "Connect backend install command stdin/stdout/stderr directly to the terminal Implies --jobs=1",
        long_help = "Connect backend install command stdin/stdout/stderr directly to the terminal\nImplies --jobs=1",
        long = "raw",
        overrides("--jobs")
    )]
    pub raw: bool,
    /// Tool(s) to use
    #[arg(positional, value_name = "TOOL@VERSION", required)]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Show the command for a shell alias
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise shell-alias get ll\u{1b}[22m\n    ls -la\n"
)]
pub struct ShellAliasGetArgs {
    /// The alias to show
    #[arg(positional, value_name = "shell_alias")]
    pub shell_alias: ::std::string::String,
}

/// List shell aliases
///
/// Shows the shell aliases that are set in the current directory. These are defined in `mise.toml` under the `[shell_alias]` section.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise shell-alias ls\u{1b}[22m\n    alias    command\n    ll       ls -la\n    gs       git status\n"
)]
pub struct ShellAliasLsArgs {
    /// Don't show table header
    #[arg(long = "no-header")]
    pub no_header: bool,
}

/// Add/update a shell alias
///
/// This modifies the contents of ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise shell-alias set ll \"ls -la\"\u{1b}[22m\n    $ \u{1b}[1mmise shell-alias set gs \"git status\"\u{1b}[22m\n"
)]
pub struct ShellAliasSetArgs {
    /// The alias name
    #[arg(positional, value_name = "shell_alias")]
    pub shell_alias: ::std::string::String,
    /// The command to run (optional if provided as ALIAS=COMMAND)
    #[arg(positional, value_name = "COMMAND")]
    pub command: ::std::option::Option<::std::string::String>,
}

/// Removes a shell alias
///
/// This modifies the contents of ~/.config/mise/config.toml
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise shell-alias unset ll\u{1b}[22m\n"
)]
pub struct ShellAliasUnsetArgs {
    /// The alias to remove
    #[arg(positional, value_name = "shell_alias")]
    pub shell_alias: ::std::string::String,
}

/// Manage shell aliases.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct ShellAliasArgs {
    /// Don't show table header
    #[arg(long = "no-header")]
    pub no_header: bool,
    #[arg(subcommand)]
    pub command: ::std::option::Option<ShellAliasCommands>,
}

#[derive(Subcommand)]
pub enum ShellAliasCommands {
    /// Show the command for a shell alias
    #[arg(name = "get")]
    Get(Box<ShellAliasGetArgs>),
    /// List shell aliases
    #[arg(name = "ls", alias = "list")]
    Ls(Box<ShellAliasLsArgs>),
    /// Add/update a shell alias
    #[arg(name = "set", alias("add", "create"))]
    Set(Box<ShellAliasSetArgs>),
    /// Removes a shell alias
    #[arg(name = "unset", alias("rm", "remove", "delete", "del"))]
    Unset(Box<ShellAliasUnsetArgs>),
}

/// Show the companies sponsoring mise and the jdx.dev open source tools
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct SponsorsArgs {}

/// Symlinks all tool versions from an external tool into mise
///
/// For example, use this to import all Homebrew node installs into mise
///
/// This won't overwrite managed installs, runtime aliases, or links from other providers.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mbrew install node@18 node@20\u{1b}[22m\n    $ \u{1b}[1mmise sync node --brew\u{1b}[22m\n    $ \u{1b}[1mmise use -g node@18\u{1b}[22m - uses Homebrew-provided node\n",
    group("SyncNodeType", required, multiple)
)]
pub struct SyncNodeArgs {
    /// Get tool versions from Homebrew
    #[arg(long = "brew", group = "SyncNodeType")]
    pub brew: bool,
    /// Get tool versions from nodenv
    #[arg(long = "nodenv", group = "SyncNodeType")]
    pub nodenv: bool,
    /// Get tool versions from nvm
    #[arg(long = "nvm", group = "SyncNodeType")]
    pub nvm: bool,
}

/// Symlinks all tool versions from an external tool into mise
///
/// For example, use this to import all pyenv installs into mise
///
/// This won't overwrite managed installs, runtime aliases, or links from other providers.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mpyenv install 3.11.0\u{1b}[22m\n    $ \u{1b}[1mmise sync python --pyenv\u{1b}[22m\n    $ \u{1b}[1mmise use -g python@3.11.0\u{1b}[22m - uses pyenv-provided python\n    \n    $ \u{1b}[1muv python install 3.11.0\u{1b}[22m\n    $ \u{1b}[1mmise install python@3.10.0\u{1b}[22m\n    $ \u{1b}[1mmise sync python --uv\u{1b}[22m\n    $ \u{1b}[1mmise x python@3.11.0 -- python -V\u{1b}[22m - uses uv-provided python\n    $ \u{1b}[1muv run -p 3.10.0 -- python -V\u{1b}[22m - uses mise-provided python\n"
)]
pub struct SyncPythonArgs {
    /// Get tool versions from pyenv
    #[arg(long = "pyenv")]
    pub pyenv: bool,
    /// Sync tool versions with uv (2-way sync)
    #[arg(long = "uv")]
    pub uv: bool,
}

/// Symlinks all ruby tool versions from an external tool into mise
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mbrew install ruby\u{1b}[22m\n    $ \u{1b}[1mmise sync ruby --brew\u{1b}[22m\n    $ \u{1b}[1mmise use -g ruby\u{1b}[22m - Use the latest version of Ruby installed by Homebrew\n"
)]
pub struct SyncRubyArgs {
    /// Get tool versions from Homebrew
    #[arg(long = "brew", required)]
    pub brew: bool,
}

/// Synchronize tools from other version managers with mise
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct SyncArgs {
    #[arg(subcommand)]
    pub command: SyncCommands,
}

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Symlinks all tool versions from an external tool into mise
    #[arg(name = "node")]
    Node(Box<SyncNodeArgs>),
    /// Symlinks all tool versions from an external tool into mise
    #[arg(name = "python")]
    Python(Box<SyncPythonArgs>),
    /// Symlinks all ruby tool versions from an external tool into mise
    #[arg(name = "ruby")]
    Ruby(Box<SyncRubyArgs>),
}

/// Create a new task
///
/// Adds a task to the local mise.toml file. See https://mise.jdx.dev/configuration.html#target-file-for-write-operations
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tasks add pre-commit --depends \"test\" --depends \"render\" -- echo pre-commit\u{1b}[22m\n"
)]
pub struct TasksAddArgs {
    /// Other names for the task
    #[arg(long = "alias", short = 'a', value_name = "ALIAS")]
    pub alias: ::std::vec::Vec<::std::string::String>,
    /// Add dependencies to the task
    #[arg(long = "depends", short = 'd', value_name = "DEPENDS")]
    pub depends: ::std::vec::Vec<::std::string::String>,
    /// Run the task in a specific directory
    #[arg(long = "dir", short = 'D', value_name = "DIR")]
    pub dir: ::std::option::Option<::std::string::String>,
    /// Create a file task instead of a toml task
    #[arg(long = "file", short = 'f')]
    pub file: bool,
    /// Hide the task from `mise tasks` and completions
    #[arg(long = "hide", short = 'H')]
    pub hide: bool,
    /// Do not print the command before running
    #[arg(long = "quiet", short = 'q')]
    pub quiet: bool,
    /// Directly connect stdin/stdout/stderr
    #[arg(long = "raw", short = 'r')]
    pub raw: bool,
    /// Glob patterns of files this task uses as input
    #[arg(long = "sources", short = 's', value_name = "SOURCES")]
    pub sources: ::std::vec::Vec<::std::string::String>,
    /// Wait for these tasks to complete if they are to run
    #[arg(long = "wait-for", short = 'w', value_name = "WAIT_FOR")]
    pub wait_for: ::std::vec::Vec<::std::string::String>,
    /// Dependencies to run after the task runs
    #[arg(long = "depends-post", value_name = "DEPENDS_POST")]
    pub depends_post: ::std::vec::Vec<::std::string::String>,
    /// Description of the task
    #[arg(long = "description", value_name = "DESCRIPTION")]
    pub description: ::std::option::Option<::std::string::String>,
    /// Glob patterns of files this task creates, to skip if they are not modified
    #[arg(long = "outputs", value_name = "OUTPUTS")]
    pub outputs: ::std::vec::Vec<::std::string::String>,
    /// Command to run on windows
    #[arg(long = "run-windows", value_name = "RUN_WINDOWS")]
    pub run_windows: ::std::option::Option<::std::string::String>,
    /// Run the task in a specific shell
    #[arg(long = "shell", value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Do not print the command or its output
    #[arg(long = "silent")]
    pub silent: bool,
    /// Tasks name to add
    #[arg(positional, value_name = "TASK")]
    pub task: ::std::string::String,
    #[arg(positional, value_name = "RUN", double_dash = "required")]
    pub run: ::std::vec::Vec<::std::string::String>,
}

/// Display a tree visualization of a dependency graph
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Show dependencies for all tasks\n    $ \u{1b}[1mmise tasks deps\u{1b}[22m\n\n    # Show dependencies for the \"lint\", \"test\" and \"check\" tasks\n    $ \u{1b}[1mmise tasks deps lint test check\u{1b}[22m\n\n    # Show dependencies in DOT format\n    $ \u{1b}[1mmise tasks deps --dot\u{1b}[22m\n\n    # Collapse repeated dependencies\n    $ \u{1b}[1mmise tasks deps --compact\u{1b}[22m\n"
)]
pub struct TasksDepsArgs {
    /// Collapse repeated dependencies after their first occurrence
    #[arg(long = "compact", conflicts("--dot"))]
    pub compact: bool,
    /// Display dependencies in DOT format
    #[arg(long = "dot")]
    pub dot: bool,
    /// Show hidden tasks
    #[arg(long = "hidden")]
    pub hidden: bool,
    #[arg(
        positional,
        value_name = "TASKS",
        help = "Tasks to show dependencies for\nCan specify multiple tasks by separating with spaces\ne.g.: mise tasks deps lint test check"
    )]
    pub tasks: ::std::vec::Vec<::std::string::String>,
}

/// Edit a task with $EDITOR
///
/// The task will be created as a standalone script if it does not already exist.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tasks edit build\u{1b}[22m\n    $ \u{1b}[1mmise tasks edit test\u{1b}[22m\n"
)]
pub struct TasksEditArgs {
    /// Display the path to the task instead of editing it
    #[arg(long = "path", short = 'p')]
    pub path: bool,
    /// Task to edit
    #[arg(positional, value_name = "TASK")]
    pub task: ::std::string::String,
}

/// [experimental] Inspect the workspace project graph
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Inspect projects and their dependency edges\n    $ \u{1b}[1mmise tasks graph\u{1b}[22m\n\n    # Emit the project graph as JSON\n    $ \u{1b}[1mmise tasks graph --json\u{1b}[22m\n\n    # Explain where inferred projects and task fields came from\n    $ \u{1b}[1mmise tasks graph --explain\u{1b}[22m\n"
)]
pub struct TasksGraphArgs {
    /// Output the project graph as JSON
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Explain provider attribution for inferred projects and tasks
    #[arg(long = "explain", conflicts("--json"))]
    pub explain: bool,
    /// Do not print table headers
    #[arg(long = "no-header", alias = "no-headers")]
    pub no_header: bool,
}

/// Get information about a task
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tasks info\u{1b}[22m\n    Name: test\n    Aliases: t\n    Description: Test the application\n    Source: ~/src/myproj/mise.toml\n\n    $ \u{1b}[1mmise tasks info test --json\u{1b}[22m\n    {\n      \"name\": \"test\",\n      \"aliases\": \"t\",\n      \"description\": \"Test the application\",\n      \"source\": \"~/src/myproj/mise.toml\",\n      \"config_sources\": [\"~/src/myproj/mise.toml\"],\n      \"depends\": [],\n      \"env\": {},\n      \"dir\": null,\n      \"hide\": false,\n      \"raw\": false,\n      \"sources\": [],\n      \"outputs\": [],\n      \"run\": [\n        \"echo \\\"testing!\\\"\"\n      ],\n      \"file\": null,\n      \"usage_spec\": {}\n    }\n"
)]
pub struct TasksInfoArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Name of the task to get information about
    #[arg(positional, value_name = "TASK")]
    pub task: ::std::string::String,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tasks ls\u{1b}[22m\n"
)]
pub struct TasksLsArgs {
    /// Only show global tasks
    #[arg(long = "global", short = 'g', overrides("--local"))]
    pub global: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Only show non-global tasks
    #[arg(long = "local", short = 'l', overrides("--global"))]
    pub local: bool,
    /// Show all columns
    #[arg(long = "extended", short = 'x')]
    pub extended: bool,
    #[arg(
        help = "Load all tasks from the entire monorepo, including sibling directories.\nBy default, only tasks from the current directory hierarchy are loaded.",
        long = "all"
    )]
    pub all: bool,
    /// Display tasks for usage completion
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// Show hidden tasks
    #[arg(long = "hidden")]
    pub hidden: bool,
    /// Only show task names, one per line. Useful for piping to fzf and similar tools.
    #[arg(long = "name-only", conflicts("--json", "--extended", "--usage"))]
    pub name_only: bool,
    /// Do not print table header
    #[arg(long = "no-header", alias = "no-headers")]
    pub no_header: bool,
    /// Sort by column. Default is name.
    #[arg(long = "sort", value_name = "COLUMN")]
    pub sort: ::std::option::Option<TasksLsSortValue>,
    /// Sort order. Default is asc.
    #[arg(long = "sort-order", value_name = "SORT_ORDER")]
    pub sort_order: ::std::option::Option<TasksLsSortOrderValue>,
    #[arg(long = "usage", hide)]
    pub usage: bool,
}

#[derive(ValueEnum)]
pub enum TasksLsSortValue {
    #[arg(name = "name")]
    Name,
    #[arg(name = "alias")]
    Alias,
    #[arg(name = "description")]
    Description,
    #[arg(name = "source")]
    Source,
}

#[derive(ValueEnum)]
pub enum TasksLsSortOrderValue {
    #[arg(name = "asc")]
    Asc,
    #[arg(name = "desc")]
    Desc,
}

/// Run task(s)
///
/// This command will run a task, or multiple tasks in parallel. Tasks may have dependencies on other tasks or on source files. If source is configured on a task, it will only run if the source files have changed.
///
/// Tasks can be defined in mise.toml or as standalone scripts. In mise.toml, tasks take this form:
///
///     [tasks.build]
///     run = "npm run build"
///     sources = ["src/**/*.ts"]
///     outputs = ["dist/**/*.js"]
///
/// Alternatively, tasks can be defined as standalone scripts. These must be located in `mise-tasks`, `.mise-tasks`, `.mise/tasks`, `mise/tasks` or `.config/mise/tasks`. The name of the script will be the name of the tasks.
///
///     $ cat .mise/tasks/build<<EOF
///     #!/usr/bin/env bash
///     npm run build
///     EOF
///     $ mise run build
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Runs the \"lint\" tasks. This needs to either be defined in mise.toml\n    # or as a standalone script. See the project README for more information.\n    $ \u{1b}[1mmise run lint\u{1b}[22m\n\n    # Forces the \"build\" tasks to run even if its sources are up-to-date.\n    $ \u{1b}[1mmise run --force build\u{1b}[22m\n\n    # Run \"test\" with stdin/stdout/stderr all connected to the current terminal.\n    # This forces `--jobs=1` to prevent interleaving of output.\n    $ \u{1b}[1mmise run --raw test\u{1b}[22m\n\n    # Runs the \"lint\", \"test\", and \"check\" tasks in parallel.\n    $ \u{1b}[1mmise run lint ::: test ::: check\u{1b}[22m\n\n    # Execute multiple tasks each with their own arguments.\n    $ \u{1b}[1mmise run cmd1 arg1 arg2 ::: cmd2 arg1 arg2\u{1b}[22m\n",
    restart_token = ":::",
    disable_help_flag
)]
pub struct TasksRunArgs {
    /// Run matching tasks only for projects affected by Git changes
    #[arg(long = "affected")]
    pub affected: bool,
    #[arg(
        help = "Git base revision for --affected\nDefaults to MISE_AFFECTED_BASE, CI metadata, or HEAD~1",
        long = "affected-base",
        requires("--affected"),
        value_name = "REV"
    )]
    pub affected_base: ::std::option::Option<::std::string::String>,
    /// Explain why projects and tasks were selected by --affected
    #[arg(
        long = "affected-explain",
        conflicts("--affected-json"),
        requires("--affected")
    )]
    pub affected_explain: bool,
    #[arg(
        help = "Git head revision for --affected\nDefaults to MISE_AFFECTED_HEAD, CI metadata, or HEAD",
        long = "affected-head",
        requires("--affected"),
        value_name = "REV"
    )]
    pub affected_head: ::std::option::Option<::std::string::String>,
    /// Output affected projects and tasks as JSON without running tasks
    #[arg(
        long = "affected-json",
        conflicts("--affected-explain"),
        requires("--affected")
    )]
    pub affected_json: bool,
    /// Open the interactive selector with all tasks from the entire monorepo
    #[arg(long = "all", conflicts("TASK", "--affected"))]
    pub all: bool,
    /// Continue running tasks even if one fails
    #[arg(long = "continue-on-error", short = 'c')]
    pub continue_on_error: bool,
    /// Change to this directory before executing the command
    #[arg(long = "cd", short = 'C', value_name = "CD")]
    pub cd: ::std::option::Option<::std::string::String>,
    /// Force the tasks to run even if outputs are up to date
    #[arg(long = "force", short = 'f')]
    pub force: bool,
    #[arg(
        help = "Number of tasks to run in parallel\nValues below 1 are treated as 1\n[default: 4]\nConfigure with `jobs` config or `MISE_JOBS` env var",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Don't actually run the task(s), just print them in order of execution
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Change how tasks information is output when running tasks
    ///
    /// - `prefix` - Print stdout/stderr by line, prefixed with the task's label - `interleave` - Print directly to stdout/stderr instead of by line - `replacing` - Stdout is replaced each time, stderr is printed as is - `timed` - Only show stdout lines if they are displayed for more than 1 second - `keep-order` - Print stdout/stderr by line, prefixed with the task's label, but keep the order of the output - `quiet` - Don't show extra output - `silent` - Don't show any output including stdout and stderr from the task except for errors
    #[arg(
        long = "output",
        short = 'o',
        env = "MISE_TASK_OUTPUT",
        value_name = "OUTPUT"
    )]
    pub output: ::std::option::Option<::std::string::String>,
    /// Don't show extra output
    #[arg(long = "quiet", short = 'q', env = "MISE_QUIET")]
    pub quiet: bool,
    #[arg(
        help = "Read/write directly to stdin/stdout/stderr instead of by line\nRedactions are not applied with this option\nConfigure with `raw` config or `MISE_RAW` env var",
        long = "raw",
        short = 'r'
    )]
    pub raw: bool,
    /// Shell to use to run toml tasks
    ///
    /// Defaults to `sh -c -o errexit -o pipefail` on unix, and `cmd /c` on Windows Can also be set with the setting `MISE_UNIX_DEFAULT_INLINE_SHELL_ARGS` or `MISE_WINDOWS_DEFAULT_INLINE_SHELL_ARGS` Or it can be overridden with the `shell` property on a task.
    #[arg(long = "shell", short = 's', value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Don't show any output except for errors
    #[arg(long = "silent", short = 'S', env = "MISE_SILENT")]
    pub silent: bool,
    #[arg(
        help = "Tool(s) to run in addition to what is in mise.toml files e.g.: node@20 python@3.10",
        long_help = "Tool(s) to run in addition to what is in mise.toml files\ne.g.: node@20 python@3.10",
        long = "tool",
        short = 't',
        value_name = "TOOL@VERSION"
    )]
    pub tool: ::std::vec::Vec<::std::string::String>,
    #[arg(
        help = "Allow specific env var through (implies --deny-env for everything else)\nSupports wildcards, e.g. --allow-env='MYAPP_*'",
        long = "allow-env",
        value_name = "VAR"
    )]
    pub allow_env: ::std::vec::Vec<::std::string::String>,
    /// Allow network to specific host (implies --deny-net for everything else)
    #[arg(long = "allow-net", value_name = "HOST")]
    pub allow_net: ::std::vec::Vec<::std::string::String>,
    /// Allow reads from specific path (implies --deny-read for everything else)
    #[arg(long = "allow-read", value_name = "PATH")]
    pub allow_read: ::std::vec::Vec<::std::string::String>,
    /// Allow writes to specific path (implies --deny-write for everything else)
    #[arg(long = "allow-write", value_name = "PATH")]
    pub allow_write: ::std::vec::Vec<::std::string::String>,
    /// Block reads, writes, network, and env vars
    #[arg(long = "deny-all")]
    pub deny_all: bool,
    /// Block env var inheritance (only PATH, HOME, USER, SHELL, TERM, LANG pass through)
    #[arg(long = "deny-env")]
    pub deny_env: bool,
    /// Block all network access
    #[arg(long = "deny-net")]
    pub deny_net: bool,
    /// Block filesystem reads (system libs and tool dirs still accessible)
    #[arg(long = "deny-read")]
    pub deny_read: bool,
    /// Block all filesystem writes
    #[arg(long = "deny-write")]
    pub deny_write: bool,
    /// Bypass the environment cache and recompute the environment
    #[arg(long = "fresh-env")]
    pub fresh_env: bool,
    /// Do not use cache on remote tasks
    #[arg(long = "no-cache", env = "MISE_TASK_REMOTE_NO_CACHE")]
    pub no_cache: bool,
    /// Skip automatic dependency preparation
    #[arg(long = "no-deps")]
    pub no_deps: bool,
    /// Hides elapsed time after each task completes
    ///
    /// Default to always hide with `MISE_TASK_TIMINGS=0`
    #[arg(long = "no-timings", alias = "no-timing")]
    pub no_timings: bool,
    /// Run only the specified tasks skipping all dependencies
    #[arg(long = "skip-deps", env = "MISE_TASK_SKIP_DEPENDS")]
    pub skip_deps: bool,
    /// Skip installing tools before running tasks
    ///
    /// Can also be set persistently with the `task.run_auto_install` setting or `MISE_TASK_RUN_AUTO_INSTALL=false` env var
    #[arg(long = "skip-tools")]
    pub skip_tools: bool,
    /// Set task output cache access for this run
    ///
    /// - `read-write` - Read cached results and write new results - `read-only` - Read cached results without writing new results - `write-only` - Write new results without reading cached results - `off` - Disable task output caching - `local-only` - Read and write only the local cache; currently equivalent to `read-write`
    #[arg(
        long = "task-cache",
        env = "MISE_TASK_CACHE",
        value_name = "TASK_CACHE",
        default = "read-write"
    )]
    pub task_cache: ::std::option::Option<TasksRunTaskCacheValue>,
    /// Explain the inputs that produced each task's output cache key
    #[arg(long = "task-cache-explain")]
    pub task_cache_explain: bool,
    /// Output cache-key input details as JSON Lines without running tasks
    #[arg(
        long = "task-cache-explain-json",
        conflicts("--task-cache-explain"),
        requires("--dry-run")
    )]
    pub task_cache_explain_json: bool,
    /// Report task output cache hits, restored bytes, and time saved
    #[arg(long = "task-cache-stats", conflicts("--dry-run"))]
    pub task_cache_stats: bool,
    #[arg(
        help = "Timeout for the task to complete\ne.g.: 30s, 5m",
        long = "timeout",
        value_name = "TIMEOUT"
    )]
    pub timeout: ::std::option::Option<::std::string::String>,
    /// Shows elapsed time after each task completes
    ///
    /// Default to always show with `MISE_TASK_TIMINGS=1`
    #[arg(long = "timings", alias = "timing", hide)]
    pub timings: bool,
    #[arg(
        positional,
        value_name = "TASK",
        help = "Tasks to run\nCan specify multiple tasks by separating with `:::`\ne.g.: mise run task1 arg1 arg2 ::: task2 arg1 arg2\nDefaults to `default` when omitted",
        double_dash = "automatic"
    )]
    pub task: ::std::option::Option<::std::string::String>,
    /// Arguments to pass to the tasks. Use ":::" to separate tasks.
    #[arg(positional, value_name = "ARGS")]
    pub args: ::std::vec::Vec<::std::string::String>,
    /// Arguments to pass to the tasks. Use ":::" to separate tasks.
    #[arg(positional, value_name = "ARGS_LAST", hide, double_dash = "required")]
    pub args_last: ::std::vec::Vec<::std::string::String>,
}

#[derive(ValueEnum)]
pub enum TasksRunTaskCacheValue {
    /// Read cached results and write new results.
    #[arg(name = "read-write")]
    ReadWrite,
    /// Read cached results without writing new results.
    #[arg(name = "read-only")]
    ReadOnly,
    /// Write new results without reading cached results.
    #[arg(name = "write-only")]
    WriteOnly,
    /// Disable task output caching for this run.
    #[arg(name = "off")]
    Off,
    /// Read and write only the local cache.
    #[arg(name = "local-only")]
    LocalOnly,
}

/// Validate tasks for common errors and issues
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Validate all tasks\n    $ \u{1b}[1mmise tasks validate\u{1b}[22m\n\n    # Validate specific tasks\n    $ \u{1b}[1mmise tasks validate build test\u{1b}[22m\n\n    # Output results as JSON\n    $ \u{1b}[1mmise tasks validate --json\u{1b}[22m\n\n    # Only show errors (skip warnings)\n    $ \u{1b}[1mmise tasks validate --errors-only\u{1b}[22m\n\n\u{1b}[1m\u{1b}[4mValidation Checks:\u{1b}[22m\u{1b}[24m\n\nThe validate command performs the following checks:\n\n  • \u{1b}[1mCircular Dependencies\u{1b}[22m: Detects dependency cycles\n  • \u{1b}[1mMissing References\u{1b}[22m: Finds references to nonexistent tasks\n  • \u{1b}[1mUsage Spec Parsing\u{1b}[22m: Validates #USAGE directives and specs\n  • \u{1b}[1mTimeout Format\u{1b}[22m: Checks timeout values are valid durations\n  • \u{1b}[1mAlias Conflicts\u{1b}[22m: Detects duplicate aliases across tasks\n  • \u{1b}[1mFile Existence\u{1b}[22m: Verifies file-based tasks exist\n  • \u{1b}[1mDirectory Templates\u{1b}[22m: Validates directory paths and templates\n  • \u{1b}[1mShell Commands\u{1b}[22m: Checks shell executables exist\n  • \u{1b}[1mGlob Patterns\u{1b}[22m: Validates source and output patterns\n  • \u{1b}[1mRun Entries\u{1b}[22m: Ensures tasks reference valid dependencies\n"
)]
pub struct TasksValidateArgs {
    /// Only show errors (skip warnings)
    #[arg(long = "errors-only")]
    pub errors_only: bool,
    /// Output validation results in JSON format
    #[arg(long = "json")]
    pub json: bool,
    #[arg(
        positional,
        value_name = "TASKS",
        help = "Tasks to validate\nIf not specified, validates all tasks"
    )]
    pub tasks: ::std::vec::Vec<::std::string::String>,
}

/// Manage tasks
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct TasksArgs {
    /// Only show global tasks
    #[arg(long = "global", short = 'g', overrides("--local"))]
    pub global: bool,
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Only show non-global tasks
    #[arg(long = "local", short = 'l', overrides("--global"))]
    pub local: bool,
    /// Show all columns
    #[arg(long = "extended", short = 'x')]
    pub extended: bool,
    #[arg(
        help = "Load all tasks from the entire monorepo, including sibling directories.\nBy default, only tasks from the current directory hierarchy are loaded.",
        long = "all"
    )]
    pub all: bool,
    /// Display tasks for usage completion
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// Show hidden tasks
    #[arg(long = "hidden")]
    pub hidden: bool,
    /// Only show task names, one per line. Useful for piping to fzf and similar tools.
    #[arg(long = "name-only", conflicts("--json", "--extended", "--usage"))]
    pub name_only: bool,
    /// Do not print table header
    #[arg(long = "no-header", alias = "no-headers")]
    pub no_header: bool,
    /// Sort by column. Default is name.
    #[arg(long = "sort", value_name = "COLUMN")]
    pub sort: ::std::option::Option<TasksSortValue>,
    /// Sort order. Default is asc.
    #[arg(long = "sort-order", value_name = "SORT_ORDER")]
    pub sort_order: ::std::option::Option<TasksSortOrderValue>,
    #[arg(long = "usage", hide)]
    pub usage: bool,
    /// Task name to get info of
    #[arg(positional, value_name = "TASK")]
    pub task: ::std::option::Option<::std::string::String>,
    #[arg(subcommand)]
    pub command: ::std::option::Option<TasksCommands>,
}

#[derive(ValueEnum)]
pub enum TasksSortValue {
    #[arg(name = "name")]
    Name,
    #[arg(name = "alias")]
    Alias,
    #[arg(name = "description")]
    Description,
    #[arg(name = "source")]
    Source,
}

#[derive(ValueEnum)]
pub enum TasksSortOrderValue {
    #[arg(name = "asc")]
    Asc,
    #[arg(name = "desc")]
    Desc,
}

#[derive(Subcommand)]
pub enum TasksCommands {
    /// Create a new task
    #[arg(name = "add")]
    Add(Box<TasksAddArgs>),
    /// Display a tree visualization of a dependency graph
    #[arg(name = "deps")]
    Deps(Box<TasksDepsArgs>),
    /// Edit a task with $EDITOR
    #[arg(name = "edit")]
    Edit(Box<TasksEditArgs>),
    /// [experimental] Inspect the workspace project graph
    #[arg(name = "graph")]
    Graph(Box<TasksGraphArgs>),
    /// Get information about a task
    #[arg(name = "info")]
    Info(Box<TasksInfoArgs>),
    #[arg(
        name = "ls",
        help = "List available tasks to execute\nThese may be included from the config file or from the project's .mise/tasks directory\nmise will merge all tasks from all parent directories into this list.",
        long_help = "List available tasks to execute\nThese may be included from the config file or from the project's .mise/tasks directory\nmise will merge all tasks from all parent directories into this list.\n\nSo if you have global tasks in `~/.config/mise/tasks/*` and project-specific tasks in\n~/myproject/.mise/tasks/*, then they'll both be available but the project-specific\ntasks will override the global ones if they have the same name."
    )]
    Ls(Box<TasksLsArgs>),
    /// Run task(s)
    #[arg(name = "run", alias = "r")]
    Run(Box<TasksRunArgs>),
    /// Validate tasks for common errors and issues
    #[arg(name = "validate")]
    Validate(Box<TasksValidateArgs>),
}

/// Test a tool installs and executes
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise test-tool ripgrep\u{1b}[22m\n"
)]
pub struct TestToolArgs {
    /// Test every tool specified in registry/
    #[arg(long = "all", short = 'a', conflicts("TOOLS", "--all-config"))]
    pub all: bool,
    #[arg(
        help = "Number of tool tests to run in parallel\nValues below 1 are treated as 1\n[default: 4]",
        long = "jobs",
        short = 'j',
        env = "MISE_TEST_TOOL_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Test all tools specified in config files
    #[arg(long = "all-config", conflicts("TOOLS", "--all"))]
    pub all_config: bool,
    /// Also test tools not defined in registry/, guessing how to test it
    #[arg(long = "include-non-defined")]
    pub include_non_defined: bool,
    #[arg(
        help = "Connect backend install command stdin/stdout/stderr directly to the terminal Implies --jobs=1",
        long_help = "Connect backend install command stdin/stdout/stderr directly to the terminal\nImplies --jobs=1",
        long = "raw",
        overrides("--jobs")
    )]
    pub raw: bool,
    /// Tool(s) to test
    #[arg(
        positional,
        value_name = "TOOLS",
        required_unless("--all", "--all-config")
    )]
    pub tools: ::std::vec::Vec<::std::string::String>,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise token forgejo\u{1b}[22m\n    codeberg.org: a180…61f6 (source: FORGEJO_TOKEN)\n\n    $ \u{1b}[1mmise token forgejo --unmask\u{1b}[22m\n    codeberg.org: a18099ca69064be387fbe37b8ad1d333758361f6 (source: FORGEJO_TOKEN)\n\n    $ \u{1b}[1mmise token forgejo forgejo.mycompany.com\u{1b}[22m\n    forgejo.mycompany.com: (none)\n"
)]
pub struct TokenForgejoArgs {
    /// Show the full unmasked token
    #[arg(long = "unmask")]
    pub unmask: bool,
    /// Forgejo hostname
    #[arg(positional, value_name = "HOST", default = "codeberg.org")]
    pub host: ::std::option::Option<::std::string::String>,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise token github\u{1b}[22m\n    github.com: ghp_…xxxx (source: GITHUB_TOKEN)\n\n    $ \u{1b}[1mmise token github --unmask\u{1b}[22m\n    github.com: ghp_xxxxxxxxxxxx (source: GITHUB_TOKEN)\n\n    $ \u{1b}[1mmise token github github.mycompany.com\u{1b}[22m\n    github.mycompany.com: (none)\n\n    $ \u{1b}[1mmise token github --oauth --refresh\u{1b}[22m\n    github.com: gho_…xxxx (source: GitHub OAuth)\n"
)]
pub struct TokenGithubArgs {
    #[arg(
        help = "Resolve only via the native GitHub OAuth source (cache, refresh, or device-code flow), bypassing other token sources",
        long_help = "Resolve only via the native GitHub OAuth source (cache,\nrefresh, or device-code flow), bypassing other token sources",
        long = "oauth"
    )]
    pub oauth: bool,
    /// Print only the token value
    #[arg(long = "raw")]
    pub raw: bool,
    #[arg(
        help = "Mint a fresh OAuth token even if the cached one has not expired, via the refresh-token grant or a new device-code flow. Use after changing the GitHub App's installations or permissions: cached tokens keep their original access until they expire",
        long_help = "Mint a fresh OAuth token even if the cached one has not\nexpired, via the refresh-token grant or a new device-code flow.\nUse after changing the GitHub App's installations or permissions:\ncached tokens keep their original access until they expire",
        long = "refresh",
        requires("--oauth")
    )]
    pub refresh: bool,
    /// Show the full unmasked token
    #[arg(long = "unmask")]
    pub unmask: bool,
    /// GitHub hostname
    #[arg(positional, value_name = "HOST", default = "github.com")]
    pub host: ::std::option::Option<::std::string::String>,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise token gitlab\u{1b}[22m\n    gitlab.com: glpa…xxxx (source: GITLAB_TOKEN)\n\n    $ \u{1b}[1mmise token gitlab --unmask\u{1b}[22m\n    gitlab.com: glpat-xxxxxxxxxxxx (source: GITLAB_TOKEN)\n\n    $ \u{1b}[1mmise token gitlab gitlab.mycompany.com\u{1b}[22m\n    gitlab.mycompany.com: (none)\n"
)]
pub struct TokenGitlabArgs {
    /// Show the full unmasked token
    #[arg(long = "unmask")]
    pub unmask: bool,
    /// GitLab hostname
    #[arg(positional, value_name = "HOST", default = "gitlab.com")]
    pub host: ::std::option::Option<::std::string::String>,
}

/// Display git provider tokens mise will use
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct TokenArgs {
    #[arg(subcommand)]
    pub command: TokenCommands,
}

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Forgejo token
    #[arg(
        name = "forgejo",
        help = "Forgejo token",
        long_help = "Display the Forgejo token mise will use for a given host\n\nShows which token source mise would use, useful for debugging\nauthentication issues. The token is masked by default."
    )]
    Forgejo(Box<TokenForgejoArgs>),
    /// GitHub token
    #[arg(
        name = "github",
        help = "GitHub token",
        long_help = "Display the GitHub token mise will use for a given host\n\nShows which token source mise would use, useful for debugging\nauthentication issues. The token is masked by default."
    )]
    Github(Box<TokenGithubArgs>),
    /// GitLab token
    #[arg(
        name = "gitlab",
        help = "GitLab token",
        long_help = "Display the GitLab token mise will use for a given host\n\nShows which token source mise would use, useful for debugging\nauthentication issues. The token is masked by default."
    )]
    Gitlab(Box<TokenGitlabArgs>),
}

/// Gets information about a tool
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise tool node\u{1b}[22m\n    Backend:            core\n    Installed Versions: 20.0.0 22.0.0\n    Active Version:     20.0.0\n    Requested Version:  20\n    Config Source:      ~/.config/mise/mise.toml\n    Tool Options:       [none]\n",
    group("ToolInfoFilter")
)]
pub struct ToolArgs {
    /// Output in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
    /// Only show active versions
    #[arg(long = "active", group = "ToolInfoFilter")]
    pub active: bool,
    /// Only show backend field
    #[arg(long = "backend", group = "ToolInfoFilter")]
    pub backend: bool,
    /// Only show config source
    #[arg(long = "config-source", group = "ToolInfoFilter")]
    pub config_source: bool,
    /// Only show description field
    #[arg(long = "description", group = "ToolInfoFilter")]
    pub description: bool,
    /// Only show installed versions
    #[arg(long = "installed", group = "ToolInfoFilter")]
    pub installed: bool,
    /// Only show requested versions
    #[arg(long = "requested", group = "ToolInfoFilter")]
    pub requested: bool,
    /// Only show tool options
    #[arg(long = "tool-options", group = "ToolInfoFilter")]
    pub tool_options: bool,
    /// Tool name to get information about
    #[arg(positional, value_name = "TOOL")]
    pub tool: ::std::string::String,
}

/// Execute a tool stub
///
/// Tool stubs are executable files containing TOML configuration that specify which tool to run and how to run it. They provide a convenient way to create portable, self-contained executables that automatically manage tool installation and execution.
///
/// A tool stub consists of: - A shebang line: #!/usr/bin/env -S mise tool-stub - TOML configuration specifying the tool, version, and options - Optional comments describing the tool's purpose
///
/// Example stub file:
///   #!/usr/bin/env -S mise tool-stub
///   # Node.js v20 development environment
///
///   tool = "node"
///   version = "20.0.0"
///   bin = "node"
///
/// The stub will automatically install the specified tool version if missing and execute it with any arguments passed to the stub.
///
/// For more information, see: https://mise.jdx.dev/dev-tools/tool-stubs.html
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(disable_help_flag, disable_version_flag)]
pub struct ToolStubArgs {
    /// Path to the TOML tool stub file to execute
    ///
    /// The stub file must contain TOML configuration specifying the tool and version to run. At minimum, it should specify a 'version' field. Other common fields include 'tool', 'bin', and backend-specific options.
    #[arg(positional, value_name = "FILE")]
    pub file: ::std::string::String,
    /// Arguments to pass to the tool
    ///
    /// All arguments after the stub file path will be forwarded to the underlying tool. Use '--' to separate mise arguments from tool arguments if needed.
    #[arg(positional, value_name = "ARGS", double_dash = "automatic")]
    pub args: ::std::vec::Vec<::std::string::String>,
}

/// Marks a config file as trusted
///
/// This means mise is allowed to parse the file when it needs to read config that may execute code or affect the environment. Without trust, mise may prompt, skip the config in some discovery paths, or fail with an untrusted-config error when it cannot prompt.
///
/// In normal mode, commands that execute project-defined behavior (`mise run`, naked task invocations such as `mise <TASK>`, `mise install`, `mise exec`, and `mise watch`) automatically trust their active config. Paranoid mode requires explicit, content-bound trust for every non-global config.
///
/// In normal mode, safe config files do not require trust: files that only contain `min_version`, `[tools]` entries with plain version strings (or arrays of them), and `[tasks]` without templates or tool options.
///
/// Trust is shared across git worktrees: a config file inside a linked worktree is trusted when the equivalent path in the repository's main checkout has been trusted. Paranoid mode disables this sharing since worktrees can check out branches with different config contents.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # trusts ~/some_dir/mise.toml\n    $ \u{1b}[1mmise trust ~/some_dir/mise.toml\u{1b}[22m\n\n    # trusts mise.toml in the current or parent directory\n    $ \u{1b}[1mmise trust\u{1b}[22m\n"
)]
pub struct TrustArgs {
    /// Trust all config files in the current directory, its parents, and its subdirectories
    ///
    /// Subdirectories are walked respecting .gitignore, skipping hidden directories and common build/dependency directories (node_modules, vendor, target, dist, build).
    #[arg(long = "all", short = 'a', conflicts("--ignore", "--untrust"))]
    pub all: bool,
    /// Do not trust this config and ignore it in the future
    #[arg(long = "ignore", conflicts("--untrust"))]
    pub ignore: bool,
    #[arg(
        help = "Show the trusted status of config files from the current directory and its parents.\nDoes not trust or untrust any files.",
        long = "show"
    )]
    pub show: bool,
    /// Remove explicit trust for this config
    #[arg(long = "untrust")]
    pub untrust: bool,
    /// The config file whose trust status to change
    #[arg(positional, value_name = "CONFIG_FILE")]
    pub config_file: ::std::option::Option<::std::string::String>,
}

/// Removes installed tool versions
///
/// This only removes the installed version, it does not modify mise.toml.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # will uninstall specific version\n    $ \u{1b}[1mmise uninstall node@18.0.0\u{1b}[22m\n\n    # will uninstall the current node version (if only one version is installed)\n    $ \u{1b}[1mmise uninstall node\u{1b}[22m\n\n    # will uninstall all installed versions of node\n    $ \u{1b}[1mmise uninstall --all node@18.0.0\u{1b}[22m # will uninstall all node versions\n"
)]
pub struct UninstallArgs {
    /// Delete all installed versions
    #[arg(long = "all", short = 'a')]
    pub all: bool,
    /// Do not actually delete anything
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Like --dry-run but exits with code 1 if there are tools to uninstall
    ///
    /// This is useful for scripts to check if tools need to be uninstalled.
    #[arg(long = "dry-run-code")]
    pub dry_run_code: bool,
    /// Tool(s) to remove
    #[arg(
        positional,
        value_name = "INSTALLED_TOOL@VERSION",
        required_unless("--all")
    )]
    pub installed_tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Remove environment variable(s) from the config file.
///
/// By default, this command modifies `mise.toml` in the current directory.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Remove NODE_ENV from the current directory's config\n    $ \u{1b}[1mmise unset NODE_ENV\u{1b}[22m\n\n    # Remove NODE_ENV from the global config\n    $ \u{1b}[1mmise unset NODE_ENV -g\u{1b}[22m\n"
)]
pub struct UnsetArgs {
    /// Specify a file to use instead of `mise.toml`
    ///
    /// Can be a file path or directory. If a directory is provided, will create/use mise.toml in that directory.
    ///
    /// Defaults to [`MISE_DEFAULT_CONFIG_FILENAME`](https://mise.jdx.dev/configuration.html#mise_default_config_filename) environment variable, or `mise.toml`. Use [`MISE_GLOBAL_CONFIG_FILE`](https://mise.jdx.dev/configuration.html#mise_global_config_file) to choose a different global config path.
    #[arg(long = "file", long = "path", short = 'f', value_name = "FILE")]
    pub file: ::std::option::Option<::std::string::String>,
    /// Use the global config file
    #[arg(long = "global", short = 'g', overrides("--file"))]
    pub global: bool,
    #[arg(
        positional,
        value_name = "ENV_KEY",
        help = "Environment variable(s) to remove\ne.g.: NODE_ENV"
    )]
    pub env_key: ::std::vec::Vec<::std::string::String>,
}

/// Remove explicit trust for a config
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct UntrustArgs {
    /// The config file to untrust
    #[arg(positional, value_name = "CONFIG_FILE")]
    pub config_file: ::std::option::Option<::std::string::String>,
}

/// Removes installed tool versions from mise.toml
///
/// By default, this will use the `mise.toml` file that has the tool defined. If multiple config files exist (e.g., both `mise.toml` and `mise.local.toml`), the lowest precedence file (`mise.toml`) will be used. See https://mise.jdx.dev/configuration.html#target-file-for-write-operations
///
/// In the following order:
///   - If `--global` is set, it will use the global config file.
///   - If `--path` is set, it will use the config file at the given path.
///   - If `--env` is set, it will use `mise.<env>.toml`.
///   - If [`MISE_DEFAULT_CONFIG_FILENAME`](https://mise.jdx.dev/configuration.html#mise_default_config_filename) is set, it will use that instead.
///   - If `MISE_OVERRIDE_CONFIG_FILENAMES` is set, it will the first from that list.
///   - Otherwise just "mise.toml" or global config if cwd is home directory.
///
/// Use [`MISE_GLOBAL_CONFIG_FILE`](https://mise.jdx.dev/configuration.html#mise_global_config_file) to choose a different global config path.
///
/// Will also prune the installed version if no other configurations are using it.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # will uninstall specific version\n    $ \u{1b}[1mmise unuse node@18.0.0\u{1b}[22m\n\n    # will uninstall specific version from global config\n    $ \u{1b}[1mmise unuse -g node@18.0.0\u{1b}[22m\n\n    # will uninstall specific version from .mise.local.toml\n    $ \u{1b}[1mmise unuse --env local node@20\u{1b}[22m\n\n    # will uninstall specific version from .mise.staging.toml\n    $ \u{1b}[1mmise unuse --env staging node@20\u{1b}[22m\n"
)]
pub struct UnuseArgs {
    /// Create/modify an environment-specific config file like .mise.<env>.toml
    #[arg(
        long = "env",
        short = 'e',
        overrides("--global", "--path"),
        value_name = "ENV"
    )]
    pub env: ::std::option::Option<::std::string::String>,
    /// Use the global config file (`~/.config/mise/config.toml`) instead of the local one
    #[arg(long = "global", short = 'g', overrides("--path", "--env"))]
    pub global: bool,
    /// Specify a path to a config file or directory
    ///
    /// If a directory is specified, it will look for a config file in that directory following the rules above.
    #[arg(
        long = "path",
        long = "file",
        short = 'p',
        overrides("--global", "--env"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Do not also prune the installed version
    #[arg(long = "no-prune")]
    pub no_prune: bool,
    /// Tool(s) to remove
    #[arg(positional, value_name = "INSTALLED_TOOL@VERSION", required)]
    pub installed_tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Upgrades outdated tools
///
/// By default, this keeps the range specified in mise.toml. So if you have node@20 set, it will upgrade to the latest 20.x.x version available. See the `--bump` flag to use the latest version and bump the version in mise.toml.
///
/// This will update mise.lock if it is enabled, see https://mise.jdx.dev/configuration/settings.html#lockfile
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mDeprecation:\u{1b}[22m\u{1b}[24m\n\nThe `-l` shorthand for `--bump` is deprecated and will be removed in mise 2027.8.5.\nAfter removal, `-l` will become shorthand for `--local`. Use `-b` or `--bump` instead.\n\n\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Upgrades node to the latest version matching the range in mise.toml\n    $ \u{1b}[1mmise upgrade node\u{1b}[22m\n\n    # Upgrades node to the latest version and bumps the version in mise.toml\n    $ \u{1b}[1mmise upgrade node --bump\u{1b}[22m\n\n    # Upgrades all tools to the latest versions\n    $ \u{1b}[1mmise upgrade\u{1b}[22m\n\n    # Upgrades all tools to the latest versions and bumps the version in mise.toml\n    $ \u{1b}[1mmise upgrade --bump\u{1b}[22m\n\n    # Just print what would be done, don't actually do it\n    $ \u{1b}[1mmise upgrade --dry-run\u{1b}[22m\n\n    # Upgrades node and python to the latest versions\n    $ \u{1b}[1mmise upgrade node python\u{1b}[22m\n\n    # Upgrade all tools except go\n    $ \u{1b}[1mmise upgrade --exclude go\u{1b}[22m\n\n    # Show a multiselect menu to choose which tools to upgrade\n    $ \u{1b}[1mmise upgrade --interactive\u{1b}[22m\n\n    # Only upgrade tools defined in local mise.toml, not global ones\n    $ \u{1b}[1mmise upgrade --local\u{1b}[22m\n"
)]
pub struct UpgradeArgs {
    /// Upgrades to the latest version available, bumping the version in mise.toml
    ///
    /// For example, if you have `node = "20.0.0"` in your mise.toml but 22.1.0 is the latest available, this will install 22.1.0 and set `node = "22.1.0"` in your config.
    ///
    /// It keeps the same precision as what was there before, so if you instead had `node = "20"`, it would change your config to `node = "22"`.
    #[arg(long = "bump", short = 'b')]
    pub bump: bool,
    /// Display multiselect menu to choose which tools to upgrade
    #[arg(long = "interactive", short = 'i', conflicts("INSTALLED_TOOL@VERSION"))]
    pub interactive: bool,
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1\n[default: 4]",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Deprecated shorthand for --bump
    #[arg(short = 'l', hide)]
    pub l: bool,
    /// Just print what would be done, don't actually do it
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    #[arg(
        help = "Tool(s) to exclude from upgrading\ne.g.: go python",
        long = "exclude",
        short = 'x',
        value_name = "INSTALLED_TOOL"
    )]
    pub exclude: ::std::vec::Vec<::std::string::String>,
    /// Like --dry-run but exits with code 1 if there are outdated tools
    ///
    /// This is useful for scripts to check if tools need to be upgraded.
    #[arg(long = "dry-run-code")]
    pub dry_run_code: bool,
    /// Upgrade all tools, including installed-but-inactive tools not present in the current config
    #[arg(long = "inactive", conflicts("--local"))]
    pub inactive: bool,
    /// Only upgrade tools defined in local config files
    ///
    /// This will only upgrade tools that are defined in project-local mise.toml and will skip tools defined in the global config (~/.config/mise/config.toml).
    #[arg(long = "local")]
    pub local: bool,
    /// Only upgrade to versions released before this date or older than this duration
    ///
    /// Supports absolute dates like "2024-06-01" and relative durations like "90d" or "1y". This can be useful for reproducibility or security purposes.
    ///
    /// This only affects fuzzy version matches like "20" or "latest". Explicitly pinned versions like "22.5.0" are not filtered.
    #[arg(
        long = "minimum-release-age",
        alias = "before",
        value_name = "MINIMUM_RELEASE_AGE"
    )]
    pub minimum_release_age: ::std::option::Option<::std::string::String>,
    /// Placeholder for future monorepo upgrades; `mise upgrade --monorepo` is not implemented yet.
    #[arg(long = "monorepo")]
    pub monorepo: bool,
    /// Do not uninstall the versions that were upgraded away from
    ///
    /// By default the old version is removed once the new one installs, unless another tracked config or tool stub still needs it. Use this to keep it anyway, e.g. when something outside of mise points at the old install directory.
    ///
    /// Set `upgrade.auto_prune = false` to make this the default.
    #[arg(long = "no-prune", overrides("--prune"))]
    pub no_prune: bool,
    /// Uninstall the versions that were upgraded away from
    ///
    /// This is already the default. Use it to override `upgrade.auto_prune = false` for a single run.
    #[arg(long = "prune", overrides("--no-prune"))]
    pub prune: bool,
    #[arg(
        help = "Connect backend install command stdin/stdout/stderr directly to the terminal Implies --jobs=1",
        long_help = "Connect backend install command stdin/stdout/stderr directly to the terminal\nImplies --jobs=1",
        long = "raw",
        overrides("--jobs")
    )]
    pub raw: bool,
    #[arg(
        positional,
        value_name = "INSTALLED_TOOL@VERSION",
        help = "Tool(s) to upgrade\ne.g.: node@20 python@3.10\nIf not specified, all current tools will be upgraded"
    )]
    pub installed_tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Generate a usage CLI spec
///
/// See https://usage.jdx.dev for more information on this specification.
#[derive(Args)]
#[arg(unknown_flags = "value")]
pub struct UsageArgs {}

/// Installs a tool and adds the version to mise.toml.
///
/// This will install the tool version if it is not already installed. By default, this will use a `mise.toml` file in the current directory. If multiple config files exist (e.g., both `mise.toml` and `mise.local.toml`), the lowest precedence file (`mise.toml`) will be used. See https://mise.jdx.dev/configuration.html#target-file-for-write-operations
///
/// In the following order:
///   - If `--global` is set, it will use the global config file.
///   - If `--path` is set, it will use the config file at the given path.
///   - If `--env` is set, it will use `mise.<env>.toml`.
///   - If [`MISE_DEFAULT_CONFIG_FILENAME`](https://mise.jdx.dev/configuration.html#mise_default_config_filename) is set, it will use that instead.
///   - If `MISE_OVERRIDE_CONFIG_FILENAMES` is set, it will the first from that list.
///   - Otherwise just "mise.toml" or global config if cwd is home directory.
///
/// Use [`MISE_GLOBAL_CONFIG_FILE`](https://mise.jdx.dev/configuration.html#mise_global_config_file) to choose a different global config path.
///
/// Use the `--global` flag to use the global config file instead.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # run with no arguments to use the interactive selector\n    $ \u{1b}[1mmise use\u{1b}[22m\n\n    # set the current version of node to 20.x in mise.toml of current directory\n    # will write the fuzzy version (e.g.: 20)\n    $ \u{1b}[1mmise use node@20\u{1b}[22m\n\n    # set the current version of node to 20.x in ~/.config/mise/config.toml\n    # will write the precise version (e.g.: 20.0.0)\n    $ \u{1b}[1mmise use -g --pin node@20\u{1b}[22m\n\n    # sets .mise.local.toml (which is intended not to be committed to a project)\n    $ \u{1b}[1mmise use --env local node@20\u{1b}[22m\n\n    # sets .mise.staging.toml (which is used if MISE_ENV=staging)\n    $ \u{1b}[1mmise use --env staging node@20\u{1b}[22m\n"
)]
pub struct UseArgs {
    /// Create/modify an environment-specific config file like .mise.<env>.toml
    #[arg(
        long = "env",
        short = 'e',
        overrides("--global", "--path"),
        value_name = "ENV"
    )]
    pub env: ::std::option::Option<::std::string::String>,
    /// Force reinstall even if already installed
    #[arg(long = "force", short = 'f', requires("TOOL@VERSION"))]
    pub force: bool,
    /// Use the global config file (`~/.config/mise/config.toml`) instead of the local one
    #[arg(long = "global", short = 'g', overrides("--path", "--env"))]
    pub global: bool,
    #[arg(
        help = "Number of jobs to run in parallel\nValues below 1 are treated as 1\n[default: 4]",
        long = "jobs",
        short = 'j',
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Perform a dry run, showing what would be installed and modified without making changes
    #[arg(long = "dry-run", short = 'n')]
    pub dry_run: bool,
    /// Specify a path to a config file or directory
    ///
    /// If a directory is specified, it will look for a config file in that directory following the rules above.
    #[arg(
        long = "path",
        short = 'p',
        overrides("--global", "--env"),
        value_name = "PATH"
    )]
    pub path: ::std::option::Option<::std::string::String>,
    /// Like --dry-run but exits with code 1 if there are changes to make
    ///
    /// This is useful for scripts to check if tools need to be added or removed.
    #[arg(long = "dry-run-code")]
    pub dry_run_code: bool,
    /// Save fuzzy version to config file
    ///
    /// e.g.: `mise use --fuzzy node@20` will save 20 as the version this is the default behavior unless `MISE_PIN=1`
    #[arg(long = "fuzzy", overrides("--pin"))]
    pub fuzzy: bool,
    /// Only install versions released before this date or older than this duration
    ///
    /// Supports absolute dates like "2024-06-01" and relative durations like "90d" or "1y".
    #[arg(
        long = "minimum-release-age",
        alias = "before",
        value_name = "MINIMUM_RELEASE_AGE"
    )]
    pub minimum_release_age: ::std::option::Option<::std::string::String>,
    /// Save the resolved concrete version to the config file
    ///
    /// If the request exactly matches an available release, that release is preferred over installed fuzzy matches. Use `prefix:` to explicitly request recursive prefix matching. e.g.: `mise use --pin node@20` will save the resolved `20.x.y` version Set `MISE_PIN=1` to make this the default behavior
    ///
    /// Consider using mise.lock as a better alternative to pinning in mise.toml: https://mise.jdx.dev/configuration/settings.html#lockfile
    #[arg(long = "pin", overrides("--fuzzy"))]
    pub pin: bool,
    #[arg(
        help = "Connect backend install command stdin/stdout/stderr directly to the terminal Implies `--jobs=1`",
        long_help = "Connect backend install command stdin/stdout/stderr directly to the terminal\nImplies `--jobs=1`",
        long = "raw",
        overrides("--jobs")
    )]
    pub raw: bool,
    /// Remove the tool(s) from config file
    #[arg(long = "remove", alias("rm", "unset"), value_name = "TOOL")]
    pub remove: ::std::vec::Vec<::std::string::String>,
    /// Tool(s) to add to config file
    ///
    /// e.g.: node@20, cargo:ripgrep@latest npm:prettier@3 If no version is specified, it will default to @latest
    ///
    /// Tool options can be set with this syntax:
    ///
    ///     mise use ubi:BurntSushi/ripgrep[exe=rg]
    #[arg(positional, value_name = "TOOL@VERSION")]
    pub tool_version: ::std::vec::Vec<::std::string::String>,
}

/// Display the version of mise
///
/// Displays the version, os, architecture, and the date of the build.
///
/// If the version is out of date, it will display a warning.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise version\u{1b}[22m\n    $ \u{1b}[1mmise --version\u{1b}[22m\n    $ \u{1b}[1mmise -v\u{1b}[22m\n    $ \u{1b}[1mmise -V\u{1b}[22m\n"
)]
pub struct VersionArgs {
    /// Print the version information in JSON format
    #[arg(long = "json", short = 'J')]
    pub json: bool,
}

/// Run task(s) and watch for changes to rerun it
///
/// This command uses the `watchexec` tool to watch for changes to files and rerun the specified task(s). It must be installed for this command to work, but you can install it with `mise use -g watchexec@latest`.
///
/// For more advanced process management (daemon management, auto-restart, readiness checks, cron scheduling), see mise's sister project: https://pitchfork.jdx.dev
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise watch build\u{1b}[22m\n    Runs the \"build\" tasks. Will re-run the tasks when any of its sources change.\n    Uses \"sources\" from the tasks definition to determine which files to watch.\n\n    $ \u{1b}[1mmise watch build --glob src/**/*.rs\u{1b}[22m\n    Runs the \"build\" tasks but specify the files to watch with a glob pattern.\n    This overrides the \"sources\" from the tasks definition.\n\n    $ \u{1b}[1mmise watch build --clear\u{1b}[22m\n    Extra arguments are passed to watchexec. See `watchexec --help` for details.\n\n    $ \u{1b}[1mmise watch serve --watch src --exts rs --restart\u{1b}[22m\n    Starts an api server, watching for changes to \"*.rs\" files in \"./src\" and kills/restarts the server when they change.\n"
)]
pub struct WatchArgs {
    /// Tasks to run
    #[arg(long = "task-flag", short = 't', hide, value_name = "TASK_FLAG")]
    pub task_flag: ::std::vec::Vec<::std::string::String>,
    #[arg(
        help = "Files to watch\nDefaults to sources from the task(s)",
        long = "glob",
        short = 'g',
        hide,
        value_name = "GLOB"
    )]
    pub glob: ::std::vec::Vec<::std::string::String>,
    /// Run only the specified tasks skipping all dependencies
    #[arg(long = "skip-deps")]
    pub skip_deps: bool,
    /// Watch a specific file or directory
    ///
    /// By default, Watchexec watches the current directory.
    ///
    /// When watching a single file, it's often better to watch the containing directory instead, and filter on the filename. Some editors may replace the file with a new one when saving, and some platforms may not detect that or further changes.
    ///
    /// Upon starting, Watchexec resolves a "project origin" from the watched paths. See the help for '--project-origin' for more information.
    ///
    /// This option can be specified multiple times to watch multiple files or directories.
    ///
    /// The special value '/dev/null', provided as the only path watched, will cause Watchexec to not watch any paths. Other event sources (like signals or key events) may still be used.
    #[arg(
        long = "watch",
        short = 'w',
        help_heading = "Filtering",
        value_name = "PATH"
    )]
    pub watch: ::std::vec::Vec<::std::string::String>,
    /// Watch a specific directory, non-recursively
    ///
    /// Unlike '-w', folders watched with this option are not recursed into.
    ///
    /// This option can be specified multiple times to watch multiple directories non-recursively.
    #[arg(
        long = "watch-non-recursive",
        short = 'W',
        help_heading = "Filtering",
        value_name = "PATH"
    )]
    pub watch_non_recursive: ::std::vec::Vec<::std::string::String>,
    /// Watch files and directories from a file
    ///
    /// Each line in the file will be interpreted as if given to '-w'.
    ///
    /// For more complex uses (like watching non-recursively), use the argfile capability: build a file containing command-line options and pass it to watchexec with `@path/to/argfile`.
    ///
    /// The special value '-' will read from STDIN; this in incompatible with '--stdin-quit'.
    #[arg(
        long = "watch-file",
        short = 'F',
        help_heading = "Filtering",
        value_name = "PATH"
    )]
    pub watch_file: ::std::option::Option<::std::string::String>,
    /// Clear screen before running command
    ///
    /// If this doesn't completely clear the screen, try '--clear=reset'.
    #[arg(
        long = "clear",
        short = 'c',
        help_heading = "Output",
        value_optional,
        default_missing = "clear",
        value_name = "MODE"
    )]
    pub clear: ::std::option::Option<WatchClearValue>,
    /// What to do when receiving events while the command is running
    ///
    /// Default is to 'do-nothing', which ignores events while the command is running, so that changes that occur due to the command are ignored, like compilation outputs. You can also use 'queue' which will run the command once again when the current run has finished if any events occur while it's running, or 'restart', which terminates the running command and starts a new one. Finally, there's 'signal', which only sends a signal; this can be useful with programs that can reload their configuration without a full restart.
    ///
    /// The signal can be specified with the '--signal' option.
    #[arg(
        long = "on-busy-update",
        short = 'o',
        value_name = "MODE",
        default = "do-nothing"
    )]
    pub on_busy_update: ::std::option::Option<WatchOnBusyUpdateValue>,
    /// Restart the process if it's still running
    ///
    /// This is a shorthand for '--on-busy-update=restart'.
    #[arg(long = "restart", short = 'r', conflicts("--on-busy-update"))]
    pub restart: bool,
    /// Send a signal to the process when it's still running
    ///
    /// Specify a signal to send to the process when it's still running. This implies '--on-busy-update=signal'; otherwise the signal used when that mode is 'restart' is controlled by '--stop-signal'.
    ///
    /// See the long documentation for '--stop-signal' for syntax.
    ///
    /// Signals are not supported on Windows at the moment, and will always be overridden to 'kill'. See '--stop-signal' for more on Windows "signals".
    #[arg(
        long = "signal",
        short = 's',
        conflicts("--restart"),
        value_name = "SIGNAL"
    )]
    pub signal: ::std::option::Option<::std::string::String>,
    /// Signal to send to stop the command
    ///
    /// This is used by 'restart' and 'signal' modes of '--on-busy-update' (unless '--signal' is provided). The restart behaviour is to send the signal, wait for the command to exit, and if it hasn't exited after some time (see '--timeout-stop'), forcefully terminate it.
    ///
    /// The default on unix is "SIGTERM".
    ///
    /// Input is parsed as a full signal name (like "SIGTERM"), a short signal name (like "TERM"), or a signal number (like "15"). All input is case-insensitive.
    ///
    /// On Windows this option is technically supported but only supports the "KILL" event, as Watchexec cannot yet deliver other events. Windows doesn't have signals as such; instead it has termination (here called "KILL" or "STOP") and "CTRL+C", "CTRL+BREAK", and "CTRL+CLOSE" events. For portability the unix signals "SIGKILL", "SIGINT", "SIGTERM", and "SIGHUP" are respectively mapped to these.
    #[arg(long = "stop-signal", value_name = "SIGNAL")]
    pub stop_signal: ::std::option::Option<::std::string::String>,
    /// Time to wait for the command to exit gracefully
    ///
    /// This is used by the 'restart' mode of '--on-busy-update'. After the graceful stop signal is sent, Watchexec will wait for the command to exit. If it hasn't exited after this time, it is forcefully terminated.
    ///
    /// Takes a unit-less value in seconds, or a time span value such as "5min 20s". Providing a unit-less value is deprecated and will warn; it will be an error in the future.
    ///
    /// The default is 10 seconds. Set to 0 to immediately force-kill the command.
    ///
    /// This has no practical effect on Windows as the command is always forcefully terminated; see '--stop-signal' for why.
    #[arg(long = "stop-timeout", value_name = "TIMEOUT", default = "10s")]
    pub stop_timeout: ::std::option::Option<::std::string::String>,
    /// Translate signals from the OS to signals to send to the command
    ///
    /// Takes a pair of signal names, separated by a colon, such as "TERM:INT" to map SIGTERM to SIGINT. The first signal is the one received by watchexec, and the second is the one sent to the command. The second can be omitted to discard the first signal, such as "TERM:" to not do anything on SIGTERM.
    ///
    /// If SIGINT or SIGTERM are mapped, then they no longer quit Watchexec. Besides making it hard to quit Watchexec itself, this is useful to send pass a Ctrl-C to the command without also terminating Watchexec and the underlying program with it, e.g. with "INT:INT".
    ///
    /// This option can be specified multiple times to map multiple signals.
    ///
    /// Signal syntax is case-insensitive for short names (like "TERM", "USR2") and long names (like "SIGKILL", "SIGHUP"). Signal numbers are also supported (like "15", "31"). On Windows, the forms "STOP", "CTRL+C", and "CTRL+BREAK" are also supported to receive, but Watchexec cannot yet deliver other "signals" than a STOP.
    #[arg(long = "map-signal", value_name = "SIGNAL:SIGNAL")]
    pub map_signal: ::std::vec::Vec<::std::string::String>,
    /// Time to wait for new events before taking action
    ///
    /// When an event is received, Watchexec will wait for up to this amount of time before handling it (such as running the command). This is essential as what you might perceive as a single change may actually emit many events, and without this behaviour, Watchexec would run much too often. Additionally, it's not infrequent that file writes are not atomic, and each write may emit an event, so this is a good way to avoid running a command while a file is partially written.
    ///
    /// An alternative use is to set a high value (like "30min" or longer), to save power or bandwidth on intensive tasks, like an ad-hoc backup script. In those use cases, note that every accumulated event will build up in memory.
    ///
    /// Takes a unit-less value in milliseconds, or a time span value such as "5sec 20ms". Providing a unit-less value is deprecated and will warn; it will be an error in the future.
    ///
    /// The default is 50 milliseconds. Setting to 0 is highly discouraged.
    #[arg(
        long = "debounce",
        short = 'd',
        value_name = "TIMEOUT",
        default = "50ms"
    )]
    pub debounce: ::std::option::Option<::std::string::String>,
    /// Exit when stdin closes
    ///
    /// This watches the stdin file descriptor for EOF, and exits Watchexec gracefully when it is closed. This is used by some process managers to avoid leaving zombie processes around.
    #[arg(long = "stdin-quit")]
    pub stdin_quit: bool,
    /// Don't load gitignores
    ///
    /// Among other VCS exclude files, like for Mercurial, Subversion, Bazaar, DARCS, Fossil. Note that Watchexec will detect which of these is in use, if any, and only load the relevant files. Both global (like '~/.gitignore') and local (like '.gitignore') files are considered.
    ///
    /// This option is useful if you want to watch files that are ignored by Git.
    #[arg(long = "no-vcs-ignore", help_heading = "Filtering")]
    pub no_vcs_ignore: bool,
    /// Don't load project-local ignores
    ///
    /// This disables loading of project-local ignore files, like '.gitignore' or '.ignore' in the watched project. This is contrasted with '--no-vcs-ignore', which disables loading of Git and other VCS ignore files, and with '--no-global-ignore', which disables loading of global or user ignore files, like '~/.gitignore' or '~/.config/watchexec/ignore'.
    ///
    /// Supported project ignore files:
    ///
    ///   - Git: .gitignore at project root and child directories, .git/info/exclude, and the file pointed to by `core.excludesFile` in .git/config.
    ///   - Mercurial: .hgignore at project root and child directories.
    ///   - Bazaar: .bzrignore at project root.
    ///   - Darcs: _darcs/prefs/boring
    ///   - Fossil: .fossil-settings/ignore-glob
    ///   - Ripgrep/Watchexec/generic: .ignore at project root and child directories.
    ///
    /// VCS ignore files (Git, Mercurial, Bazaar, Darcs, Fossil) are only used if the corresponding VCS is discovered to be in use for the project/origin. For example, a .bzrignore in a Git repository will be discarded.
    #[arg(long = "no-project-ignore", help_heading = "Filtering")]
    pub no_project_ignore: bool,
    /// Don't load global ignores
    ///
    /// This disables loading of global or user ignore files, like '~/.gitignore', '~/.config/watchexec/ignore', or '%APPDATA%\Bazaar\2.0\ignore'. Contrast with '--no-vcs-ignore' and '--no-project-ignore'.
    ///
    /// Supported global ignore files
    ///
    ///   - Git (if core.excludesFile is set): the file at that path
    ///   - Git (otherwise): the first found of $XDG_CONFIG_HOME/git/ignore, %APPDATA%/.gitignore, %USERPROFILE%/.gitignore, $HOME/.config/git/ignore, $HOME/.gitignore.
    ///   - Bazaar: the first found of %APPDATA%/Bazaar/2.0/ignore, $HOME/.bazaar/ignore.
    ///   - Watchexec: the first found of $XDG_CONFIG_HOME/watchexec/ignore, %APPDATA%/watchexec/ignore, %USERPROFILE%/.watchexec/ignore, $HOME/.watchexec/ignore.
    ///
    /// Like for project files, Git and Bazaar global files will only be used for the corresponding VCS as used in the project.
    #[arg(long = "no-global-ignore", help_heading = "Filtering")]
    pub no_global_ignore: bool,
    /// Don't use internal default ignores
    ///
    /// Watchexec has a set of default ignore patterns, such as editor swap files, `*.pyc`, `*.pyo`, `.DS_Store`, `.bzr`, `_darcs`, `.fossil-settings`, `.git`, `.hg`, `.pijul`, `.svn`, and Watchexec log files.
    #[arg(long = "no-default-ignore", help_heading = "Filtering")]
    pub no_default_ignore: bool,
    /// Don't discover ignore files at all
    ///
    /// This is a shorthand for '--no-global-ignore', '--no-vcs-ignore', '--no-project-ignore', but even more efficient as it will skip all the ignore discovery mechanisms from the get go.
    ///
    /// Note that default ignores are still loaded, see '--no-default-ignore'.
    #[arg(long = "no-discover-ignore", help_heading = "Filtering")]
    pub no_discover_ignore: bool,
    /// Don't ignore anything at all
    ///
    /// This is a shorthand for '--no-discover-ignore', '--no-default-ignore'.
    ///
    /// Note that ignores explicitly loaded via other command line options, such as '--ignore' or '--ignore-file', will still be used.
    #[arg(long = "ignore-nothing", help_heading = "Filtering")]
    pub ignore_nothing: bool,
    /// Wait until first change before running command
    ///
    /// By default, Watchexec will run the command once immediately. With this option, it will instead wait until an event is detected before running the command as normal.
    #[arg(long = "postpone", short = 'p')]
    pub postpone: bool,
    /// Sleep before running the command
    ///
    /// This option will cause Watchexec to sleep for the specified amount of time before running the command, after an event is detected. This is like using "sleep 5 && command" in a shell, but portable and slightly more efficient.
    ///
    /// Takes a unit-less value in seconds, or a time span value such as "2min 5s". Providing a unit-less value is deprecated and will warn; it will be an error in the future.
    #[arg(long = "delay-run", value_name = "DURATION")]
    pub delay_run: ::std::option::Option<::std::string::String>,
    /// Poll for filesystem changes
    ///
    /// By default, and where available, Watchexec uses the operating system's native file system watching capabilities. This option disables that and instead uses a polling mechanism, which is less efficient but can work around issues with some file systems (like network shares) or edge cases.
    ///
    /// Optionally takes a unit-less value in milliseconds, or a time span value such as "2s 500ms", to use as the polling interval. If not specified, the default is 30 seconds. Providing a unit-less value is deprecated and will warn; it will be an error in the future.
    ///
    /// Aliased as '--force-poll'.
    #[arg(
        long = "poll",
        alias = "force-poll",
        value_optional,
        default_missing = "30s",
        value_name = "INTERVAL"
    )]
    pub poll: ::std::option::Option<::std::string::String>,
    /// Use a different shell
    ///
    /// By default, Watchexec will use '$SHELL' if it's defined or a default of 'sh' on Unix-likes, and either 'pwsh', 'powershell', or 'cmd' (CMD.EXE) on Windows, depending on what Watchexec detects is the running shell.
    ///
    /// With this option, you can override that and use a different shell, for example one with more features or one which has your custom aliases and functions.
    ///
    /// If the value has spaces, it is parsed as a command line, and the first word used as the shell program, with the rest as arguments to the shell.
    ///
    /// The command is run with the '-c' flag (except for 'cmd' on Windows, where it's '/C').
    ///
    /// The special value 'none' can be used to disable shell use entirely. In that case, the command provided to Watchexec will be parsed, with the first word being the executable and the rest being the arguments, and executed directly. Note that this parsing is rudimentary, and may not work as expected in all cases.
    ///
    /// Using 'none' is a little more efficient and can enable a stricter interpretation of the input, but it also means that you can't use shell features like globbing, redirection, control flow, logic, or pipes.
    ///
    /// Examples:
    ///
    /// Use without shell:
    ///
    ///   $ watchexec -n -- zsh -x -o shwordsplit scr
    ///
    /// Use with powershell core:
    ///
    ///   $ watchexec --shell=pwsh -- Test-Connection localhost
    ///
    /// Use with CMD.exe:
    ///
    ///   $ watchexec --shell=cmd -- dir
    ///
    /// Use with a different unix shell:
    ///
    ///   $ watchexec --shell=bash -- 'echo $BASH_VERSION'
    ///
    /// Use with a unix shell and options:
    ///
    ///   $ watchexec --shell='zsh -x -o shwordsplit' -- scr
    #[arg(long = "shell", help_heading = "Command", value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    /// Shorthand for '--shell=none'
    #[arg(short = 'n', help_heading = "Command")]
    pub n: bool,
    /// Configure event emission
    ///
    /// Watchexec can emit event information when running a command, which can be used by the child process to target specific changed files.
    ///
    /// One thing to take care with is assuming inherent behaviour where there is only chance. Notably, it could appear as if the `RENAMED` variable contains both the original and the new path being renamed. In previous versions, it would even appear on some platforms as if the original always came before the new. However, none of this was true. It's impossible to reliably and portably know which changed path is the old or new, "half" renames may appear (only the original, only the new), "unknown" renames may appear (change was a rename, but whether it was the old or new isn't known), rename events might split across two debouncing boundaries, and so on.
    ///
    /// This option controls where that information is emitted. It defaults to 'none', which doesn't emit event information at all. The other options are 'environment' (deprecated), 'stdio', 'file', 'json-stdio', and 'json-file'.
    ///
    /// The 'stdio' and 'file' modes are text-based: 'stdio' writes absolute paths to the stdin of the command, one per line, each prefixed with `create:`, `remove:`, `rename:`, `modify:`, or `other:`, then closes the handle; 'file' writes the same thing to a temporary file, and its path is given with the $WATCHEXEC_EVENTS_FILE environment variable.
    ///
    /// There are also two JSON modes, which are based on JSON objects and can represent the full set of events Watchexec handles. Here's an example of a folder being created on Linux:
    ///
    /// ```json
    ///   {
    ///     "tags": [
    ///       {
    ///         "kind": "path",
    ///         "absolute": "/home/user/your/new-folder",
    ///         "filetype": "dir"
    ///       },
    ///       {
    ///         "kind": "fs",
    ///         "simple": "create",
    ///         "full": "Create(Folder)"
    ///       },
    ///       {
    ///         "kind": "source",
    ///         "source": "filesystem",
    ///       }
    ///     ],
    ///     "metadata": {
    ///       "notify-backend": "inotify"
    ///     }
    ///   }
    /// ```
    ///
    /// The fields are as follows:
    ///
    ///   - `tags`, structured event data.
    ///   - `tags[].kind`, which can be:
    ///     * 'path', along with:
    ///       + `absolute`, an absolute path.
    ///       + `filetype`, a file type if known ('dir', 'file', 'symlink', 'other').
    ///     * 'fs':
    ///       + `simple`, the "simple" event type ('access', 'create', 'modify', 'remove', or 'other').
    ///       + `full`, the "full" event type, which is too complex to fully describe here, but looks like 'General(Precise(Specific))'.
    ///     * 'source', along with:
    ///       + `source`, the source of the event ('filesystem', 'keyboard', 'mouse', 'os', 'time', 'internal').
    ///     * 'keyboard', along with:
    ///       + `keycode`. Currently only the value 'eof' is supported.
    ///     * 'process', for events caused by processes:
    ///       + `pid`, the process ID.
    ///     * 'signal', for signals sent to Watchexec:
    ///       + `signal`, the normalised signal name ('hangup', 'interrupt', 'quit', 'terminate', 'user1', 'user2').
    ///     * 'completion', for when a command ends:
    ///       + `disposition`, the exit disposition ('success', 'error', 'signal', 'stop', 'exception', 'continued').
    ///       + `code`, the exit, signal, stop, or exception code.
    ///   - `metadata`, additional information about the event.
    ///
    /// The 'json-stdio' mode will emit JSON events to the standard input of the command, one per line, then close stdin. The 'json-file' mode will create a temporary file, write the events to it, and provide the path to the file with the $WATCHEXEC_EVENTS_FILE environment variable.
    ///
    /// Finally, the 'environment' mode was the default until 2.0. It sets environment variables with the paths of the affected files, for filesystem events:
    ///
    /// $WATCHEXEC_COMMON_PATH is set to the longest common path of all of the below variables, and so should be prepended to each path to obtain the full/real path. Then:
    ///
    ///   - $WATCHEXEC_CREATED_PATH is set when files/folders were created
    ///   - $WATCHEXEC_REMOVED_PATH is set when files/folders were removed
    ///   - $WATCHEXEC_RENAMED_PATH is set when files/folders were renamed
    ///   - $WATCHEXEC_WRITTEN_PATH is set when files/folders were modified
    ///   - $WATCHEXEC_META_CHANGED_PATH is set when files/folders' metadata were modified
    ///   - $WATCHEXEC_OTHERWISE_CHANGED_PATH is set for every other kind of pathed event
    ///
    /// Multiple paths are separated by the system path separator, ';' on Windows and ':' on unix. Within each variable, paths are deduplicated and sorted in binary order (i.e. neither Unicode nor locale aware).
    ///
    /// This is the legacy mode, is deprecated, and will be removed in the future. The environment is a very restricted space, while also limited in what it can usefully represent. Large numbers of files will either cause the environment to be truncated, or may error or crash the process entirely. The $WATCHEXEC_COMMON_PATH is also unintuitive, as demonstrated by the multiple confused queries that have landed in my inbox over the years.
    #[arg(
        long = "emit-events-to",
        help_heading = "Command",
        value_name = "MODE",
        default = "none"
    )]
    pub emit_events_to: ::std::option::Option<WatchEmitEventsToValue>,
    /// Only emit events to stdout, run no commands.
    ///
    /// This is a convenience option for using Watchexec as a file watcher, without running any commands. It is almost equivalent to using `cat` as the command, except that it will not spawn a new process for each event.
    ///
    /// This option requires `--emit-events-to` to be set, and restricts the available modes to `stdio` and `json-stdio`, modifying their behaviour to write to stdout instead of the stdin of the command.
    #[arg(
        long = "only-emit-events",
        help_heading = "Output",
        conflicts("--manual")
    )]
    pub only_emit_events: bool,
    /// Add env vars to the command
    ///
    /// This is a convenience option for setting environment variables for the command, without setting them for the Watchexec process itself.
    ///
    /// Use key=value syntax. Multiple variables can be set by repeating the option.
    #[arg(
        long = "env",
        short = 'E',
        help_heading = "Command",
        value_name = "KEY=VALUE"
    )]
    pub env: ::std::vec::Vec<::std::string::String>,
    /// Configure how the process is wrapped
    ///
    /// By default, Watchexec will run the command in a session on macOS, in a process group on other Unix platforms, and in a Job Object in Windows.
    ///
    /// Some Unix programs prefer running in a session, while others do not work in a process group.
    ///
    /// Use 'group' to use a process group, 'session' to use a process session, and 'none' to run the command directly. On Windows, either of 'group' or 'session' will use a Job Object.
    #[arg(long = "wrap-process", help_heading = "Command", value_name = "MODE")]
    pub wrap_process: ::std::option::Option<WatchWrapProcessValue>,
    /// Alert when commands start and end
    ///
    /// With this, Watchexec will emit a desktop notification when a command starts and ends, on supported platforms. On unsupported platforms, it may silently do nothing, or log a warning.
    #[arg(long = "notify", short = 'N', help_heading = "Output")]
    pub notify: bool,
    /// When to use terminal colours
    ///
    /// Setting the environment variable `NO_COLOR` to any value is equivalent to `--color=never`.
    #[arg(
        long = "color",
        alias = "colour",
        help_heading = "Output",
        value_name = "MODE",
        default = "auto"
    )]
    pub color: ::std::option::Option<WatchColorValue>,
    /// Print how long the command took to run
    ///
    /// This may not be exactly accurate, as it includes some overhead from Watchexec itself. Use the `time` utility, high-precision timers, or benchmarking tools for more accurate results.
    #[arg(long = "timings", help_heading = "Output")]
    pub timings: bool,
    /// Don't print starting and stopping messages
    ///
    /// By default Watchexec will print a message when the command starts and stops. This option disables this behaviour, so only the command's output, warnings, and errors will be printed.
    #[arg(long = "quiet", short = 'q', help_heading = "Output")]
    pub quiet: bool,
    /// Ring the terminal bell on command completion
    #[arg(long = "bell", help_heading = "Output")]
    pub bell: bool,
    /// Set the project origin
    ///
    /// Watchexec will attempt to discover the project's "origin" (or "root") by searching for a variety of markers, like files or directory patterns. It does its best but sometimes gets it it wrong, and you can override that with this option.
    ///
    /// The project origin is used to determine the path of certain ignore files, which VCS is being used, the meaning of a leading '/' in filtering patterns, and maybe more in the future.
    ///
    /// When set, Watchexec will also not bother searching, which can be significantly faster.
    #[arg(long = "project-origin", value_name = "DIRECTORY")]
    pub project_origin: ::std::option::Option<::std::string::String>,
    /// Set the working directory
    ///
    /// By default, the working directory of the command is the working directory of Watchexec. You can change that with this option. Note that paths may be less intuitive to use with this.
    #[arg(long = "workdir", value_name = "DIRECTORY")]
    pub workdir: ::std::option::Option<::std::string::String>,
    /// Filename extensions to filter to
    ///
    /// This is a quick filter to only emit events for files with the given extensions. Extensions can be given with or without the leading dot (e.g. 'js' or '.js'). Multiple extensions can be given by repeating the option or by separating them with commas.
    #[arg(
        long = "exts",
        short = 'e',
        help_heading = "Filtering",
        delimiter = ',',
        value_name = "EXTENSIONS"
    )]
    pub exts: ::std::vec::Vec<::std::string::String>,
    /// Filename patterns to filter to
    ///
    /// Provide a glob-like filter pattern, and only events for files matching the pattern will be emitted. Multiple patterns can be given by repeating the option. Events that are not from files (e.g. signals, keyboard events) will pass through untouched.
    #[arg(
        long = "filter",
        short = 'f',
        help_heading = "Filtering",
        value_name = "PATTERN"
    )]
    pub filter: ::std::vec::Vec<::std::string::String>,
    /// Files to load filters from
    ///
    /// Provide a path to a file containing filters, one per line. Empty lines and lines starting with '#' are ignored. Uses the same pattern format as the '--filter' option.
    ///
    /// This can also be used via the $WATCHEXEC_FILTER_FILES environment variable.
    #[arg(
        long = "filter-file",
        env = "WATCHEXEC_FILTER_FILES",
        help_heading = "Filtering",
        delimiter = ':',
        value_name = "PATH"
    )]
    pub filter_file: ::std::vec::Vec<::std::string::String>,
    /// [experimental] Filter programs.
    ///
    /// /!\ This option is EXPERIMENTAL and may change and/or vanish without notice.
    ///
    /// Provide your own custom filter programs in jaq (similar to jq) syntax. Programs are given an event in the same format as described in '--emit-events-to' and must return a boolean. Invalid programs will make watchexec fail to start; use '-v' to see program runtime errors.
    ///
    /// In addition to the jaq stdlib, watchexec adds some custom filter definitions:
    ///
    ///   - 'path | file_meta' returns file metadata or null if the file does not exist.
    ///
    ///   - 'path | file_size' returns the size of the file at path, or null if it does not exist.
    ///
    ///   - 'path | file_read(bytes)' returns a string with the first n bytes of the file at path.
    ///     If the file is smaller than n bytes, the whole file is returned. There is no filter to
    ///     read the whole file at once to encourage limiting the amount of data read and processed.
    ///
    ///   - 'string | hash', and 'path | file_hash' return the hash of the string or file at path.
    ///     No guarantee is made about the algorithm used: treat it as an opaque value.
    ///
    ///   - 'any | kv_store(key)', 'kv_fetch(key)', and 'kv_clear' provide a simple key-value store.
    ///     Data is kept in memory only, there is no persistence. Consistency is not guaranteed.
    ///
    ///   - 'any | printout', 'any | printerr', and 'any | log(level)' will print or log any given
    ///     value to stdout, stderr, or the log (levels = error, warn, info, debug, trace), and
    ///     pass the value through (so '[1] | log("debug") | .[]' will produce a '1' and log '[1]').
    ///
    /// All filtering done with such programs, and especially those using kv or filesystem access, is much slower than the other filtering methods. If filtering is too slow, events will back up and stall watchexec. Take care when designing your filters.
    ///
    /// If the argument to this option starts with an '@', the rest of the argument is taken to be the path to a file containing a jaq program.
    ///
    /// Jaq programs are run in order, after all other filters, and short-circuit: if a filter (jaq or not) rejects an event, execution stops there, and no other filters are run. Additionally, they stop after outputting the first value, so you'll want to use 'any' or 'all' when iterating, otherwise only the first item will be processed, which can be quite confusing!
    ///
    /// Find user-contributed programs or submit your own useful ones at <https://github.com/watchexec/watchexec/discussions/592>.
    ///
    /// ## Examples:
    ///
    /// Regexp ignore filter on paths:
    ///
    ///   'all(.tags[] | select(.kind == "path"); .absolute | test("[.]test[.]js$")) | not'
    ///
    /// Pass any event that creates a file:
    ///
    ///   'any(.tags[] | select(.kind == "fs"); .simple == "create")'
    ///
    /// Pass events that touch executable files:
    ///
    ///   'any(.tags[] | select(.kind == "path" && .filetype == "file"); .absolute | metadata | .executable)'
    ///
    /// Ignore files that start with shebangs:
    ///
    ///   'any(.tags[] | select(.kind == "path" && .filetype == "file"); .absolute | read(2) == "#!") | not'
    #[arg(
        long = "filter-prog",
        short = 'J',
        help_heading = "Filtering",
        value_name = "EXPRESSION"
    )]
    pub filter_prog: ::std::vec::Vec<::std::string::String>,
    /// Filename patterns to filter out
    ///
    /// Provide a glob-like filter pattern, and events for files matching the pattern will be excluded. Multiple patterns can be given by repeating the option. Events that are not from files (e.g. signals, keyboard events) will pass through untouched.
    #[arg(
        long = "ignore",
        short = 'i',
        help_heading = "Filtering",
        value_name = "PATTERN"
    )]
    pub ignore: ::std::vec::Vec<::std::string::String>,
    /// Files to load ignores from
    ///
    /// Provide a path to a file containing ignores, one per line. Empty lines and lines starting with '#' are ignored. Uses the same pattern format as the '--ignore' option.
    ///
    /// This can also be used via the $WATCHEXEC_IGNORE_FILES environment variable.
    #[arg(
        long = "ignore-file",
        env = "WATCHEXEC_IGNORE_FILES",
        help_heading = "Filtering",
        delimiter = ':',
        value_name = "PATH"
    )]
    pub ignore_file: ::std::vec::Vec<::std::string::String>,
    /// Filesystem events to filter to
    ///
    /// This is a quick filter to only emit events for the given types of filesystem changes. Choose from 'access', 'create', 'remove', 'rename', 'modify', 'metadata'. Multiple types can be given by repeating the option or by separating them with commas. By default, this is all types except for 'access'.
    ///
    /// This may apply filtering at the kernel level when possible, which can be more efficient, but may be more confusing when reading the logs.
    #[arg(
        long = "fs-events",
        help_heading = "Filtering",
        delimiter = ',',
        value_name = "EVENTS",
        default = "create,remove,rename,modify,metadata"
    )]
    pub fs_events: ::std::vec::Vec<WatchFsEventsValue>,
    /// Don't emit fs events for metadata changes
    ///
    /// This is a shorthand for '--fs-events create,remove,rename,modify'. Using it alongside the '--fs-events' option is non-sensical and not allowed.
    #[arg(long = "no-meta", help_heading = "Filtering", conflicts("--fs-events"))]
    pub no_meta: bool,
    /// Print events that trigger actions
    ///
    /// This prints the events that triggered the action when handling it (after debouncing), in a human readable form. This is useful for debugging filters.
    ///
    /// Use '-vvv' instead when you need more diagnostic information.
    #[arg(long = "print-events", help_heading = "Debugging")]
    pub print_events: bool,
    /// Show the manual page
    ///
    /// This shows the manual page for Watchexec, if the output is a terminal and the 'man' program is available. If not, the manual page is printed to stdout in ROFF format (suitable for writing to a watchexec.1 file).
    #[arg(long = "manual", help_heading = "Debugging")]
    pub manual: bool,
    #[arg(
        positional,
        value_name = "TASK",
        help = "Tasks to run\nCan specify multiple tasks by separating with `:::`\ne.g.: `mise run task1 arg1 arg2 ::: task2 arg1 arg2`\nDefaults to `default`",
        double_dash = "automatic"
    )]
    pub task: ::std::option::Option<::std::string::String>,
    /// Task and arguments to run
    #[arg(positional, value_name = "ARGS", double_dash = "automatic")]
    pub args: ::std::vec::Vec<::std::string::String>,
}

#[derive(ValueEnum)]
pub enum WatchClearValue {
    #[arg(name = "clear")]
    Clear,
    #[arg(name = "reset")]
    Reset,
}

#[derive(ValueEnum)]
pub enum WatchOnBusyUpdateValue {
    #[arg(name = "queue")]
    Queue,
    #[arg(name = "do-nothing")]
    DoNothing,
    #[arg(name = "restart")]
    Restart,
    #[arg(name = "signal")]
    Signal,
}

#[derive(ValueEnum)]
pub enum WatchEmitEventsToValue {
    #[arg(name = "environment")]
    Environment,
    #[arg(name = "stdio")]
    Stdio,
    #[arg(name = "file")]
    File,
    #[arg(name = "json-stdio")]
    JsonStdio,
    #[arg(name = "json-file")]
    JsonFile,
    #[arg(name = "none")]
    None,
}

#[derive(ValueEnum)]
pub enum WatchWrapProcessValue {
    #[arg(name = "group")]
    Group,
    #[arg(name = "session")]
    Session,
    #[arg(name = "none")]
    None,
}

#[derive(ValueEnum)]
pub enum WatchColorValue {
    #[arg(name = "auto")]
    Auto,
    #[arg(name = "always")]
    Always,
    #[arg(name = "never")]
    Never,
}

#[derive(ValueEnum)]
pub enum WatchFsEventsValue {
    #[arg(name = "access")]
    Access,
    #[arg(name = "create")]
    Create,
    #[arg(name = "remove")]
    Remove,
    #[arg(name = "rename")]
    Rename,
    #[arg(name = "modify")]
    Modify,
    #[arg(name = "metadata")]
    Metadata,
}

/// Display the installation path for a tool
///
/// The tool must be installed for this to work.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    # Show the latest installed version of node\n    # If it is is not installed, errors\n    $ \u{1b}[1mmise where node@20\u{1b}[22m\n    /home/jdx/.local/share/mise/installs/node/20.0.0\n\n    # Show the current, active install directory of node\n    # Errors if node is not referenced in any .tool-version file\n    $ \u{1b}[1mmise where node\u{1b}[22m\n    /home/jdx/.local/share/mise/installs/node/20.0.0\n"
)]
pub struct WhereArgs {
    #[arg(
        positional,
        value_name = "TOOL@VERSION",
        help = "Tool(s) to look up\ne.g.: ruby@3\nif \"@<PREFIX>\" is specified, it will show the latest installed version\nthat matches the prefix\notherwise, it will show the current, active installed version"
    )]
    pub tool_version: ::std::string::String,
    #[arg(
        positional,
        value_name = "ASDF_VERSION",
        help = "the version prefix to use when querying the latest version\nsame as the first argument after the \"@\"\nused for asdf compatibility",
        hide
    )]
    pub asdf_version: ::std::option::Option<::std::string::String>,
}

/// Shows the path that a tool's bin points to.
///
/// Use this to figure out what version of a tool is currently active.
#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise which node\u{1b}[22m\n    /home/username/.local/share/mise/installs/node/20.0.0/bin/node\n\n    $ \u{1b}[1mmise which node --plugin\u{1b}[22m\n    node\n\n    $ \u{1b}[1mmise which node --version\u{1b}[22m\n    20.0.0\n"
)]
pub struct WhichArgs {
    #[arg(
        help = "Use a specific tool@version\ne.g.: `mise which npm --tool=node@20`",
        long = "tool",
        short = 't',
        value_name = "TOOL@VERSION"
    )]
    pub tool: ::std::option::Option<::std::string::String>,
    #[arg(long = "complete", hide)]
    pub complete: bool,
    /// Show the plugin name instead of the path
    #[arg(long = "plugin", conflicts("--version"))]
    pub plugin: bool,
    /// Show the version instead of the path
    #[arg(long = "version", conflicts("--plugin"))]
    pub version: bool,
    /// The bin to look up
    #[arg(positional, value_name = "BIN_NAME", required_unless("--complete"))]
    pub bin_name: ::std::option::Option<::std::string::String>,
}

#[derive(Args)]
#[arg(unknown_flags = "value")]
#[arg(
    name = "mise",
    about = "Dev tools, env vars, and tasks in one CLI",
    long_about = "mise prepares your development environment before each command runs. https://github.com/jdx/mise",
    arg_required_else_help,
    after_long_help = "\u{1b}[1m\u{1b}[4mExamples:\u{1b}[22m\u{1b}[24m\n\n    $ \u{1b}[1mmise install node@20.0.0\u{1b}[22m       Install a specific node version\n    $ \u{1b}[1mmise install node@20\u{1b}[22m           Install a version matching a prefix\n    $ \u{1b}[1mmise install node\u{1b}[22m              Install the node version defined in config\n    $ \u{1b}[1mmise install\u{1b}[22m                   Install all plugins/tools defined in config\n\n    $ \u{1b}[1mmise install cargo:ripgrep\u{1b}[22m     Install something via cargo\n    $ \u{1b}[1mmise install npm:prettier\u{1b}[22m      Install something via npm\n\n    $ \u{1b}[1mmise use node@20\u{1b}[22m               Use node-20.x in current project\n    $ \u{1b}[1mmise use -g node@20\u{1b}[22m            Use node-20.x as default\n    $ \u{1b}[1mmise use node@latest\u{1b}[22m           Use latest node in current directory\n\n    $ \u{1b}[1mmise up --interactive\u{1b}[22m          Show a menu to upgrade tools\n\n    $ \u{1b}[1mmise x -- npm install\u{1b}[22m          `npm install` w/ config loaded into PATH\n    $ \u{1b}[1mmise x node@20 -- node app.js\u{1b}[22m  `node app.js` w/ config + node-20.x on PATH\n\n    $ \u{1b}[1mmise set NODE_ENV=production\u{1b}[22m   Set NODE_ENV=production in config\n\n    $ \u{1b}[1mmise run build\u{1b}[22m                 Run `build` tasks\n    $ \u{1b}[1mmise watch build\u{1b}[22m               Run `build` tasks repeatedly when files change\n\n    $ \u{1b}[1mmise settings\u{1b}[22m                  Show settings in use\n    $ \u{1b}[1mmise settings color=0\u{1b}[22m          Disable color by modifying global config file\n",
    default_subcommand = "run"
)]
pub struct Cli {
    /// Continue running tasks even if one fails
    #[arg(long = "continue-on-error", short = 'c', hide)]
    pub continue_on_error: bool,
    /// Change directory before running command
    #[arg(long = "cd", short = 'C', global, value_name = "DIR")]
    pub cd: ::std::option::Option<::std::string::String>,
    /// Set the environment for loading `mise.<ENV>.toml`
    #[arg(long = "env", short = 'E', global, value_name = "ENV")]
    pub env: ::std::vec::Vec<::std::string::String>,
    /// Force the operation
    #[arg(long = "force", short = 'f', hide)]
    pub force: bool,
    /// How many jobs to run in parallel; values below 1 are treated as 1 [default: 8]
    #[arg(
        long = "jobs",
        short = 'j',
        global,
        env = "MISE_JOBS",
        value_name = "JOBS"
    )]
    pub jobs: ::std::option::Option<::std::string::String>,
    /// Dry run, don't actually do anything
    #[arg(long = "dry-run", short = 'n', hide)]
    pub dry_run: bool,
    /// Set the profile (environment)
    #[arg(
        long = "profile",
        short = 'P',
        global,
        hide,
        conflicts("--env"),
        value_name = "PROFILE"
    )]
    pub profile: ::std::vec::Vec<::std::string::String>,
    /// Suppress non-error messages
    #[arg(
        long = "quiet",
        short = 'q',
        global,
        overrides("--silent", "--trace", "--verbose", "--debug", "--log-level")
    )]
    pub quiet: bool,
    #[arg(long = "shell", short = 's', hide, value_name = "SHELL")]
    pub shell: ::std::option::Option<::std::string::String>,
    #[arg(
        help = "Tool(s) to run in addition to what is in mise.toml files e.g.: node@20 python@3.10",
        long_help = "Tool(s) to run in addition to what is in mise.toml files\ne.g.: node@20 python@3.10",
        long = "tool",
        short = 't',
        hide,
        env = "MISE_QUIET",
        value_name = "TOOL@VERSION"
    )]
    pub tool: ::std::vec::Vec<::std::string::String>,
    /// Show extra output (use -vv for even more)
    #[arg(
        long = "verbose",
        short = 'v',
        global,
        count,
        overrides("--quiet", "--silent", "--trace", "--debug")
    )]
    pub verbose: u8,
    #[arg(long = "version", short = 'V', hide)]
    pub version: bool,
    /// Answer yes to all confirmation prompts
    #[arg(long = "yes", short = 'y', global)]
    pub yes: bool,
    /// Sets log level to debug
    #[arg(
        long = "debug",
        global,
        hide,
        overrides("--quiet", "--trace", "--verbose", "--silent", "--log-level")
    )]
    pub debug: bool,
    #[arg(
        long = "log-level",
        global,
        hide,
        overrides("--quiet", "--trace", "--verbose", "--silent", "--debug"),
        value_name = "LEVEL"
    )]
    pub log_level: ::std::option::Option<LogLevelValue>,
    /// Do not load any config files
    ///
    /// Can also use `MISE_NO_CONFIG=1`
    #[arg(long = "no-config")]
    pub no_config: bool,
    /// Do not load environment variables from config files
    ///
    /// Can also use `MISE_NO_ENV=1`
    #[arg(long = "no-env")]
    pub no_env: bool,
    /// Do not execute hooks from config files
    ///
    /// Can also use `MISE_NO_HOOKS=1`
    #[arg(long = "no-hooks")]
    pub no_hooks: bool,
    /// Hides elapsed time after each task completes
    ///
    /// Default to always hide with `MISE_TASK_TIMINGS=0`
    #[arg(long = "no-timings", alias = "no-timing", hide)]
    pub no_timings: bool,
    #[arg(long = "output", value_name = "OUTPUT")]
    pub output: ::std::option::Option<::std::string::String>,
    /// Read/write directly to stdin/stdout/stderr instead of by line
    #[arg(long = "raw", global)]
    pub raw: bool,
    /// Require lockfile URLs to be present during installation
    ///
    /// Fails if tools don't have pre-resolved URLs in the lockfile for the current platform. This prevents API calls to GitHub, aqua registry, etc. Can also be enabled via MISE_LOCKED=1 or settings.locked=true
    #[arg(long = "locked", global)]
    pub locked: bool,
    /// Suppress all task output and mise non-error messages
    #[arg(
        long = "silent",
        global,
        overrides("--quiet", "--trace", "--verbose", "--debug", "--log-level")
    )]
    pub silent: bool,
    /// Shows elapsed time after each task completes
    ///
    /// Default to always show with `MISE_TASK_TIMINGS=1`
    #[arg(long = "timings", alias = "timing", hide)]
    pub timings: bool,
    /// Sets log level to trace
    #[arg(
        long = "trace",
        global,
        hide,
        overrides("--quiet", "--silent", "--verbose", "--debug", "--log-level")
    )]
    pub trace: bool,
    #[arg(
        positional,
        value_name = "TASK",
        help = "Task to run",
        long_help = "Task to run.\n\nShorthand for `mise tasks run <TASK>`.",
        double_dash = "automatic"
    )]
    pub task: ::std::option::Option<::std::string::String>,
    /// Task arguments
    #[arg(positional, value_name = "TASK_ARGS", hide)]
    pub task_args: ::std::vec::Vec<::std::string::String>,
    #[arg(
        positional,
        value_name = "TASK_ARGS_LAST",
        hide,
        double_dash = "required"
    )]
    pub task_args_last: ::std::vec::Vec<::std::string::String>,
    #[arg(subcommand)]
    pub command: ::std::option::Option<Commands>,
}

#[derive(ValueEnum)]
pub enum LogLevelValue {
    #[arg(name = "trace")]
    Trace,
    #[arg(name = "debug")]
    Debug,
    #[arg(name = "info")]
    Info,
    #[arg(name = "warning")]
    Warning,
    #[arg(name = "error")]
    Error,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initializes mise in the current shell session
    #[arg(name = "activate")]
    Activate(Box<ActivateArgs>),
    /// Manage tool version aliases.
    #[arg(name = "tool-alias", alias_hidden("alias", "aliases"))]
    ToolAlias(Box<ToolAliasArgs>),
    /// [internal] simulates asdf for plugins that call "asdf" internally
    #[arg(name = "asdf", hide)]
    Asdf(Box<AsdfArgs>),
    /// Manage backends
    #[arg(name = "backends", alias_hidden("b", "backend", "backend-list"))]
    Backends(Box<BackendsArgs>),
    /// List all the active runtime bin paths
    #[arg(name = "bin-paths")]
    BinPaths(Box<BinPathsArgs>),
    /// Set up a machine for the current config in one command
    #[arg(name = "bootstrap", alias_hidden = "bs")]
    Bootstrap(Box<BootstrapArgs>),
    /// Manage the mise cache
    #[arg(name = "cache")]
    Cache(Box<CacheArgs>),
    /// Generate shell completions
    #[arg(name = "completion", alias_hidden("complete", "completions"))]
    Completion(Box<CompletionArgs>),
    /// Manage config files
    #[arg(name = "config", alias = "cfg", alias_hidden = "toml")]
    Config(Box<ConfigArgs>),
    /// Shows current active and installed runtime versions
    #[arg(name = "current", hide)]
    Current(Box<CurrentArgs>),
    /// Disable mise for current shell session
    #[arg(name = "deactivate")]
    Deactivate(Box<DeactivateArgs>),
    /// Output direnv function to use mise inside direnv
    #[arg(name = "direnv", hide)]
    Direnv(Box<DirenvArgs>),
    /// Manage dotfiles from `[dotfiles]` (deprecated)
    #[arg(name = "dotfiles", hide)]
    Dotfiles(Box<DotfilesArgs>),
    /// Check mise installation for possible problems
    #[arg(name = "doctor", alias = "dr")]
    Doctor(Box<DoctorArgs>),
    /// Starts a new shell with the mise environment built from the current configuration
    #[arg(name = "en")]
    En(Box<EnArgs>),
    /// Exports env vars to activate mise a single time
    #[arg(name = "env", alias = "e")]
    Env(Box<EnvArgs>),
    /// Execute a command with tool(s) set
    #[arg(name = "exec", alias = "x")]
    Exec(Box<ExecArgs>),
    /// Formats mise.toml
    #[arg(name = "fmt")]
    Fmt(Box<FmtArgs>),
    /// Generate files for various tools/services
    #[arg(name = "generate", alias = "gen", alias_hidden = "g")]
    Generate(Box<GenerateArgs>),
    /// GitHub related commands
    #[arg(name = "github", hide)]
    Github(Box<GithubArgs>),
    /// Sets/gets the global tool version(s)
    #[arg(name = "global", hide)]
    Global(Box<GlobalArgs>),
    /// [internal] called by activate hook to update env vars directory change
    #[arg(name = "hook-env", hide)]
    HookEnv(Box<HookEnvArgs>),
    /// [internal] called by shell when a command is not found
    #[arg(name = "hook-not-found", hide)]
    HookNotFound(Box<HookNotFoundArgs>),
    /// Removes mise CLI and all related data
    #[arg(name = "implode")]
    Implode(Box<ImplodeArgs>),
    /// Edit mise.toml interactively
    #[arg(name = "edit")]
    Edit(Box<EditArgs>),
    /// Install a tool version
    #[arg(name = "install", alias = "i")]
    Install(Box<InstallArgs>),
    /// Install a tool version to a specific path
    #[arg(name = "install-into")]
    InstallInto(Box<InstallIntoArgs>),
    /// Gets the latest available version for a plugin
    #[arg(name = "latest")]
    Latest(Box<LatestArgs>),
    /// Symlinks a tool version into mise
    #[arg(name = "link", alias = "ln")]
    Link(Box<LinkArgs>),
    /// Sets/gets tool version in local .tool-versions or mise.toml
    #[arg(name = "local", hide, alias_hidden = "l")]
    Local(Box<LocalArgs>),
    /// Update lockfile checksums and URLs for all specified platforms
    #[arg(name = "lock")]
    Lock(Box<LockArgs>),
    /// List installed and active tool versions
    #[arg(name = "ls", alias = "list")]
    Ls(Box<LsArgs>),
    /// List runtime versions available for install.
    #[arg(name = "ls-remote", alias_hidden("list-all", "list-remote"))]
    LsRemote(Box<LsRemoteArgs>),
    /// Run Model Context Protocol (MCP) server
    #[arg(name = "mcp")]
    Mcp(Box<McpArgs>),
    /// [experimental] Build OCI container images from a mise.toml
    #[arg(name = "oci")]
    Oci(Box<OciArgs>),
    /// Shows outdated tool versions
    #[arg(name = "outdated")]
    Outdated(Box<OutdatedArgs>),
    /// Show the individuals supporting mise as Patron-tier members
    #[arg(name = "patrons")]
    Patrons(Box<PatronsArgs>),
    /// Manage plugins
    #[arg(name = "plugins", alias = "p", alias_hidden("plugin", "plugin-list"))]
    Plugins(Box<PluginsArgs>),
    /// [experimental] Manage project dependencies
    #[arg(name = "deps", alias = "dep", alias_hidden = "prepare")]
    Deps(Box<DepsArgs>),
    /// Delete unused versions of tools
    #[arg(name = "prune")]
    Prune(Box<PruneArgs>),
    /// List available tools to install
    #[arg(name = "registry")]
    Registry(Box<RegistryArgs>),
    /// internal command to generate markdown from help
    #[arg(name = "render-help", hide)]
    RenderHelp(Box<RenderHelpArgs>),
    /// Creates new shims based on bin paths from currently installed tools.
    #[arg(name = "reshim")]
    Reshim(Box<ReshimArgs>),
    /// Run task(s)
    #[arg(name = "run", alias = "r")]
    Run(Box<RunArgs>),
    /// Search for tools in the registry
    #[arg(name = "search")]
    Search(Box<SearchArgs>),
    /// Updates mise itself.
    #[arg(name = "self-update")]
    SelfUpdate(Box<SelfUpdateArgs>),
    /// Set environment variables in mise.toml
    #[arg(name = "set", alias_hidden("ev", "env-vars"))]
    Set(Box<SetArgs>),
    /// Manage settings
    #[arg(name = "settings")]
    Settings(Box<SettingsArgs>),
    /// Sets a tool version for the current session.
    #[arg(name = "shell", alias = "sh")]
    Shell(Box<ShellArgs>),
    /// Manage shell aliases.
    #[arg(name = "shell-alias")]
    ShellAlias(Box<ShellAliasArgs>),
    /// Show the companies sponsoring mise and the jdx.dev open source tools
    #[arg(name = "sponsors")]
    Sponsors(Box<SponsorsArgs>),
    /// Synchronize tools from other version managers with mise
    #[arg(name = "sync")]
    Sync(Box<SyncArgs>),
    /// Manage tasks
    #[arg(name = "tasks", alias = "t", alias_hidden = "task")]
    Tasks(Box<TasksArgs>),
    /// Test a tool installs and executes
    #[arg(name = "test-tool")]
    TestTool(Box<TestToolArgs>),
    /// Display git provider tokens mise will use
    #[arg(name = "token")]
    Token(Box<TokenArgs>),
    /// Gets information about a tool
    #[arg(name = "tool")]
    Tool(Box<ToolArgs>),
    /// Execute a tool stub
    #[arg(name = "tool-stub")]
    ToolStub(Box<ToolStubArgs>),
    /// Marks a config file as trusted
    #[arg(name = "trust")]
    Trust(Box<TrustArgs>),
    /// Removes installed tool versions
    #[arg(name = "uninstall")]
    Uninstall(Box<UninstallArgs>),
    /// Remove environment variable(s) from the config file.
    #[arg(name = "unset")]
    Unset(Box<UnsetArgs>),
    /// Remove explicit trust for a config
    #[arg(name = "untrust")]
    Untrust(Box<UntrustArgs>),
    /// Removes installed tool versions from mise.toml
    #[arg(name = "unuse", alias("rm", "remove"))]
    Unuse(Box<UnuseArgs>),
    /// Upgrades outdated tools
    #[arg(name = "upgrade", alias = "up")]
    Upgrade(Box<UpgradeArgs>),
    /// Generate a usage CLI spec
    #[arg(name = "usage", hide)]
    Usage(Box<UsageArgs>),
    /// Installs a tool and adds the version to mise.toml.
    #[arg(name = "use", alias = "u")]
    Use(Box<UseArgs>),
    /// Display the version of mise
    #[arg(name = "version", alias = "v")]
    Version(Box<VersionArgs>),
    /// Run task(s) and watch for changes to rerun it
    #[arg(name = "watch", alias = "w")]
    Watch(Box<WatchArgs>),
    /// Display the installation path for a tool
    #[arg(name = "where")]
    Where(Box<WhereArgs>),
    /// Shows the path that a tool's bin points to.
    #[arg(name = "which")]
    Which(Box<WhichArgs>),
}
