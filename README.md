# shell-escape

This will escape strings in a way that they can be inserted into shell scripts without the risk that they're interpreted as, say, multiple arguments (like with Bash's _word splitting_), paths (Bash's _pathname expansion_), shell metacharacters, function calls, or other syntax. This is not as simple as using quotes.

Inspired by the Haskell [shell-escape](https://github.com/solidsnack/shell-escape) package, which is the most comprehensive implementation of shell escaping I've yet seen.

For now this package only implements escaping for Bash.
