<!--
    References; overridden in `src/lib.rs`.
    See https://linebender.org/blog/doc-include/.
 -->

[`&str`]: https://doc.rust-lang.org/stable/std/primitive.str.html
[`String`]: https://doc.rust-lang.org/stable/alloc/string/struct.String.html
[`bstr::BStr`]: https://docs.rs/bstr/latest/bstr/struct.BStr.html
[`bstr::BString`]: https://docs.rs/bstr/latest/bstr/struct.BString.html
[`slice`]: https://doc.rust-lang.org/stable/std/primitive.slice.html
[`Vec<u8>`]: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html
[`OsStr`]: https://doc.rust-lang.org/stable/std/ffi/struct.OsStr.html
[`OsString`]: https://doc.rust-lang.org/stable/std/ffi/struct.OsString.html
[`Path`]: https://doc.rust-lang.org/stable/std/path/struct.Path.html
[`PathBuf`]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html
[`Sh`]: https://docs.rs/shell-quote/latest/shell_quote/struct.Sh.html
[`Dash`]: https://docs.rs/shell-quote/latest/shell_quote/struct.Dash.html
[`Bash`]: https://docs.rs/shell-quote/latest/shell_quote/struct.Bash.html
[`Fish`]: https://docs.rs/shell-quote/latest/shell_quote/struct.Fish.html
[`Zsh`]: https://docs.rs/shell-quote/latest/shell_quote/struct.Zsh.html
[`QuoteRefExt`]: https://docs.rs/shell-quote/latest/shell_quote/trait.QuoteRefExt.html
[`QuoteRefExt::quoted`]: https://docs.rs/shell-quote/latest/shell_quote/trait.QuoteRefExt.html#tymethod.quoted
[`QuoteExt`]: https://docs.rs/shell-quote/latest/shell_quote/trait.QuoteExt.html

<!-- References end. -->

<div class="readme-only">

# shell-quote

</div>

**shell-quote** escapes strings in a way that they can be inserted into shell
scripts without the risk that they're interpreted as, say, multiple arguments
(like with Bash's _word splitting_), paths (Bash's _pathname expansion_), shell
metacharacters, function calls, or other syntax. This is frequently not as
simple as wrapping a string in quotes.

This package implements escaping for [GNU Bash][gnu-bash], [Z Shell][z-shell],
[fish][], and `/bin/sh`-like shells including [Dash][dash].

[dash]: https://en.wikipedia.org/wiki/Almquist_shell#dash
[gnu-bash]: https://www.gnu.org/software/bash/
[z-shell]: https://zsh.sourceforge.io/
[fish]: https://fishshell.com/

It can take as input many different string and byte string types:

- [`&str`] and [`String`]
- [`&bstr::BStr`][`bstr::BStr`] and [`bstr::BString`]
- [`&[u8]`][`slice`] and [`Vec<u8>`]
- [`&OsStr`][`OsStr`] and [`OsString`] (on UNIX)
- [`&Path`][`Path`] and [`PathBuf`]

and produce output as (or push into) the following types:

- [`String`] (for shells that support it, i.e. not [`Sh`]/[`Dash`])
- [`bstr::BString`]
- [`Vec<u8>`]
- [`OsString`] (on UNIX)

Inspired by the Haskell [shell-escape][] package.

[shell-escape]: https://github.com/solidsnack/shell-escape

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
let quoted: String = "foo bar".quoted(Bash);
assert_eq!(quoted, "$'foo bar'");
let quoted: Vec<u8> = "foo bar".quoted(Sh);
assert_eq!(quoted, b"foo' bar'");
let quoted: String = "foo bar".quoted(Fish);
assert_eq!(quoted, "foo' bar'");
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

## Notes on string encoding

<div class="warning">

Here we will use [`Bash`] for the example, but other shells may have similar _or
different_ behaviours; check their documentation.

</div>

When we use [`&str`] or [`String`] as an input type, UTF-8 code points of U+0080
and above are written into the quoted form just as they are encoded in UTF-8,
i.e. the bytes are the same and there are no escape sequences. Compare this to
using a different input type:

```rust
# use shell_quote::{Bash, QuoteRefExt};
let data: &str = "café";
let data_utf8_quoted_from_string_type: Vec<u8> = data.quoted(Bash);
assert_eq!(&data_utf8_quoted_from_string_type, b"$'caf\xC3\xA9'"); // UTF-8, verbatim.
let data_utf8_quoted_from_bytes: Vec<u8> = data.as_bytes().quoted(Bash);
assert_eq!(&data_utf8_quoted_from_bytes, b"$'caf\\xC3\\xA9'"); // Now hex escaped!
```

It follows then, supposing you need to use a text encoding that is not UTF-8,
that string types must be encoded _before_ passing to the functions from this
crate.

For example, the character 'é' (U+00E9):

- In ISO-8859-1, it is represented by the single byte `0xE9`.
- In UTF-8, it is represented by the two bytes `0xC3 0xA9`.

Using a hypothetical `encode_iso_8859_1` function:

```rust
# use shell_quote::{Bash, QuoteRefExt};
# fn encode_iso_8859_1(_s: &str) -> &[u8] {
#     &[99, 97, 102, 233]
# }
let data = "café";
let data_utf8_quoted: Vec<u8> = data.quoted(Bash);
assert_eq!(&data_utf8_quoted, b"$'caf\xC3\xA9'"); // UTF-8: 2 bytes for é.
let data_iso_8859_1: &[u8] = encode_iso_8859_1(data);
let data_iso_8859_1_quoted: Vec<u8> = data_iso_8859_1.quoted(Bash);
assert_eq!(&data_iso_8859_1_quoted, b"$'caf\\xE9'"); // ISO-8859-1: 1 byte, hex escaped.
```

## Compatibility

[`Sh`] can serve as a lowest common denominator for Bash, Z Shell, and
`/bin/sh`-like shells like Dash. However, fish's quoting rules are different
enough that you must use [`Fish`] for fish scripts.

Note that using [`Sh`] as a lowest common denominator brings with it other
issues; read its documentation carefully to understand the limitations.

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
