# shell-quote

This escapes strings in a way that they can be inserted into shell scripts
without the risk that they're interpreted as, say, multiple arguments (like with
Bash's _word splitting_), paths (Bash's _pathname expansion_), shell
metacharacters, function calls, or other syntax. This is frequently not as
simple as wrapping a string in quotes.

Inspired by the Haskell [shell-escape][] package, which is the most
comprehensive implementation of shell escaping I've yet seen.

For now this package implements escaping for `/bin/sh`-like shells, [GNU
Bash][gnu-bash] and [fish][].
Please read the documentation for each module to learn about some limitations
and caveats.

[shell-escape]: https://github.com/solidsnack/shell-escape
[gnu-bash]: https://www.gnu.org/software/bash/
[fish]: https://fishshell.com/

## Examples

When quoting using raw bytes it can be convenient to call [`Bash`]'s and
[`Sh`]'s associated functions directly:

```rust
use shell_quote::{Bash, Sh, Fish};
assert_eq!(Bash::quote("foobar"), b"foobar");
assert_eq!(Sh::quote("foobar"), b"foobar");
assert_eq!(Fish::quote("foobar"), b"foobar");
assert_eq!(Bash::quote("foo bar"), b"$'foo bar'");
assert_eq!(Sh::quote("foo bar"), b"'foo bar'");
assert_eq!(Fish::quote("foo bar"), b"'foo bar'");
```

It's also possible to use the extension trait [`QuoteRefExt`] which provides a
[`quoted`][`QuoteRefExt::quoted`] function:

```rust
use shell_quote::{Bash, Sh, Fish, QuoteRefExt};
let quoted: Vec<u8> = "foo bar".quoted(Bash);
assert_eq!(quoted, b"$'foo bar'");
let quoted: Vec<u8> = "foo bar".quoted(Sh);
assert_eq!(quoted, b"'foo bar'");
let quoted: Vec<u8> = "foo bar".quoted(Fish);
assert_eq!(quoted, b"'foo bar'");
```

Or the extension trait [`QuoteExt`] for pushing quoted strings into a buffer:

```rust
use shell_quote::{Bash, QuoteExt};
let mut script: String = "echo ".into();
script.push_quoted(Bash, "foo bar");
script.push_str(" > ");
script.push_quoted(Bash, "/path/(to)/[output]");
assert_eq!(script, "echo $'foo bar' > $'/path/(to)/[output]'");
```
