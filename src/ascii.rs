#![cfg(any(feature = "bash", feature = "fish", feature = "sh"))]

//! Scanner for ASCII control codes, shell metacharacters, printable characters,
//! and extended codes, i.e. classify each byte in a stream according to where
//! it appears in extended ASCII.
//!
//! # On widening the inert set
//!
//! [`Char::PrintableInert`] means "safe to emit bare, wherever this byte lands
//! in a word, in every shell we support". Moving a byte into it makes output
//! terser, which is nice, but the cost of being wrong is that we emit something
//! the shell does not read as a single literal word – so the bar is high.
//!
//! `:`, `@`, and `+` clear it. Each was checked in leading, medial, and
//! trailing position against `/bin/sh`, Bash 3.2 and 5.3, Dash, Z Shell 5.9,
//! and fish 4.8.
//!
//! `%` and `=` were tried and rejected. Both are inert in most positions, and
//! both could in principle be handled by tracking position within the word, but
//! between them they attracted this list, and there is no reason to think it is
//! complete:
//!
//! - `FOO=bar` is an assignment rather than a word in Bourne-like shells, and
//!   in fish it is an outright error, "Unsupported use of '='".
//! - `FOO+=bar` is an _append_ assignment in Bash and Z Shell. Note that `+` is
//!   itself inert, so any position-tracking scheme has to know that a name may
//!   be followed by an optional `+` before the `=`. This one was missed first
//!   time round.
//! - `=foo` is subject to `=` expansion in Z Shell, where `EQUALS` is on by
//!   default. This matters here because `Zsh` is an alias for `Bash`.
//! - With `MAGIC_EQUAL_SUBST` set, Z Shell expands after _any_ `=` in _any_
//!   word: `--arg=~root` becomes `--arg=/var/root`. We survive that only
//!   because `~`, `$`, `*`, `?`, and `[` are all still quoted; keep it that way.
//! - `%1` in command position is a job specification. In Bash **no quoting
//!   helps** – see the warning on [`crate::Bash`] – but in Z Shell quoting does
//!   help, so making `%` inert would take something Z Shell can be protected
//!   from and make it unfixable there too.
//!
//! For balance, these look dangerous and are not, all checked against Bash and
//! Z Shell: `--arg=var`, `a.b=c`, `1=x`, `a+b=c`, and `FOO++=bar` cannot be read
//! as assignments; and `-=`, `*=`, `/=`, `%=`, `<<=`, `>>=`, `&=`, `^=`, `|=`
//! are _arithmetic_ operators, meaningful only inside `(( … ))`, `let`, and
//! `$(( … ))`, never at the level of a word.
//!
//! The lesson worth keeping: shell behaviour here is conditional on shell
//! options (`EXTENDED_GLOB`, `MAGIC_EQUAL_SUBST`), on job control, and on
//! whether the shell is interactive – none of which a plain non-interactive
//! `sh -c` test exercises. Probing that way will under-report. It is how both
//! `FOO+=bar` and `%1` were missed on the first attempt.

use std::borrow::Borrow;

#[derive(PartialEq)]
pub(crate) enum Char {
    Bell,
    Backspace,
    Escape,
    FormFeed,
    NewLine,
    CarriageReturn,
    HorizontalTab,
    VerticalTab,
    Control(u8),
    Backslash,
    SingleQuote,
    DoubleQuote,
    Delete,
    PrintableInert(u8),
    Printable(u8),
    Extended(u8),
}

impl Char {
    pub fn from<T: Borrow<u8>>(ch: T) -> Self {
        let ch = *ch.borrow();
        use Char::*;
        match ch {
            // ASCII control characters that frequently have dedicated backslash
            // sequences when quoted.
            BEL => Bell,
            BS => Backspace,
            ESC => Escape,
            FF => FormFeed,
            LF => NewLine,
            CR => CarriageReturn,
            TAB => HorizontalTab,
            VT => VerticalTab,

            // ASCII control characters, the rest.
            0x00..=0x06 | 0x0E..=0x1A | 0x1C..=0x1F => Control(ch),

            // ASCII printable characters that can have dedicated backslash
            // sequences when quoted or otherwise need some special treatment.
            b'\\' => Backslash,
            b'\'' => SingleQuote,
            b'\"' => DoubleQuote,
            DEL => Delete,

            // ASCII printable letters, numbers, and "safe" punctuation.
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => PrintableInert(ch),
            b',' | b'.' | b'/' | b'_' | b'-' => PrintableInert(ch),

            // ASCII punctuation which is inert wherever it lands in a word, in
            // every shell this crate supports – checked in leading, medial, and
            // trailing position against `/bin/sh`, Bash 3.2 and 5.3, Dash, Z
            // Shell 5.9, and fish 4.8.
            b':' | b'@' | b'+' => PrintableInert(ch),

            // ASCII punctuation which can have significance in the shell.
            b'|' | b'&' | b';' | b'(' | b')' | b'<' | b'>' => Printable(ch),
            b' ' | b'?' | b'[' | b']' | b'{' | b'}' | b'`' => Printable(ch),
            b'~' | b'!' | b'$' | b'*' | b'#' | b'^' => Printable(ch),

            // These two look inert and are not; they are quoted deliberately,
            // and the module documentation above says at length why, so that
            // nobody has to rediscover it.
            b'%' | b'=' => Printable(ch),

            // ASCII extended characters, or high bytes.
            0x80..=0xff => Extended(ch),
        }
    }

    #[inline]
    pub fn is_inert(&self) -> bool {
        matches!(self, Char::PrintableInert(_))
    }

    #[inline]
    #[cfg(feature = "sh")]
    pub fn code(&self) -> u8 {
        use Char::*;
        match *self {
            Bell => BEL,
            Backspace => BS,
            Escape => ESC,
            FormFeed => FF,
            NewLine => LF,
            CarriageReturn => CR,
            HorizontalTab => TAB,
            VerticalTab => VT,
            Control(ch) => ch,
            Backslash => b'\\',
            SingleQuote => b'\'',
            DoubleQuote => b'"',
            Delete => DEL,
            PrintableInert(ch) => ch,
            Printable(ch) => ch,
            Extended(ch) => ch,
        }
    }
}

const BEL: u8 = 0x07; // -> \a
const BS: u8 = 0x08; // -> \b
const TAB: u8 = 0x09; // -> \t
const LF: u8 = 0x0A; // -> \n
const VT: u8 = 0x0B; // -> \v
const FF: u8 = 0x0C; // -> \f
const CR: u8 = 0x0D; // -> \r
const ESC: u8 = 0x1B; // -> \e
const DEL: u8 = 0x7F;

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "sh")]
    fn test_code() {
        for ch in u8::MIN..=u8::MAX {
            let char = super::Char::from(ch);
            assert_eq!(ch, char.code());
        }
    }
}
