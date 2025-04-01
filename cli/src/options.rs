use std::ffi::OsString;

use clap::{command, Parser, ValueEnum, ValueHint};

#[derive(Parser, Debug)]
#[command(
    author, version, about, long_about = None, max_term_width = 80,
)]
pub struct Options {
    #[arg(
        short = 's',
        long = "shell",
        help = "The shell for which to quote arguments.",
        value_hint = ValueHint::ExecutablePath,
        value_enum,
        env = "SHELL",
    )]
    pub shell: Shell,

    #[arg(
        help = "The arguments to quote. When none are provided, reads from stdin.",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub command: Vec<OsString>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    #[value(alias = "zsh")]
    Bash,
    Fish,
    #[value(alias = "dash")]
    Sh,
}
