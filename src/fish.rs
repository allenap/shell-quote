use crate::{ascii::Char, quoter::QuoterSealed, util::u8_to_hex, Quotable, Quoter};

/// Quote byte strings for use with fish.
///
/// The documentation on [quoting][] and [escaping characters][] in fish is
/// confusing at first, especially when coming from a Bourne-like shell, but
/// essentially we have to be able to move and and out of a quoted string
/// context. For example, the escape sequence `\t` for a tab _must_ be outside
/// of quotes, single or double, to be recognised as a tab character by fish:
///
/// ```fish
/// echo 'foo'\t'bar'
/// ```
///
/// This emphasises the importance of using the correct quoting module for the
/// target shell.
///
/// [quoting]: https://fishshell.com/docs/current/language.html#quotes
/// [escaping characters]:
///     https://fishshell.com/docs/current/language.html#escaping-characters
#[derive(Debug, Clone, Copy)]
pub struct Fish;

impl Quoter for Fish {}

/// Expose [`Quoter`] implementation as default impl too, for convenience.
impl QuoterSealed for Fish {
    fn quote<'a, S: ?Sized + Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        Self::quote(s)
    }
    fn quote_into<'a, S: ?Sized + Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        Self::quote_into(s, sout)
    }
}

impl Fish {
    /// Quote a string of bytes into a new `Vec<u8>`.
    ///
    /// This will return one of the following:
    /// - The string as-is, if no escaping is necessary.
    /// - An escaped string, like `'foo \'bar'`, `\a'ABC'`
    ///
    /// See [`quote_into`](#method.quote_into) for a variant that extends an
    /// existing `Vec` instead of allocating a new one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::{Fish, Quoter};
    /// assert_eq!(Fish::quote("foobar"), b"foobar");
    /// assert_eq!(Fish::quote("foo 'bar"), b"foo' \\'bar'");
    /// ```
    pub fn quote<'a, S: ?Sized + Into<Quotable<'a>>>(s: S) -> Vec<u8> {
        let sin: Quotable<'a> = s.into();
        match escape_prepare(sin.bytes) {
            Prepared::Empty => vec![b'\'', b'\''],
            Prepared::Inert => sin.bytes.into(),
            Prepared::Escape(esc) => {
                let mut sout = Vec::with_capacity(esc.len() + 2);
                escape_chars(esc, &mut sout); // Do the work.
                sout
            }
        }
    }

    /// Quote a string of bytes into an existing `Vec<u8>`.
    ///
    /// See [`quote`](#method.quote) for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use shell_quote::{Fish, Quoter};
    /// let mut buf = Vec::with_capacity(128);
    /// Fish::quote_into("foobar", &mut buf);
    /// buf.push(b' ');  // Add a space.
    /// Fish::quote_into("foo 'bar", &mut buf);
    /// assert_eq!(buf, b"foobar foo' \\'bar'");
    /// ```
    ///
    pub fn quote_into<'a, S: ?Sized + Into<Quotable<'a>>>(s: S, sout: &mut Vec<u8>) {
        let sin: Quotable<'a> = s.into();
        match escape_prepare(sin.bytes) {
            Prepared::Empty => sout.extend(b"''"),
            Prepared::Inert => sout.extend(sin.bytes),
            Prepared::Escape(esc) => {
                sout.reserve(esc.len() + 2);
                escape_chars(esc, sout); // Do the work.
            }
        }
    }
}

// ----------------------------------------------------------------------------

enum Prepared {
    Empty,
    Inert,
    Escape(Vec<Char>),
}

fn escape_prepare(sin: &[u8]) -> Prepared {
    let esc: Vec<_> = sin.iter().map(Char::from).collect();
    // An optimisation: if the string is not empty and contains only "safe"
    // characters we can avoid further work.
    if esc.is_empty() {
        Prepared::Empty
    } else if esc.iter().all(Char::is_inert) {
        Prepared::Inert
    } else {
        Prepared::Escape(esc)
    }
}

fn escape_chars(esc: Vec<Char>, sout: &mut Vec<u8>) {
    #[derive(PartialEq)]
    enum QuoteStyle {
        Inside,
        Outside,
        Whatever,
    }
    use QuoteStyle::*;

    let mut inside_quotes_now = false;
    let mut push_literal = |style: QuoteStyle, literal: &[u8]| {
        if inside_quotes_now {
            if style == Outside {
                sout.push(b'\'');
                inside_quotes_now = false;
            }
        } else {
            if style == Inside {
                sout.push(b'\'');
                inside_quotes_now = true;
            }
        }
        sout.extend(literal);
    };
    for mode in esc {
        use Char::*;
        let mut tmp = b"\\x00".to_owned();
        match mode {
            Bell => push_literal(Outside, b"\\a"),
            Backspace => push_literal(Outside, b"\\b"),
            Escape => push_literal(Outside, b"\\e"),
            FormFeed => push_literal(Outside, b"\\f"),
            NewLine => push_literal(Outside, b"\\n"),
            CarriageReturn => push_literal(Outside, b"\\r"),
            HorizontalTab => push_literal(Outside, b"\\t"),
            VerticalTab => push_literal(Outside, b"\\v"),
            Control(ch) => {
                tmp[2..].copy_from_slice(&u8_to_hex(ch));
                push_literal(Outside, &tmp[..])
            }
            Backslash => push_literal(Inside, b"\\\\"),
            SingleQuote => push_literal(Inside, b"\\'"),
            DoubleQuote => push_literal(Inside, b"\""),
            Delete => push_literal(Outside, b"\\x7F"),
            PrintableInert(ch) => push_literal(Whatever, &ch.to_le_bytes()),
            Printable(ch) => push_literal(Inside, &ch.to_le_bytes()),
            Extended(ch) => {
                tmp[2..].copy_from_slice(&u8_to_hex(ch));
                push_literal(Outside, &tmp[..])
            }
        }
    }
    if inside_quotes_now {
        if sout.last() == Some(&b'\'') {
            sout.pop(); // Remove trailing quote.
        } else {
            sout.push(b'\'');
        }
    }
}
