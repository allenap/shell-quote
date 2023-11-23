# shell-quote

This escapes strings in a way that they can be inserted into shell scripts
without the risk that they're interpreted as, say, multiple arguments (like with
Bash's _word splitting_), paths (Bash's _pathname expansion_), shell
metacharacters, function calls, or other syntax. This is frequently not as
simple as wrapping a string in quotes.

Inspired by the Haskell [shell-escape][] package, which is the most
comprehensive implementation of shell escaping I've yet seen.

For now this package implements escaping for `/bin/sh`-like shells and [GNU
Bash][gnu-bash]. Please read the documentation for each module to learn about
some limitations and caveats.

[shell-escape]: https://github.com/solidsnack/shell-escape
[gnu-bash]: https://www.gnu.org/software/bash/

## Examples

When quoting using raw bytes it can be convenient to call [`Bash`]'s and
[`Sh`]'s associated functions directly:

```
use shell_quote::{Bash, Sh};
assert_eq!(Bash::quote("foobar"), b"foobar");
assert_eq!(Sh::quote("foobar"), b"foobar");
assert_eq!(Bash::quote("foo bar"), b"$'foo bar'");
assert_eq!(Sh::quote("foo bar"), b"'foo bar'");
```

It's also possible to use the extension trait [`QuoteExt`]:

```
use shell_quote::{Bash, Sh, QuoteExt};
assert_eq!(Vec::quoted(Bash, "foo bar"), b"$'foo bar'");
assert_eq!(Vec::quoted(Sh, "foo bar"), b"'foo bar'");
```

or, to construct something more elaborate:

```
use shell_quote::{Bash, QuoteExt};
let mut script: String = "echo ".into();
script.push_quoted(Bash, "foo bar");
script.push_str(" > ");
script.push_quoted(Bash, "/path/(to)/[output]");
assert_eq!(script, "echo $'foo bar' > $'/path/(to)/[output]'");
```
