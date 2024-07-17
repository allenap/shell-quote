use std::io::{self, IsTerminal, Read, Write};

use shell_quote::{Bash, Fish, Sh};

mod options;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = <options::Options as clap::Parser>::parse();
    let quoted: Vec<u8> = if options.command.is_empty() && !io::stdin().is_terminal() {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf)?;
        match options.shell {
            options::Shell::Bash => Bash::quote_vec(&buf),
            options::Shell::Fish => Fish::quote_vec(&buf),
            options::Shell::Sh => Sh::quote_vec(&buf),
        }
    } else {
        options
            .command
            .iter()
            .fold(Vec::<u8>::new(), |mut acc, arg| {
                if !acc.is_empty() {
                    acc.push(b' ');
                }
                match options.shell {
                    options::Shell::Bash => Bash::quote_into_vec(arg, &mut acc),
                    options::Shell::Fish => Fish::quote_into_vec(arg, &mut acc),
                    options::Shell::Sh => Sh::quote_into_vec(arg, &mut acc),
                };
                acc
            })
    };
    io::stdout().write_all(&quoted)?;

    Ok(())
}
