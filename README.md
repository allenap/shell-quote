# shell-escape

This escapes strings in a way that they can be inserted into shell scripts
without the risk that they're interpreted as, say, multiple arguments (like with
Bash's _word splitting_), paths (Bash's _pathname expansion_), shell
metacharacters, function calls, or other syntax. This is frequently not as
simple as wrapping a string in quotes.

Inspired by the Haskell [shell-escape][] package, which is the most comprehensive
implementation of shell escaping I've yet seen.

For now this package implements escaping for `/bin/sh`-like shells and [GNU Bash][gnu-bash].

[shell-escape]: https://github.com/solidsnack/shell-escape
[gnu-bash]: https://www.gnu.org/software/bash/
