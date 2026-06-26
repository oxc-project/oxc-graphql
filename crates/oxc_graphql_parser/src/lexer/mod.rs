mod cursor;
mod lookup;
mod token;
mod token_kind;

use crate::Error;
use crate::LimitTracker;
use crate::lexer::cursor::Cursor;
pub use token::Token;
pub use token_kind::TokenKind;

/// Parses GraphQL source text into tokens.
/// ```rust
/// use oxc_graphql_parser::Lexer;
///
/// let query = "
/// {
///     animal
///     ...snackSelection
///     ... on Pet {
///       playmates {
///         count
///       }
///     }
/// }
/// ";
/// let (tokens, errors) = Lexer::new(query).lex();
/// assert_eq!(errors.len(), 0);
/// ```
#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    finished: bool,
    cursor: Cursor<'a>,
    pub(crate) limit_tracker: LimitTracker,
}

#[derive(Debug)]
enum State {
    Start,
    Ident,
    StringLiteralEscapedUnicode(usize),
    StringLiteral,
    StringLiteralStart,
    BlockStringLiteral,
    BlockStringLiteralBackslash,
    StringLiteralBackslash,
    LeadingZero,
    IntegerPart,
    DecimalPoint,
    FractionalPart,
    ExponentIndicator,
    ExponentSign,
    ExponentDigit,
    Whitespace,
    Comment,
    SpreadOperator,
    MinusSign,
}

impl<'a> Lexer<'a> {
    /// Create a lexer for a GraphQL source text.
    ///
    /// The Lexer is an iterator over tokens and errors:
    /// ```rust
    /// use oxc_graphql_parser::Lexer;
    ///
    /// let query = "# --- GraphQL here ---";
    ///
    /// let mut lexer = Lexer::new(query);
    /// let mut tokens = vec![];
    /// for token in lexer {
    ///     match token {
    ///         Ok(token) => tokens.push(token),
    ///         Err(error) => panic!("{:?}", error),
    ///     }
    /// }
    /// ```
    pub fn new(input: &'a str) -> Self {
        Self {
            cursor: Cursor::new(input),
            finished: false,
            limit_tracker: LimitTracker::new(usize::MAX),
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit_tracker = LimitTracker::new(limit);
        self
    }

    /// Lex the full source text, consuming the lexer.
    pub fn lex(self) -> (Vec<Token<'a>>, Vec<Error>) {
        let mut tokens = vec![];
        let mut errors = vec![];

        for item in self {
            match item {
                Ok(token) => tokens.push(token),
                Err(error) => errors.push(error),
            }
        }

        (tokens, errors)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if self.limit_tracker.check_and_increment() {
            self.finished = true;
            return Some(Err(Error::limit(
                "token limit reached, aborting lexing",
                self.cursor.index(),
            )));
        }

        match self.cursor.advance() {
            Ok(token) => {
                if matches!(token.kind(), TokenKind::Eof) {
                    self.finished = true;
                }

                Some(Ok(token))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Cursor<'a> {
    fn advance(&mut self) -> Result<Token<'a>, Error> {
        let mut state = State::Start;
        let mut token = Token { kind: TokenKind::Eof, data: "", index: self.index() };

        loop {
            let Some(c) = self.bump() else {
                return self.eof(state, token);
            };
            match state {
                State::Start => {
                    if let Some(t) = lookup::punctuation_kind(c) {
                        token.kind = t;
                        token.data = self.current_str();
                        return Ok(token);
                    }

                    if lookup::is_namestart(c) {
                        token.kind = TokenKind::Name;
                        state = State::Ident;

                        continue;
                    }

                    if c != b'0' && c.is_ascii_digit() {
                        token.kind = TokenKind::Int;
                        state = State::IntegerPart;

                        continue;
                    }

                    match c {
                        b'"' => {
                            token.kind = TokenKind::StringValue;
                            state = State::StringLiteralStart;
                        }
                        b'#' => {
                            token.kind = TokenKind::Comment;
                            state = State::Comment;
                        }
                        b'.' => {
                            token.kind = TokenKind::Spread;
                            state = State::SpreadOperator;
                        }
                        b'-' => {
                            token.kind = TokenKind::Int;
                            state = State::MinusSign;
                        }
                        b'0' => {
                            token.kind = TokenKind::Int;
                            state = State::LeadingZero;
                        }
                        c if is_whitespace_assimilated(c) || (c == 0xEF && self.eat_bom()) => {
                            token.kind = TokenKind::Whitespace;
                            state = State::Whitespace;
                        }
                        c => {
                            let c = self.char_for_error(c);
                            return Err(Error::with_loc(
                                format!(r#"Unexpected character "{c}""#),
                                self.current_str().to_string(),
                                token.index,
                            ));
                        }
                    };
                }
                State::Ident => match c {
                    curr if is_name_continue(curr) => {}
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                },
                State::Whitespace => match c {
                    curr if is_whitespace_assimilated(curr) || (curr == 0xEF && self.eat_bom()) => {
                    }
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                },
                State::BlockStringLiteral => match c {
                    b'\\' => {
                        state = State::BlockStringLiteralBackslash;
                    }
                    b'"'
                        // Require two additional quotes to complete the triple quote.
                        if self.eatc(b'"') && self.eatc(b'"') => {
                            token.data = self.current_str();
                            return self.done(token);
                        }
                    _ => {}
                },
                State::StringLiteralStart => match c {
                    b'"' => {
                        if self.eatc(b'"') {
                            state = State::BlockStringLiteral;

                            continue;
                        }

                        token.data = self.current_str();
                        return self.done(token);
                    }
                    b'\\' => {
                        state = State::StringLiteralBackslash;
                    }
                    _ => {
                        state = State::StringLiteral;

                        continue;
                    }
                },
                State::StringLiteralEscapedUnicode(remaining) => match c {
                    b'"' => {
                        self.add_err(Error::with_loc(
                            "incomplete unicode escape sequence",
                            char::from(c).to_string(),
                            token.index,
                        ));
                        token.data = self.current_str();
                        return self.done(token);
                    }
                    c if !c.is_ascii_hexdigit() => {
                        self.add_err(Error::with_loc(
                            "invalid unicode escape sequence",
                            c.to_string(),
                            0,
                        ));
                        state = State::StringLiteral;

                        continue;
                    }
                    _ => {
                        if remaining <= 1 {
                            state = State::StringLiteral;
                            let hex_end = self.offset + 1;
                            let hex_start = hex_end - 4;
                            let hex = &self.source[hex_start..hex_end];
                            // `is_ascii_hexdigit()` checks in previous iterations ensures
                            // this `unwrap()` does not panic:
                            let code_point = u32::from_str_radix(hex, 16).unwrap();
                            if char::from_u32(code_point).is_none() {
                                // TODO: https://github.com/oxc-project/oxc-graphql-parser/issues/657 needs
                                // changes both here and in `ast/node_ext.rs`
                                let escape_sequence_start = hex_start - 2; // include "\u"
                                let escape_sequence = &self.source[escape_sequence_start..hex_end];
                                self.add_err(Error::with_loc(
                                    "surrogate code point is invalid in unicode escape sequence \
                                     (paired surrogate not supported yet: \
                                     https://github.com/oxc-project/oxc-graphql-parser/issues/657)",
                                    escape_sequence.to_owned(),
                                    0,
                                ));
                            }
                            continue;
                        }

                        state = State::StringLiteralEscapedUnicode(remaining - 1)
                    }
                },
                State::StringLiteral => match c {
                    b'"' => {
                        token.data = self.current_str();
                        return self.done(token);
                    }
                    curr if is_line_terminator(curr) => {
                        self.add_err(Error::with_loc(
                            "unexpected line terminator",
                            "".to_string(),
                            0,
                        ));
                    }
                    b'\\' => {
                        state = State::StringLiteralBackslash;
                    }
                    _ => {}
                },
                State::BlockStringLiteralBackslash => match c {
                    b'"' => {
                        // If this is \""", we need to eat 3 in total, and then continue parsing.
                        // The lexer does not un-escape escape sequences so it's OK
                        // if we take this path for \"", even if that is technically not an escape
                        // sequence.
                        if self.eatc(b'"') {
                            self.eatc(b'"');
                        }

                        state = State::BlockStringLiteral;
                    }
                    b'\\' => {
                        // We need to stay in the backslash state:
                        // it's legal to write \\\""" with two literal backslashes
                        // and then the escape sequence.
                    }
                    _ => {
                        state = State::BlockStringLiteral;
                    }
                },
                State::StringLiteralBackslash => match c {
                    curr if is_escaped_char(curr) => {
                        state = State::StringLiteral;
                    }
                    b'u' => {
                        state = State::StringLiteralEscapedUnicode(4);
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        self.add_err(Error::with_loc(
                            "unexpected escaped character",
                            c.to_string(),
                            0,
                        ));

                        state = State::StringLiteral;
                    }
                },
                State::LeadingZero => match c {
                    b'.' => {
                        token.kind = TokenKind::Float;
                        state = State::DecimalPoint;
                    }
                    b'e' | b'E' => {
                        token.kind = TokenKind::Float;
                        state = State::ExponentIndicator;
                    }
                    _ if c.is_ascii_digit() => {
                        return Err(Error::with_loc(
                            "Numbers must not have non-significant leading zeroes",
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ if lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as integer suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                },
                State::IntegerPart => match c {
                    curr if curr.is_ascii_digit() => {}
                    b'.' => {
                        token.kind = TokenKind::Float;
                        state = State::DecimalPoint;
                    }
                    b'e' | b'E' => {
                        token.kind = TokenKind::Float;
                        state = State::ExponentIndicator;
                    }
                    _ if lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as integer suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                },
                State::DecimalPoint => match c {
                    curr if curr.is_ascii_digit() => {
                        state = State::FractionalPart;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`, expected fractional digit"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                State::FractionalPart => match c {
                    curr if curr.is_ascii_digit() => {}
                    b'e' | b'E' => {
                        state = State::ExponentIndicator;
                    }
                    _ if c == b'.' || lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as float suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                },
                State::ExponentIndicator => match c {
                    _ if c.is_ascii_digit() => {
                        state = State::ExponentDigit;
                    }
                    b'+' | b'-' => {
                        state = State::ExponentSign;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`, expected exponent digit or sign"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                State::ExponentSign => match c {
                    _ if c.is_ascii_digit() => {
                        state = State::ExponentDigit;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`, expected exponent digit"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                State::ExponentDigit => match c {
                    _ if c.is_ascii_digit() => {
                        state = State::ExponentDigit;
                    }
                    _ if c == b'.' || lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as float suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                },
                State::SpreadOperator => {
                    if c == b'.' && self.eatc(b'.') {
                        token.data = self.current_str();
                        return Ok(token);
                    }
                    return self.unterminated_spread_operator(&token);
                }
                State::MinusSign => match c {
                    b'0' => {
                        state = State::LeadingZero;
                    }
                    curr if curr.is_ascii_digit() => {
                        state = State::IntegerPart;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                State::Comment => match c {
                    curr if is_line_terminator(curr) => {
                        token.data = self.prev_str();
                        return self.done(token);
                    }
                    _ => {}
                },
            }
        }
    }

    fn char_for_error(&mut self, c: u8) -> char {
        if c.is_ascii() { char::from(c) } else { self.consume_current_char() }
    }

    fn eof(&mut self, state: State, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        match state {
            State::Start => {
                // Report EOF at the end of the input rather than one byte past it.
                let end = self.source.len();
                self.offset = end;
                token.index = end;
                Ok(token)
            }
            State::StringLiteralStart => {
                let curr = self.current_str();

                Err(Error::with_loc(
                    "unexpected end of data while lexing string value",
                    curr.to_string(),
                    token.index,
                ))
            }
            State::StringLiteral
            | State::BlockStringLiteral
            | State::StringLiteralEscapedUnicode(_)
            | State::BlockStringLiteralBackslash
            | State::StringLiteralBackslash => {
                let curr = self.drain();

                Err(Error::with_loc("unterminated string value", curr.to_string(), token.index))
            }
            State::SpreadOperator => self.unterminated_spread_operator(&token),
            State::MinusSign => Err(Error::with_loc(
                "Unexpected character \"-\"",
                self.current_str().to_string(),
                token.index,
            )),
            State::DecimalPoint | State::ExponentIndicator | State::ExponentSign => {
                Err(Error::with_loc(
                    "Unexpected EOF in float value",
                    self.current_str().to_string(),
                    token.index,
                ))
            }
            State::Ident
            | State::LeadingZero
            | State::IntegerPart
            | State::FractionalPart
            | State::ExponentDigit
            | State::Whitespace
            | State::Comment => {
                if let Some(mut err) = self.err.take() {
                    err.set_data(self.current_str().to_string());
                    return Err(err);
                }

                token.data = self.current_str();

                Ok(token)
            }
        }
    }

    fn unterminated_spread_operator(&mut self, token: &Token<'a>) -> Result<Token<'a>, Error> {
        let data = self.current_str();

        Err(Error::with_loc("Unterminated spread operator", data.to_string(), token.index))
    }

    fn done(&mut self, token: Token<'a>) -> Result<Token<'a>, Error> {
        if let Some(mut err) = self.err.take() {
            err.set_data(token.data.to_string());
            err.index = token.index;
            return Err(err);
        }
        Ok(token)
    }
}

/// Ignored tokens other than comments and commas are assimilated to whitespace
/// <https://spec.graphql.org/October2021/#Ignored>
fn is_whitespace_assimilated(c: u8) -> bool {
    matches!(
        c,
        // https://spec.graphql.org/October2021/#WhiteSpace
        b'\t'
        | b' '
        // https://spec.graphql.org/October2021/#LineTerminator
        | b'\n'
        | b'\r'
    )
}

/// <https://spec.graphql.org/October2021/#NameContinue>
fn is_name_continue(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
}

fn is_line_terminator(c: u8) -> bool {
    matches!(c, b'\n' | b'\r')
}

// EscapedCharacter
//     "  \  /  b  f  n  r  t
fn is_escaped_char(c: u8) -> bool {
    matches!(c, b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't')
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unterminated_string() {
        let schema = r#"
type Query {
    name: String
    format: String = "Y-m-d\\TH:i:sP"
}
        "#;
        let (tokens, errors) = Lexer::new(schema).lex();
        dbg!(tokens);
        dbg!(errors);
    }

    #[test]
    fn token_limit() {
        let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(10);
        let (tokens, errors) = lexer.lex();
        assert_eq!(tokens.len(), 10);
        assert_eq!(errors, &[Error::limit("token limit reached, aborting lexing", 17)]);
    }

    #[test]
    fn token_limit_exact() {
        let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(26);
        let (tokens, errors) = lexer.lex();
        assert_eq!(tokens.len(), 26);
        assert!(errors.is_empty());

        let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(25);
        let (tokens, errors) = lexer.lex();
        assert_eq!(tokens.len(), 25);
        assert_eq!(errors, &[Error::limit("token limit reached, aborting lexing", 31)]);
    }

    #[test]
    fn errors_and_token_limit() {
        let lexer = Lexer::new("type Query { ..a a a a a a a a a }").with_limit(10);
        let (tokens, errors) = lexer.lex();
        // Errors contribute to the token limit
        assert_eq!(tokens.len(), 9);
        assert_eq!(
            errors,
            &[
                Error::with_loc("Unterminated spread operator", "..".to_string(), 13),
                Error::limit("token limit reached, aborting lexing", 18),
            ],
        );
    }

    #[test]
    fn stream_produces_original_input() {
        let schema = r#"
type Query {
    name: String
    format: String = "Y-m-d\\TH:i:sP"
}
        "#;

        let lexer = Lexer::new(schema);
        let processed_schema =
            lexer.into_iter().fold(String::new(), |acc, token| acc + token.unwrap().data());

        assert_eq!(schema, processed_schema);
    }

    #[test]
    fn quoted_block_comment() {
        let input = r#"
"""
Not an escape character:
'/\W/'
Escape character:
\"""
\"""\"""
Not escape characters:
\" \""
Escape character followed by a quote:
\""""
"""
        "#;

        let (tokens, errors) = Lexer::new(input).lex();
        assert!(errors.is_empty());
        // The token data should be literally the source text.
        assert_eq!(
            tokens[1].data,
            r#"
"""
Not an escape character:
'/\W/'
Escape character:
\"""
\"""\"""
Not escape characters:
\" \""
Escape character followed by a quote:
\""""
"""
"#
            .trim(),
        );

        let input = r#"
# String contents: """
"""\""""""
# Unclosed block string
"""\"""
        "#;
        let (tokens, errors) = Lexer::new(input).lex();
        assert_eq!(tokens[3].data, r#""""\"""""""#);
        assert_eq!(
            errors,
            &[Error::with_loc(
                "unterminated string value",
                r#""""\"""
        "#
                .to_string(),
                59,
            )]
        );
    }

    #[test]
    fn unexpected_character() {
        let schema = r#"
type Query {
    name: String
}
/
        "#;
        let (tokens, errors) = Lexer::new(schema).lex();
        dbg!(tokens);
        assert_eq!(errors, &[Error::with_loc("Unexpected character \"/\"", "/".to_string(), 33,)]);
    }
}
