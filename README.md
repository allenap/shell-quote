# shell-quote

This escapes strings in a way that they can be inserted into shell scripts
without the risk that they're interpreted as, say, multiple arguments (like with
Bash's _word splitting_), paths (Bash's _pathname expansion_), shell
metacharacters, function calls, or other syntax. This is frequently not as
simple as wrapping a string in quotes.

This package implements escaping for [GNU Bash][gnu-bash], [Z Shell][z-shell],
[fish][], and `/bin/sh`-like shells including [Dash][dash].

[dash]: https://en.wikipedia.org/wiki/Almquist_shell#dash
[gnu-bash]: https://www.gnu.org/software/bash/
[z-shell]: https://zsh.sourceforge.io/
[fish]: https://fishshell.com/

Inspired by the Haskell [shell-escape][] package.

[shell-escape]: https://github.com/solidsnack/shell-escape

## Compatibility

[`Sh`] can serve as a lowest common denominator for Bash, Z Shell, and
`/bin/sh`-like shells like Dash. However, fish's quoting rules are different
enough that you must use [`Fish`] for fish scripts.

## Feature flags

The following are all enabled by default:

- `bstr`: Support [`bstr::BStr`] and [`bstr::BString`].
- `bash`: Support [Bash][gnu-bash] and [Z Shell][z-shell].
- `fish`: Support [fish][].
- `sh`: Support `/bin/sh`-like shells including [Dash][dash].

To limit support to specific shells, you must disable this crate's default
features in `Cargo.toml` and re-enable those you want. For example:

```toml
[dependencies]
shell-quote = { version = "*", default-features = false, features = ["bash"] }
```

## Examples

When quoting using raw bytes it can be convenient to call [`Sh`]'s, [`Dash`]'s,
[`Bash`]'s, [`Fish`]'s, and [`Zsh`]'s associated functions directly:

```rust
use shell_quote::{Bash, Dash, Fish, Sh, Zsh};
// No quoting is necessary for simple strings.
assert_eq!(Sh::quote_vec("foobar"), b"foobar");
assert_eq!(Dash::quote_vec("foobar"), b"foobar");  // `Dash` is an alias for `Sh`
assert_eq!(Bash::quote_vec("foobar"), b"foobar");
assert_eq!(Zsh::quote_vec("foobar"), b"foobar");   // `Zsh` is an alias for `Bash`
assert_eq!(Fish::quote_vec("foobar"), b"foobar");
// In all shells, quoting is necessary for strings with spaces.
assert_eq!(Sh::quote_vec("foo bar"), b"foo' bar'");
assert_eq!(Dash::quote_vec("foo bar"), b"foo' bar'");
assert_eq!(Bash::quote_vec("foo bar"), b"$'foo bar'");
assert_eq!(Zsh::quote_vec("foo bar"), b"$'foo bar'");
assert_eq!(Fish::quote_vec("foo bar"), b"foo' bar'");
```

It's also possible to use the extension trait [`QuoteRefExt`] which provides a
[`quoted`][`QuoteRefExt::quoted`] function:

```rust
use shell_quote::{Bash, Sh, Fish, QuoteRefExt};
let quoted: Vec<u8> = "foo bar".quoted(Bash);
assert_eq!(quoted, b"$'foo bar'");
let quoted: Vec<u8> = "foo bar".quoted(Sh);
assert_eq!(quoted, b"foo' bar'");
let quoted: Vec<u8> = "foo bar".quoted(Fish);
assert_eq!(quoted, b"foo' bar'");
```

Or the extension trait [`QuoteExt`] for pushing quoted strings into a buffer:

```rust
use shell_quote::{Bash, QuoteExt};
let mut script: bstr::BString = "echo ".into();
script.push_quoted(Bash, "foo bar");
script.extend(b" > ");
script.push_quoted(Bash, "/path/(to)/[output]");
assert_eq!(script, "echo $'foo bar' > $'/path/(to)/[output]'");
```
