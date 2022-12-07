use crate::util::{is_token, trimmed, FlagOperations};
use nom::{
    branch::alt,
    bytes::complete::tag as complete_tag,
    bytes::streaming::{tag, take_till, take_till1, take_while, take_while1},
    character::{
        complete::space1 as complete_space1,
        is_space,
        streaming::{space0, space1},
    },
    combinator::{complete, map, not, opt, peek},
    sequence::tuple,
    Err::Incomplete,
    IResult, Needed,
};

/// Helper for Parsed bytes and corresponding Flags
pub type ParsedBytes<'a> = (&'a [u8], u64);
// Helper for Parsed Headers and corresonding termination
pub type ParsedHeaders = (Vec<Header>, bool);
// Helper for matched leading whitespace, byes, and trailing whitespace
pub type SurroundedBytes<'a> = (&'a [u8], &'a [u8], &'a [u8]);
// Helper for matched eol+ folding bytes + flags
pub type FoldingBytes<'a> = (&'a [u8], &'a [u8], u64);
// Helper for folding or terminator bytes
pub type FoldingOrTerminator<'a> = (ParsedBytes<'a>, Option<&'a [u8]>);
// Helper for value bytes and the value terminator
pub type ValueBytes<'a> = (&'a [u8], FoldingOrTerminator<'a>);

#[derive(Debug, PartialEq, Eq)]
pub struct Flags;

impl Flags {
    pub const FOLDING: u64 = 0x0001;
    pub const FOLDING_SPECIAL_CASE: u64 = (0x0002 | Self::FOLDING);
    pub const NAME_EMPTY: u64 = 0x0004;
    pub const VALUE_EMPTY: u64 = 0x0008;
    pub const NAME_NON_TOKEN_CHARS: u64 = 0x0010;
    pub const NAME_TRAILING_WHITESPACE: u64 = 0x0020;
    pub const NAME_LEADING_WHITESPACE: u64 = 0x0040;
    pub const NULL_TERMINATED: u64 = 0x0080;
    pub const MISSING_COLON: u64 = (0x0100 | Self::NAME_EMPTY);
    pub const DEFORMED_EOL: u64 = 0x0200;
    pub const TERMINATOR_SPECIAL_CASE: u64 = 0x0400;
    pub const DEFORMED_SEPARATOR: u64 = (0x0800 | Self::NAME_NON_TOKEN_CHARS);
    pub const FOLDING_EMPTY: u64 = (0x1000 | Self::DEFORMED_EOL);
    pub const PART_HEADER_REPEATED: u64 = 0x4000;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    pub name: Vec<u8>,
    pub flags: u64,
}

impl Name {
    pub fn new(name: &[u8], flags: u64) -> Self {
        Self {
            name: trimmed(name).to_vec(),
            flags,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value {
    pub value: Vec<u8>,
    pub flags: u64,
}

impl Value {
    pub fn new(value: &[u8], flags: u64) -> Self {
        Self {
            value: trimmed(value).to_vec(),
            flags,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub name: Name,
    pub value: Value,
}

impl Header {
    pub fn new(name: Name, value: Value) -> Self {
        Self { name, value }
    }

    pub fn new_with_flags(
        name_bytes: &[u8],
        name_flags: u64,
        value_bytes: &[u8],
        value_flags: u64,
    ) -> Self {
        Self::new(
            Name::new(name_bytes, name_flags),
            Value::new(value_bytes, value_flags),
        )
    }
}

/// Enumerates possible parser types
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Side {
    /// Request Parser: null terminates
    Request,
    /// Response Parser: accepts CR as a line ending
    Response,
}

pub struct Parser {
    side: Side,
    complete: bool,
}

impl Parser {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            complete: false,
        }
    }

    /// Sets the parser complete state.
    ///
    /// If set to true, parser operates under the assumption that no more data is incoming
    pub fn set_complete(&mut self, complete: bool) {
        self.complete = complete;
    }

    /// Returns true if c is a line feed character
    fn is_eol(&self) -> impl Fn(u8) -> bool + '_ {
        move |c| c == b'\n' || (self.side == Side::Response && c == b'\r')
    }

    /// Parse one complete end of line character or character set
    fn complete_eol_regular(&self) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> + '_ {
        move |input| {
            if self.side == Side::Response {
                alt((
                    complete_tag("\r\n"),
                    complete_tag("\n\r"),
                    complete_tag("\n"),
                    complete_tag("\r"),
                ))(input)
            } else {
                alt((complete_tag("\r\n"), complete_tag("\n")))(input)
            }
        }
    }

    /// Parse one complete deformed end of line character set
    fn complete_eol_deformed(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            if self.side == Side::Response {
                alt((
                    map(
                        tuple((
                            complete_tag("\n\r\r\n"),
                            peek(alt((complete_tag("\n"), complete_tag("\r\n")))),
                        )),
                        |(eol, _)| (eol, Flags::DEFORMED_EOL),
                    ),
                    // Treat EOL + empty folding + EOL as just EOL
                    self.folding_empty(),
                    map(
                        tuple((
                            complete_tag("\r\n\r"),
                            take_while1(|c| c == b'\r' || c == b' ' || c == b'\t'),
                            opt(complete_tag("\n")),
                            not(alt((complete_tag("\n"), complete_tag("\r\n")))),
                        )),
                        |(eol1, eol2, eol3, _): (&[u8], &[u8], Option<&[u8]>, _)| {
                            (
                                &input[..(eol1.len() + eol2.len() + eol3.unwrap_or(b"").len())],
                                Flags::DEFORMED_EOL,
                            )
                        },
                    ),
                ))(input)
            } else {
                map(
                    alt((
                        tuple((
                            complete_tag("\n\r\r\n"),
                            peek(alt((complete_tag("\n"), complete_tag("\r\n")))),
                        )),
                        tuple((complete_tag("\n\r"), peek(complete_tag("\r\n")))),
                    )),
                    |(eol, _)| (eol, Flags::DEFORMED_EOL),
                )(input)
            }
        }
    }

    /// Parse one complete end of line character or character set
    fn complete_eol(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            alt((
                self.complete_eol_deformed(),
                map(self.complete_eol_regular(), |eol| (eol, 0)),
            ))(input)
        }
    }

    /// Parse one header end of line, and guarantee that it is not folding
    fn eol(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            map(
                tuple((self.complete_eol(), not(folding_lws))),
                |(end, _)| end,
            )(input)
        }
    }

    /// Parse one null byte or one end of line, and guarantee that it is not folding
    fn null_or_eol(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| alt((null, self.eol()))(input)
    }

    /// Parse one null byte or complete end of line
    fn complete_null_or_eol(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| alt((null, self.complete_eol()))(input)
    }

    /// Parse empty header folding as a single EOL (eol + whitespace + eol = eol)
    fn folding_empty(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            map(
                tuple((
                    self.complete_eol_regular(),
                    complete_space1,
                    self.complete_eol_regular(),
                )),
                |(eol1, spaces, eol2): SurroundedBytes| {
                    (
                        &input[..eol1.len() + spaces.len() + eol2.len()],
                        Flags::FOLDING_EMPTY,
                    )
                },
            )(input)
        }
    }
    /// Parse header folding bytes (eol + whitespace or eol + special cases)
    fn folding(&self) -> impl Fn(&[u8]) -> IResult<&[u8], FoldingBytes> + '_ {
        move |input| {
            if self.side == Side::Response {
                map(
                    tuple((
                        not(self.folding_empty()),
                        map(self.complete_eol_regular(), |eol| (eol, 0)),
                        folding_lws,
                    )),
                    |(_, (eol, flags), (folding_lws, other_flags))| {
                        (eol, folding_lws, flags | other_flags)
                    },
                )(input)
            } else {
                map(
                    tuple((self.complete_eol(), folding_lws)),
                    |((eol, flags), (folding_lws, other_flags))| {
                        (eol, folding_lws, flags | other_flags)
                    },
                )(input)
            }
        }
    }

    /// Special case check for end of headers with space or tab seperating the EOLs
    fn terminator_special_case(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            //Treat the empty folding as a single EOL when it is followed by another eol.
            alt((
                map(
                    tuple((self.folding_empty(), peek(self.complete_eol_regular()))),
                    |((eol, flags), _)| (eol, Flags::TERMINATOR_SPECIAL_CASE | flags),
                ),
                map(
                    tuple((
                        self.complete_eol_regular(),
                        space1,
                        peek(tuple((
                            self.complete_eol_regular(),
                            not(tuple((token_chars, separator_regular))),
                        ))),
                    )),
                    |(eol, space, _)| {
                        (
                            &input[..eol.len() + space.len()],
                            Flags::TERMINATOR_SPECIAL_CASE,
                        )
                    },
                ),
            ))(input)
        }
    }

    /// Parse complete folding bytes or a value terminator (eol or null)
    fn complete_folding_or_terminator(
        &self,
    ) -> impl Fn(&[u8]) -> IResult<&[u8], FoldingOrTerminator> + '_ {
        move |input| {
            if self.side == Side::Response {
                alt((
                    complete(map(self.terminator_special_case(), |result| (result, None))),
                    complete(map(self.folding(), |(end, fold, flags)| {
                        ((end, flags), Some(fold))
                    })),
                    map(self.complete_null_or_eol(), |end| (end, None)),
                ))(input)
            } else {
                alt((
                    complete(map(self.folding(), |(end, fold, flags)| {
                        ((end, flags), Some(fold))
                    })),
                    map(self.complete_null_or_eol(), |end| (end, None)),
                ))(input)
            }
        }
    }

    /// Parse complete folding bytes or a value terminator (eol or null)
    fn streaming_folding_or_terminator(
        &self,
    ) -> impl Fn(&[u8]) -> IResult<&[u8], FoldingOrTerminator> + '_ {
        move |input| {
            if self.side == Side::Response {
                alt((
                    map(self.terminator_special_case(), |result| (result, None)),
                    map(self.folding(), |(end, fold, flags)| {
                        ((end, flags), Some(fold))
                    }),
                    map(self.null_or_eol(), |end| (end, None)),
                ))(input)
            } else {
                alt((
                    map(self.folding(), |(end, fold, flags)| {
                        ((end, flags), Some(fold))
                    }),
                    map(self.null_or_eol(), |end| (end, None)),
                ))(input)
            }
        }
    }

    /// Parse folding bytes or a value terminator (eol or null)
    fn folding_or_terminator(&self) -> impl Fn(&[u8]) -> IResult<&[u8], FoldingOrTerminator> + '_ {
        move |input| {
            if self.complete {
                self.complete_folding_or_terminator()(input)
            } else {
                self.streaming_folding_or_terminator()(input)
            }
        }
    }

    /// Parse a header value.
    /// Returns the bytes and the value terminator; null, eol or folding
    /// eg. (bytes, (eol_bytes, Option<fold_bytes>))
    fn value_bytes(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ValueBytes> + '_ {
        move |input| {
            let (mut remaining, mut value) = take_till(self.is_eol())(input)?;
            if value.last() == Some(&b'\r') {
                value = &value[..value.len() - 1];
                remaining = &input[value.len()..];
            }
            let (remaining, result) = self.folding_or_terminator()(remaining)?;
            Ok((remaining, (value, result)))
        }
    }

    /// Parse a complete header value, including any folded headers
    fn value(&self) -> impl Fn(&[u8]) -> IResult<&[u8], Value> + '_ {
        move |input| {
            let (rest, (val_bytes, ((_eol, mut flags), fold))) = self.value_bytes()(input)?;
            let mut value = val_bytes.to_vec();
            if fold.is_some() {
                let mut i = rest;
                loop {
                    if self.side == Side::Response {
                        // Peek ahead for ambiguous name with lws vs. value with folding
                        match tuple((token_chars, separator_regular))(i) {
                            Ok(_) => {
                                flags.unset(Flags::FOLDING_SPECIAL_CASE);
                                if value.is_empty() {
                                    flags.set(Flags::VALUE_EMPTY);
                                }
                                return Ok((rest, Value::new(&value, flags)));
                            }
                            Err(Incomplete(_)) => {
                                return Err(Incomplete(Needed::new(1)));
                            }
                            _ => {}
                        }
                    }
                    let (rest, (val_bytes, ((_eol, other_flags), fold))) = self.value_bytes()(i)?;
                    i = rest;
                    flags.set(other_flags);
                    //If the value is empty, the value started with a fold and we don't want to push back a space
                    if !value.is_empty() {
                        value.push(b' ');
                    }
                    value.extend(val_bytes);
                    if fold.is_none() {
                        return Ok((rest, Value::new(&value, flags)));
                    }
                }
            } else {
                if value.is_empty() {
                    flags.set(Flags::VALUE_EMPTY);
                }
                Ok((rest, Value::new(&value, flags)))
            }
        }
    }

    /// Parse one header name, incluing the : and trailing whitespace
    fn name(&self) -> impl Fn(&[u8]) -> IResult<&[u8], Name> + '_ {
        move |input| {
            //We first attempt to parse a token name before we attempt a non token name
            map(
                alt((self.token_name(), self.non_token_name())),
                |(name, flags)| Name::new(name, flags),
            )(input)
        }
    }

    /// Parse name containing non token characters with either regular separator or deformed separator
    fn non_token_name(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            map(
                alt((
                    tuple((
                        take_till(|c| c == b':' || self.is_terminator(c) || c == b'\r'),
                        peek(self.separator()),
                    )),
                    tuple((
                        take_till(|c| c == b':' || self.is_terminator(c)),
                        peek(self.separator()),
                    )),
                )),
                |(name, _): (&[u8], _)| {
                    let mut flags = Flags::NAME_NON_TOKEN_CHARS;
                    if !name.is_empty() {
                        if is_space(name[0]) {
                            flags.set(Flags::NAME_LEADING_WHITESPACE)
                        }
                        if let Some(end) = name.last() {
                            if is_space(*end) {
                                flags.set(Flags::NAME_TRAILING_WHITESPACE);
                            }
                        }
                    } else {
                        flags.set(Flags::NAME_EMPTY)
                    }
                    (name, flags)
                },
            )(input)
        }
    }

    /// Check if the byte is a line ending character
    fn is_terminator(&self, c: u8) -> bool {
        c == b'\n'
    }

    /// Handles accepted deformed separators
    fn separator_deformed(&self) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> + '_ {
        move |input| {
            map(
                tuple((
                    not(tuple((self.complete_eol(), self.complete_eol()))),
                    alt((
                        map(
                            tuple((
                                take_while1(is_special_whitespace),
                                complete_tag(":"),
                                space0,
                                not(tuple((self.complete_eol(), self.complete_eol()))),
                                take_while(is_special_whitespace),
                            )),
                            |(_, tagged, _, _, _)| tagged,
                        ),
                        map(
                            tuple((
                                take_while(is_special_whitespace),
                                complete_tag(":"),
                                space0,
                                not(tuple((self.complete_eol(), self.complete_eol()))),
                                take_while1(is_special_whitespace),
                            )),
                            |(_, tagged, _, _, _)| tagged,
                        ),
                    )),
                )),
                |(_, sep)| sep,
            )(input)
        }
    }

    /// Parse a separator between header name and value
    fn separator(&self) -> impl Fn(&[u8]) -> IResult<&[u8], u64> + '_ {
        move |input| {
            if self.side == Side::Response {
                alt((
                    map(self.separator_deformed(), |_| Flags::DEFORMED_SEPARATOR),
                    map(separator_regular, |_| 0),
                ))(input)
            } else {
                map(separator_regular, |_| 0)(input)
            }
        }
    }

    /// Parse name containing only token characters
    fn token_name(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedBytes> + '_ {
        move |input| {
            // The name should consist only of token characters (i.e., no spaces, seperators, control characters, etc)
            map(
                tuple((token_chars, peek(self.separator()))),
                |((leading_spaces, name, trailing_spaces), _): (SurroundedBytes, _)| {
                    let mut flags = 0;
                    if !name.is_empty() {
                        if !leading_spaces.is_empty() {
                            flags.set(Flags::NAME_LEADING_WHITESPACE)
                        }
                        if !trailing_spaces.is_empty() {
                            flags.set(Flags::NAME_TRAILING_WHITESPACE)
                        }
                    } else {
                        flags.set(Flags::NAME_EMPTY)
                    }
                    let slice_len = leading_spaces.len() + name.len() + trailing_spaces.len();
                    (&input[..slice_len], flags)
                },
            )(input)
        }
    }

    /// Parse data before an eol with no colon as an empty name with the data as the value
    fn header_sans_colon(&self) -> impl Fn(&[u8]) -> IResult<&[u8], Header> + '_ {
        move |input| {
            let (mut remaining, (_, mut value)) = tuple((
                not(complete_tag("\r\n")),
                take_till1(|c| c == b':' || self.is_terminator(c)),
            ))(input)?;
            if value.last() == Some(&b'\r') {
                value = &value[..value.len() - 1];
                remaining = &input[value.len()..];
            }
            let (remaining, (_, flags)) = self.complete_null_or_eol()(remaining)?;
            Ok((
                remaining,
                Header::new_with_flags(
                    b"",
                    Flags::MISSING_COLON | flags,
                    value,
                    Flags::MISSING_COLON | flags,
                ),
            ))
        }
    }

    /// Parse a header name separator value
    fn header_with_colon(&self) -> impl Fn(&[u8]) -> IResult<&[u8], Header> + '_ {
        move |input| {
            map(
                tuple((self.name(), self.separator(), self.value())),
                |(mut name, flag, mut value)| {
                    name.flags |= flag;
                    value.flags |= flag;
                    Header::new(name, value)
                },
            )(input)
        }
    }

    /// Parses a header name and value with, or without a colon separator
    fn header(&self) -> impl Fn(&[u8]) -> IResult<&[u8], Header> + '_ {
        move |input| alt((self.header_with_colon(), self.header_sans_colon()))(input)
    }

    /// Parse multiple headers and indicate if end of headers or null was found
    pub fn headers(&self) -> impl Fn(&[u8]) -> IResult<&[u8], ParsedHeaders> + '_ {
        move |input| {
            let (rest, head) = self.header()(input)?;
            let is_null_terminated = head.value.flags.is_set(Flags::NULL_TERMINATED);
            let mut out = Vec::with_capacity(16);
            out.push(head);
            if is_null_terminated {
                return Ok((rest, (out, true)));
            }
            if let Ok((rest, _eoh)) = self.complete_eol()(rest) {
                return Ok((rest, (out, true)));
            }
            let mut i = rest;
            loop {
                match self.header()(i) {
                    Ok((rest, head)) => {
                        i = rest;
                        let is_null_terminated = head.value.flags.is_set(Flags::NULL_TERMINATED);
                        out.push(head);
                        if is_null_terminated {
                            return Ok((rest, (out, true)));
                        }
                        if let Ok((rest, _eoh)) = self.complete_eol()(rest) {
                            return Ok((rest, (out, true)));
                        }
                    }
                    Err(Incomplete(_)) => {
                        return Ok((i, (out, false)));
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

/// Parse one null character and return it and the NULL_TERMINATED flag
fn null(input: &[u8]) -> IResult<&[u8], ParsedBytes> {
    map(complete_tag("\0"), |null| (null, Flags::NULL_TERMINATED))(input)
}

/// Handles any special cases that are exceptions to the spec
///
/// Currently handles the use of a single CR as folding LWS
fn folding_lws_special(input: &[u8]) -> IResult<&[u8], &[u8]> {
    map(
        tuple((tag("\r"), not(alt((tag("\r"), tag("\n")))), space0)),
        |(fold, _, spaces): (&[u8], _, &[u8])| &input[..fold.len() + spaces.len()],
    )(input)
}

/// Extracts any folding lws (whitespace or any special cases)
fn folding_lws(input: &[u8]) -> IResult<&[u8], ParsedBytes> {
    alt((
        map(space1, |fold| (fold, Flags::FOLDING)),
        map(folding_lws_special, |fold| {
            (fold, Flags::FOLDING_SPECIAL_CASE)
        }),
    ))(input)
}

/// Parse a regular separator (colon followed by optional spaces) between header name and value
fn separator_regular(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    tuple((complete_tag(":"), space0))(input)
}

type leading_token_trailing<'a> = (&'a [u8], &'a [u8], &'a [u8]);
/// Parse token characters with leading and trailing whitespace
fn token_chars(input: &[u8]) -> IResult<&[u8], leading_token_trailing> {
    tuple((space0, take_while(is_token), space0))(input)
}

/// Check if the input is a space, HT, VT, CR, LF, or FF
fn is_special_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == b'\x0b' || c == b'\x0c'
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::NomError;
    use nom::{
        error::ErrorKind::{Not, Tag},
        Err::{Error, Incomplete},
        Needed,
    };
    use rstest::rstest;
    macro_rules! b {
        ($b: literal) => {
            $b.as_bytes()
        };
    }

    #[rstest]
    #[case::null_does_not_terminate(b"k1:v1\r\nk2:v2 before\0v2 after\r\n\r\n",Ok((b!(""), (vec![Header::new_with_flags(b"k1", 0, b"v1", 0), Header::new_with_flags(b"k2", 0, b"v2 before\0v2 after", 0)], true))), None)]
    #[case::flags(b"k1:v1\r\n:v2\r\n v2+\r\nk3: v3\r\nk4 v4\r\nk\r5:v\r5\n\rmore\r\n\r\n", Ok((b!(""), (
            vec![
                Header::new_with_flags(b"k1", 0, b"v1", 0),
                Header::new_with_flags(b"", Flags::NAME_EMPTY, b"v2 v2+", Flags::FOLDING),
                Header::new_with_flags(b"k3", 0, b"v3", 0),
                Header::new_with_flags(b"", Flags::MISSING_COLON, b"k4 v4", Flags::MISSING_COLON),
                Header::new_with_flags(b"k\r5", Flags::NAME_NON_TOKEN_CHARS, b"v\r5 more", Flags::FOLDING_SPECIAL_CASE)
                ], true))), Some(Ok((b!(""), (
            vec![
                Header::new_with_flags(b"k1", 0, b"v1", 0),
                Header::new_with_flags(b"", Flags::NAME_EMPTY, b"v2 v2+", Flags::FOLDING),
                Header::new_with_flags(b"k3", 0, b"v3", 0),
                Header::new_with_flags(b"", Flags::MISSING_COLON, b"k4 v4", Flags::MISSING_COLON),
                Header::new_with_flags(b"k\r5", Flags::NAME_NON_TOKEN_CHARS, b"v", 0),
                Header::new_with_flags(b"", Flags::MISSING_COLON, b"5", Flags::MISSING_COLON),
                Header::new_with_flags(b"", Flags::MISSING_COLON, b"more", Flags::MISSING_COLON)
                ], true)))))]
    #[case::incomplete_eoh(b"k1:v1\r\nk2:v2\r", Ok((b!("k2:v2\r"), (vec![Header::new_with_flags(b"k1", 0, b"v1", 0)], false))), None)]
    #[case::incomplete_eoh_null(b"k1:v1\nk2:v2\0v2\r\nk3:v3\r", Ok((b!("k3:v3\r"), (vec![Header::new_with_flags(b"k1", 0, b"v1", 0), Header::new_with_flags(b"k2", 0, b"v2\0v2", 0)], false))), None)]
    fn test_headers(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], ParsedHeaders>,
        #[case] diff_res_expected: Option<IResult<&[u8], ParsedHeaders>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.headers()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.headers()(input), res_expected);
        } else {
            assert_eq!(res_parser.headers()(input), expected);
        }
    }

    #[rstest]
    #[case::only_lf_eoh(
        b"Name1: Value1\nName2:Value2\nName3: Val\n ue3\nName4: Value4\n Value4.1\n Value4.2\n\n",
        None
    )]
    #[case::only_crlf_eoh(b"Name1: Value1\r\nName2:Value2\r\nName3: Val\r\n ue3\r\nName4: Value4\r\n Value4.1\r\n Value4.2\r\n\r\n", None)]
    #[case::crlf_lf_eoh(b"Name1: Value1\r\nName2:Value2\nName3: Val\r\n ue3\r\nName4: Value4\r\n Value4.1\n Value4.2\r\n\n", None)]
    #[case::only_cr(b"Name1: Value1\rName2:Value2\rName3: Val\r\n ue3\rName4: Value4\r\n Value4.1\r\n Value4.2\r\r\n", Some(Err(Incomplete(Needed::new(1)))))]
    #[case::cr_lf_crlf_eoh(b"Name1: Value1\rName2:Value2\rName3: Val\r\n ue3\r\nName4: Value4\r\n Value4.1\n Value4.2\r\n\n", Some(Ok((b!(""),
        (
            vec![
                Header::new_with_flags(b"Name1", 0, b"Value1\rName2:Value2\rName3: Val ue3", Flags::FOLDING),
                Header::new_with_flags(b"Name4", 0, b"Value4 Value4.1 Value4.2", Flags::FOLDING)
                ],
                true
        )))))]
    #[case::crlf_lfcr_lf(b"Name1: Value1\r\nName2:Value2\nName3: Val\n\r ue3\n\rName4: Value4\r\n Value4.1\n Value4.2\r\n\n", Some(Ok((b!(""),
        (
            vec![
                Header::new_with_flags(b"Name1", 0, b"Value1", 0),
                Header::new_with_flags(b"Name2", 0, b"Value2", 0),
                Header::new_with_flags(b"Name3", 0, b"Val ue3 Name4: Value4 Value4.1 Value4.2", Flags::FOLDING_SPECIAL_CASE),
                ],
                true
        )))))]
    #[case::lfcr_eoh(b"Name1: Value1\n\rName2:Value2\n\rName3: Val\n\r ue3\n\rName4: Value4\n\r Value4.1\n\r Value4.2\n\r\n\r", Some(Ok((b!("\r"),
        (
            vec![Header::new_with_flags(b"Name1", 0, b"Value1 Name2:Value2 Name3: Val ue3 Name4: Value4 Value4.1 Value4.2", Flags::FOLDING_SPECIAL_CASE)],
            true
        )))))]
    fn test_headers_eoh(
        #[case] input: &[u8],
        #[case] diff_req_expected: Option<IResult<&[u8], ParsedHeaders>>,
    ) {
        let expected = Ok((
            b!(""),
            (
                vec![
                    Header::new_with_flags(b"Name1", 0, b"Value1", 0),
                    Header::new_with_flags(b"Name2", 0, b"Value2", 0),
                    Header::new_with_flags(b"Name3", 0, b"Val ue3", Flags::FOLDING),
                    Header::new_with_flags(
                        b"Name4",
                        0,
                        b"Value4 Value4.1 Value4.2",
                        Flags::FOLDING,
                    ),
                ],
                true,
            ),
        ));
        let req_parser = Parser::new(Side::Request);
        let res_parser = Parser::new(Side::Response);
        if let Some(req_expected) = diff_req_expected {
            assert_eq!(req_parser.headers()(input), req_expected);
        } else {
            assert_eq!(req_parser.headers()(input), expected);
        }
        assert_eq!(res_parser.headers()(input), expected);
    }

    #[rstest]
    #[case::incomplete(b"K V", Err(Incomplete(Needed::new(1))))]
    #[case::contains_colon(b"K:V\r\n", Err(Error(NomError::new(b!(":V\r\n"), Tag))))]
    #[case::empty_name_value(b"\r\n", Err(Error(NomError::new(b!("\r\n"), Not))))]
    #[case::contains_null(b"K V\0alue\r\n", Ok((b!(""), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K V\0alue", Flags::MISSING_COLON))))]
    #[case::folding(b"K V\ralue\r\n", Ok((b!(""), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K V\ralue", Flags::MISSING_COLON))))]
    #[case::crlf(b"K V\r\nk1:v1\r\n", Ok((b!("k1:v1\r\n"), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K V", Flags::MISSING_COLON))))]
    #[case::lf(b"K V\nk1:v1\r\n", Ok((b!("k1:v1\r\n"), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K V", Flags::MISSING_COLON))))]
    fn test_header_sans_colon(#[case] input: &[u8], #[case] expected: IResult<&[u8], Header>) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.header_sans_colon()(input), expected);

        let res_parser = Parser::new(Side::Response);
        assert_eq!(res_parser.header_sans_colon()(input), expected);
    }

    #[rstest]
    #[case::incomplete(b"K: V", Err(Incomplete(Needed::new(1))))]
    #[case::contains_colon(b"K: V\r\n", Err(Incomplete(Needed::new(1))))]
    #[case::missing_colon(b"K V\r\nK:V\r\n", Err(Error(NomError::new(b!("\nK:V\r\n"), Tag))))]
    #[case::contains_null(b":\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"", Flags::NAME_EMPTY, b"", Flags::VALUE_EMPTY))))]
    #[case::folding(b"K:\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"", Flags::VALUE_EMPTY))))]
    #[case::crlf(b":V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"", Flags::NAME_EMPTY, b"V", 0))))]
    #[case::lf(b"K:folded\r\n\rV\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"folded V", Flags::FOLDING_SPECIAL_CASE))))]
    #[case::lf(b"K: V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V", 0))))]
    #[case::lf(b"K: V before\0 V after\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V before\0 V after", 0))))]
    #[case::lf(b"K: V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V", 0))))]
    #[case::lf(b"K: V before\0 V after\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V before\0 V after", 0))))]
    #[case::lf(b"K: V\r\n a\r\n l\r\n u\r\n\te\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V a l u e", Flags::FOLDING))))]
    fn test_header_with_colon(#[case] input: &[u8], #[case] expected: IResult<&[u8], Header>) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.header_with_colon()(input), expected);

        let res_parser = Parser::new(Side::Response);
        assert_eq!(res_parser.header_with_colon()(input), expected);
    }

    #[rstest]
    #[case::incomplete(b"K: V", Err(Incomplete(Needed::new(1))), None)]
    #[case::contains_colon(b"K: V\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::missing_colon(b"K V\r\n", Ok((b!(""), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K V", Flags::MISSING_COLON))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::missing_colon(b"K1 V1\r\nK2:V2\n\r\n", Ok((b!("K2:V2\n\r\n"), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K1 V1", Flags::MISSING_COLON))), None)]
    #[case::empty_name_value(b"K1:V1\nK2:V2\n\r\n", Ok((b!("K2:V2\n\r\n"), Header::new_with_flags(b"K1", 0, b"V1", 0))), None)]
    #[case::contains_null(b":\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"", Flags::NAME_EMPTY, b"", Flags::VALUE_EMPTY))), None)]
    #[case::folding(b"K:\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"", Flags::VALUE_EMPTY))), None)]
    #[case::empty_name(b":V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"", Flags::NAME_EMPTY, b"V", 0))), None)]
    #[case::special_folding(b"K:folded\r\n\rV\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"folded V", Flags::FOLDING_SPECIAL_CASE))), None)]
    #[case(b"K: V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V", 0))), None)]
    #[case::folding(b"K: V\n a\r\n l\n u\r\n\te\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V a l u e", Flags::FOLDING))), None)]
    #[case(b"Host:www.google.com\rName: Value\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"Host", 0, b"www.google.com\rName: Value", 0))), Some(Ok((b!("Name: Value\r\n\r\n"), Header::new_with_flags(b"Host", 0, b"www.google.com", 0)))))]
    #[case(b"K: V before\0 V after\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V before\0 V after", 0))), None)]
    #[case::folding(b"K: V\r a\r\n l\n u\r\n\te\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V\r a l u e", Flags::FOLDING))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"V a l u e", Flags::FOLDING)))))]
    #[case::deformed_folding(b"K:deformed folded\n\r V\n\r\r\n\n", Ok((b!("\n"), Header::new_with_flags(b"K", 0, b"deformed folded V", Flags::FOLDING_SPECIAL_CASE | Flags::DEFORMED_EOL))), Some(Ok((b!("\n"), Header::new_with_flags(b"K", 0, b"deformed folded V", Flags::FOLDING | Flags::DEFORMED_EOL)))))]
    #[case::deformed_folding(b"K:deformed folded\n\r V\r\n\r\n", Ok(( b!("\r\n"), Header::new_with_flags(b"K", 0, b"deformed folded V", Flags::FOLDING_SPECIAL_CASE))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"deformed folded V", Flags::FOLDING)))))]
    #[case::deformed_folding(b"K:deformed folded\n\r\r V\r\n\r\n", Ok(( b!("\r\r V\r\n\r\n"), Header::new_with_flags(b"K", 0, b"deformed folded", 0))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", 0, b"deformed folded V", Flags::FOLDING_SPECIAL_CASE)))))]
    #[case::non_token_trailing_ws(b"K\r \r :\r V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K\r \r ", Flags::NAME_NON_TOKEN_CHARS | Flags::NAME_TRAILING_WHITESPACE, b"\r V", 0))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"V", Flags::DEFORMED_SEPARATOR)))))]
    #[case::deformed_sep(b"K\n\r \r\n :\r\n V\r\n\r\n", Ok((b!("\r \r\n :\r\n V\r\n\r\n"), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K", Flags::MISSING_COLON))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"V", Flags::DEFORMED_SEPARATOR)))))]
    #[case::deformed_sep(b"K\r\n \r\n :\r\n V\r\n\r\n", Ok((b!(" \r\n :\r\n V\r\n\r\n"), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K", Flags::MISSING_COLON))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"V", Flags::DEFORMED_SEPARATOR)))))]
    #[case::empty_value_deformed(b"K:\r\n\0Value\r\n V\r\n\r\n", Ok((b!("\0Value\r\n V\r\n\r\n"), Header::new_with_flags(b"K", 0, b"", Flags::VALUE_EMPTY))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"\0Value V", Flags::DEFORMED_SEPARATOR | Flags::FOLDING)))))]
    #[case::missing_colon(b"K\r\n:Value\r\n V\r\n\r\n", Ok((b!(":Value\r\n V\r\n\r\n"), Header::new_with_flags(b"", Flags::MISSING_COLON, b"K", Flags::MISSING_COLON))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"Value V", Flags::DEFORMED_SEPARATOR | Flags::FOLDING)))))]
    #[case::non_token(b"K\x0c:Value\r\n V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K\x0c", Flags::NAME_NON_TOKEN_CHARS, b"Value V", Flags::FOLDING))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"Value V", Flags::DEFORMED_SEPARATOR | Flags::FOLDING)))))]
    #[case::non_token_trailing(b"K\r :Value\r\n V\r\n\r\n", Ok((b!("\r\n"), Header::new_with_flags(b"K\r ", Flags::NAME_TRAILING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS, b"Value V", Flags::FOLDING))), Some(Ok((b!("\r\n"), Header::new_with_flags(b"K", Flags::DEFORMED_SEPARATOR, b"Value V", Flags::DEFORMED_SEPARATOR | Flags::FOLDING)))))]
    fn test_header(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], Header>,
        #[case] diff_res_expected: Option<IResult<&[u8], Header>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.header()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.header()(input), res_expected);
        } else {
            assert_eq!(res_parser.header()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b" : ", Err(Error(NomError::new(b!(" : "), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::incomplete(b" ", Err(Error(NomError::new(b!(" "), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::colon(b":value", Ok((b!("value"), 0)), None)]
    #[case::colon_whitespace(b": value", Ok((b!("value"), 0)), None)]
    #[case::colon_tab(b":\t value", Ok((b!("value"), 0)), None)]
    #[case::deformed_sep(b"\r\n \n:\t\r\n value", Err(Error(NomError::new(b!("\r\n \n:\t\r\n value"), Tag))), Some(Ok((b!("value"), Flags::DEFORMED_SEPARATOR))))]
    #[case::deformed_sep(b"\x0c:\t value", Err(Error(NomError::new(b!("\x0c:\t value"), Tag))), Some(Ok((b!("value"), Flags::DEFORMED_SEPARATOR))))]
    #[case::deformed_sep(b"\r: value", Err(Error(NomError::new(b!("\r: value"), Tag))), Some(Ok((b!("value"), Flags::DEFORMED_SEPARATOR))))]
    fn test_separators(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], u64>,
        #[case] diff_res_expected: Option<IResult<&[u8], u64>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.separator()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.separator()(input), res_expected);
        } else {
            assert_eq!(res_parser.separator()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b"name", Err(Incomplete(Needed::new(1))))]
    #[case::token(b"name:", Ok((b!(":"), (b!(""), b!("name"), b!("")))))]
    #[case::trailing_whitespace(b"name :", Ok((b!(":"), (b!(""), b!("name"), b!(" ")))))]
    #[case::surrounding_whitespace(b" name :", Ok((b!(":"), (b!(" "), b!("name"), b!(" ")))))]
    fn test_token_chars(#[case] input: &[u8], #[case] expected: IResult<&[u8], SurroundedBytes>) {
        assert_eq!(token_chars(input), expected);
    }

    #[rstest]
    #[case::incomplete(b"Hello", Err(Incomplete(Needed::new(1))), None)]
    #[case::name(b"Hello: world", Ok((b!(": world"), (b!("Hello"), 0))), None)]
    #[case::leading_whitespace(b" Hello: world", Ok((b!(": world"), (b!(" Hello"), Flags::NAME_LEADING_WHITESPACE))), None)]
    #[case::trailing_whitespace(b"Hello : world", Ok((b!(": world"), (b!("Hello "), Flags::NAME_TRAILING_WHITESPACE))), None)]
    #[case::surrounding_whitespace(b" Hello : world", Ok((b!(": world"), (b!(" Hello "), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE))), None)]
    #[case::surrounding_whitespace_response(b" Hello \r\n \n:\n world", Err(Error(NomError::new(b!("\r\n \n:\n world"), Tag))), Some(Ok((b!("\r\n \n:\n world"), (b!(" Hello "), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE)))))]
    #[case::surrounding_whitespace_response(b" Hello \n\r \n:\n world", Err(Error(NomError::new(b!("\n\r \n:\n world"), Tag))), Some(Ok((b!("\n\r \n:\n world"), (b!(" Hello "), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE)))))]
    #[case::invalid_space(b"Hello Invalid: world", Err(Error(NomError::new(b!("Invalid: world"), Tag))), None)]
    #[case::invalid_semicolon(b"Hello;Invalid: world", Err(Error(NomError::new(b!(";Invalid: world"), Tag))), None)]
    #[case::invalid_cr(b"Hello\rInvalid: world", Err(Error(NomError::new(b!("\rInvalid: world"), Tag))), None)]
    #[case::invalid_lf(b"Hello\nInvalid: world", Err(Error(NomError::new(b!("\nInvalid: world"), Tag))), None)]
    #[case::invalid_null(b"Hello\0Invalid: world", Err(Error(NomError::new(b!("\0Invalid: world"), Tag))), None)]
    fn test_token_name(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], ParsedBytes>,
        #[case] diff_res_expected: Option<IResult<&[u8], ParsedBytes>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.token_name()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.token_name()(input), res_expected);
        } else {
            assert_eq!(res_parser.token_name()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b"Hello", Err(Incomplete(Needed::new(1))), None)]
    #[case::name(b"Hello: world", Ok((b!(": world"), (b!("Hello"), Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::leading_whitespace(b" Hello: world", Ok((b!(": world"), (b!(" Hello"), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::trailing_whitespace(b"Hello : world", Ok((b!(": world"), (b!("Hello "), Flags::NAME_TRAILING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::surrounding_whitespace(b" Hello : world", Ok((b!(": world"), (b!(" Hello "), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::surrounding_whitespace_response(b" Hello \r\n \n:\n world", Err(Error(NomError::new(b!("\n \n:\n world"), Tag))), Some(Ok((b!("\r\n \n:\n world"), (b!(" Hello "), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS)))))]
    #[case::surrounding_whitespace_response(b" Hello \n\r \n:\n world", Err(Error(NomError::new(b!("\n\r \n:\n world"), Tag))), Some(Ok((b!("\n\r \n:\n world"), (b!(" Hello "), Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS)))))]
    #[case::space(b"Hello Invalid: world", Ok((b!(": world"), (b!("Hello Invalid"), Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::semicolon(b"Hello;Invalid: world", Ok((b!(": world"), (b!("Hello;Invalid"), Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::invalid_cr(b"Hello\rInvalid: world", Ok((b!(": world"), (b!("Hello\rInvalid"), Flags::NAME_NON_TOKEN_CHARS))), None)]
    #[case::invalid_lf(b"Hello\nInvalid: world", Err(Error(NomError::new(b!("\nInvalid: world"), Tag))), None)]
    #[case::invalid_null(b"Hello\0Invalid: world", Ok((b!(": world"), (b!("Hello\0Invalid"), Flags::NAME_NON_TOKEN_CHARS))), None)]
    fn test_non_token_name(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], ParsedBytes>,
        #[case] diff_res_expected: Option<IResult<&[u8], ParsedBytes>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.non_token_name()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.non_token_name()(input), res_expected);
        } else {
            assert_eq!(res_parser.non_token_name()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b"Hello", Err(Incomplete(Needed::new(1))), None)]
    #[case::name(b"Hello: world", Ok((b!(": world"), Name {name: b"Hello".to_vec(), flags: 0})), None)]
    #[case::name(b"Host:www.google.com\rName: Value", Ok((b!(":www.google.com\rName: Value"), Name {name: b"Host".to_vec(), flags: 0})), None)]
    #[case::trailing_whitespace(b"Hello : world", Ok((b!(": world"), Name {name: b"Hello".to_vec(), flags: Flags::NAME_TRAILING_WHITESPACE})), None)]
    #[case::surrounding_whitespace(b" Hello : world", Ok((b!(": world"), Name {name: b"Hello".to_vec(), flags: Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE})), None)]
    #[case::semicolon(b"Hello;invalid: world", Ok((b!(": world"), Name {name: b"Hello;invalid".to_vec(), flags: Flags::NAME_NON_TOKEN_CHARS})), None)]
    #[case::space(b"Hello invalid: world", Ok((b!(": world"), Name {name: b"Hello invalid".to_vec(), flags: Flags::NAME_NON_TOKEN_CHARS})), None)]
    #[case::surrounding_internal_space(b" Hello invalid : world", Ok((b!(": world"), Name {name: b"Hello invalid".to_vec(), flags: Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE | Flags::NAME_NON_TOKEN_CHARS})), None)]
    #[case::empty_name(b"   : world", Ok((b!(": world"), Name {name: b"".to_vec(), flags: Flags::NAME_EMPTY})), None)]
    #[case::empty_name_response(b"\r\n \r\n:\r\n world", Err(Error(NomError::new(b!("\n \r\n:\r\n world"), Tag))), Some(Ok((b!("\r\n \r\n:\r\n world"), Name {name: b"".to_vec(), flags : Flags::NAME_EMPTY}))))]
    #[case::surrounding_whitespace_response(b" Hello \r\n \n: \nworld", Err(Error(NomError::new(b!("\n \n: \nworld"), Tag))), Some(Ok((b!("\r\n \n: \nworld"), Name {name: b"Hello".to_vec(), flags: Flags::NAME_LEADING_WHITESPACE | Flags::NAME_TRAILING_WHITESPACE}))))]
    fn test_name(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], Name>,
        #[case] diff_res_expected: Option<IResult<&[u8], Name>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.name()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.name()(input), res_expected);
        } else {
            assert_eq!(res_parser.name()(input), expected);
        }
    }

    #[rstest]
    #[case(b"test", Err(Error(NomError::new(b!("test"), Tag))))]
    #[case(b"\r\n", Err(Error(NomError::new(b!("\r\n"), Tag))))]
    #[case(b"\n", Err(Error(NomError::new(b!("\n"), Tag))))]
    #[case(b"\0a", Ok((b!("a"), (b!("\0"), Flags::NULL_TERMINATED))))]
    fn test_null(#[case] input: &[u8], #[case] expected: IResult<&[u8], ParsedBytes>) {
        assert_eq!(null(input), expected);
    }

    #[rstest]
    #[case::not_eol(b"test", Err(Error(NomError::new(b!("test"), Tag))), None)]
    #[case::incomplete_eol(b"\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_eol(b"\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_eol(b"\r\n\t", Err(Incomplete(Needed::new(1))), None)]
    #[case::complete_cr(b"\ra", Err(Error(NomError::new(b!("\ra"), Tag))), Some(Ok((b!("a"), (b!("\r"), 0)))))]
    #[case::incomplete_crcr(b"\r\r", Err(Error(NomError::new(b!("\r\r"), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::incomplete_lfcr(b"\n\r", Err(Incomplete(Needed::new(1))), None)]
    #[case::complete_lfcr(b"\n\ra", Err(Error(NomError::new(b!("\ra"), Not))), Some(Ok((b!("a"), (b!("\n\r"), 0)))))]
    #[case::lfcrlf(b"\n\r\n", Ok((b!("\r\n"), (b!("\n"), 0))), Some(Ok((b!("\n"), (b!("\n\r"), 0)))))]
    #[case::lfcrlfcr(b"\n\r\n\r", Ok((b!("\r\n\r"), (b!("\n"), 0))), Some(Ok((b!("\n\r"), (b!("\n\r"), 0)))))]
    #[case::complete_lf(b"\na", Ok((b!("a"), (b!("\n"), 0))), None)]
    #[case::complete_lfcrcrlf(b"\n\r\r\na", Ok((b!("\r\na"), (b!("\n\r"), Flags::DEFORMED_EOL))), Some(Ok((b!("\r\na"), (b!("\n\r"), 0)))))]
    #[case::complete_crlfcrlf(b"\r\n\r\na", Ok((b!("\r\na"), (b!("\r\n"), 0))), None)]
    #[case::incomplete_crlf(b"\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_lf(b"\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::lfcrcrlf(b"\n\r\r\n", Ok((b!("\r\n"), (b!("\n\r"), Flags::DEFORMED_EOL))), Some(Ok((b!("\r\n"), (b!("\n\r"), 0)))))]
    #[case::crlfcrlf(b"\r\n\r\n", Ok((b!("\r\n"), (b!("\r\n"), 0))), None)]
    #[case::null(b"\0a", Err(Error(NomError::new(b!("\0a"), Tag))), None)]
    fn test_eol(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], ParsedBytes>,
        #[case] diff_res_expected: Option<IResult<&[u8], ParsedBytes>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.eol()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.eol()(input), res_expected);
        } else {
            assert_eq!(res_parser.eol()(input), expected);
        }
    }

    #[rstest]
    #[case::not_eol(b"test", Err(Error(NomError::new(b!("test"), Tag))), None)]
    #[case::incomplete_eol(b"\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_eol(b"\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_eol(b"\r\n\t", Err(Incomplete(Needed::new(1))), None)]
    #[case::complete_cr(b"\ra", Err(Error(NomError::new(b!("\ra"), Tag))), Some(Ok((b!("a"), (b!("\r"), 0)))))]
    #[case::incomplete_crcr(b"\r\r", Err(Error(NomError::new(b!("\r\r"), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::incomplete_lfcr(b"\n\r", Err(Incomplete(Needed::new(1))), None)]
    #[case::complete_lfcr(b"\n\ra", Err(Error(NomError::new(b!("\ra"), Not))), Some(Ok((b!("a"), (b!("\n\r"), 0)))))]
    #[case::lfcrlf(b"\n\r\n", Ok((b!("\r\n"), (b!("\n"), 0))), Some(Ok((b!("\n"), (b!("\n\r"), 0)))))]
    #[case::lfcrlfcr(b"\n\r\n\r", Ok((b!("\r\n\r"), (b!("\n"), 0))), Some(Ok((b!("\n\r"), (b!("\n\r"), 0)))))]
    #[case::complete_lf(b"\na", Ok((b!("a"), (b!("\n"), 0))), None)]
    #[case::complete_lfcrcrlf(b"\n\r\r\na", Ok((b!("\r\na"), (b!("\n\r"), Flags::DEFORMED_EOL))), Some(Ok((b!("\r\na"), (b!("\n\r"), 0)))))]
    #[case::complete_crlfcrlf(b"\r\n\r\na", Ok((b!("\r\na"), (b!("\r\n"), 0))), None)]
    #[case::incomplete_crlf(b"\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_lf(b"\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::lfcrcrlf(b"\n\r\r\n", Ok((b!("\r\n"), (b!("\n\r"), Flags::DEFORMED_EOL))), Some(Ok((b!("\r\n"), (b!("\n\r"), 0)))))]
    #[case::crlfcrlf(b"\r\n\r\n", Ok((b!("\r\n"), (b!("\r\n"), 0))), None)]
    #[case::null(b"\0a", Ok((b!("a"), (b!("\0"), Flags::NULL_TERMINATED))), None)]
    fn test_null_or_eol(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], ParsedBytes>,
        #[case] diff_res_expected: Option<IResult<&[u8], ParsedBytes>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.null_or_eol()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.null_or_eol()(input), res_expected);
        } else {
            assert_eq!(res_parser.null_or_eol()(input), expected);
        }
    }

    #[rstest]
    #[case(b'\n', true)]
    #[case(b'\0', false)]
    #[case(b'\t', false)]
    #[case(b' ', false)]
    #[case(b'\r', false)]
    fn test_terminator(#[case] input: u8, #[case] expected: bool) {
        let req_parser = Parser::new(Side::Request);
        let res_parser = Parser::new(Side::Response);
        assert_eq!(req_parser.is_terminator(input), expected);
        assert_eq!(res_parser.is_terminator(input), expected);
    }

    #[rstest]
    #[case::no_fold_tag(b"test", Err(Error(NomError::new(b!("test"), Tag))))]
    #[case::incomplete(b"\r", Err(Incomplete(Needed::new(1))))]
    #[case::not_folding(b"\r\n", Err(Error(NomError::new(b!("\n"), Not))))]
    #[case::not_folding(b"\r\r", Err(Error(NomError::new(b!("\r"), Not))))]
    #[case::non_special_folding(b"\r\r\t next", Err(Error(NomError::new(b!("\r\t next"), Not))))]
    #[case::non_special_folding(b"\r\n\t next", Err(Error(NomError::new(b!("\n\t next"), Not))))]
    #[case::special_folding(b"\rnext", Ok((b!("next"), b!("\r"))))]
    #[case::special_folding(b"\r\t next", Ok((b!("next"), b!("\r\t "))))]
    fn test_folding_lws_special(#[case] input: &[u8], #[case] expected: IResult<&[u8], &[u8]>) {
        assert_eq!(folding_lws_special(input), expected);
    }

    #[rstest]
    #[case::no_fold_tag(b"test", Err(Error(NomError::new(b!("test"), Tag))))]
    #[case::incomplete(b"\r", Err(Incomplete(Needed::new(1))))]
    #[case::not_folding(b"\r\n", Err(Error(NomError::new(b!("\n"), Not))))]
    #[case::not_folding(b"\r\r", Err(Error(NomError::new(b!("\r"), Not))))]
    #[case::special_folding(b"\rnext", Ok((b!("next"), (b!("\r"), Flags::FOLDING_SPECIAL_CASE))))]
    #[case::special_folding(b"\r\t next", Ok((b!("next"), (b!("\r\t "), Flags::FOLDING_SPECIAL_CASE))))]
    #[case::folding(b" next", Ok((b!("next"), (b!(" "), Flags::FOLDING))))]
    #[case::folding(b"\tnext", Ok((b!("next"), (b!("\t"), Flags::FOLDING))))]
    #[case::folding(b"\t next", Ok((b!("next"), (b!("\t "), Flags::FOLDING))))]
    #[case::folding(b"\t\t\r\n", Ok((b!("\r\n"), (b!("\t\t"), Flags::FOLDING))))]
    #[case::folding(b"\t \t\r", Ok((b!("\r"), (b!("\t \t"), Flags::FOLDING))))]
    #[case::folding(b"     \n", Ok((b!("\n"), (b!("     "), Flags::FOLDING))))]
    fn test_folding_lws(#[case] input: &[u8], #[case] expected: IResult<&[u8], ParsedBytes>) {
        assert_eq!(folding_lws(input), expected);
    }

    #[rstest]
    #[case::no_fold_tag(b"test", Err(Error(NomError::new(b!("test"), Tag))), None)]
    #[case::cr(b"\r", Err(Error(NomError::new(b!("\r"), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::crcr(b"\r\r",  Err(Error(NomError::new(b!("\r\r"), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::incomplete_crlf(b"\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_crlf_ws(b"\r\n\t", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_crlf_ws(b"\r\n \t", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete_crlfcr(b"\r\n\r", Err(Incomplete(Needed::new(1))), None)]
    #[case::not_fold(b"\r\n\r\n", Err(Error(NomError::new(b!("\n"), Not))), None)]
    #[case::not_fold(b"\r\n\r\r", Err(Error(NomError::new(b!("\r"), Not))), None)]
    #[case::fold(b"\r\n next", Ok((b!("next"), (b!("\r\n"), b!(" "), Flags::FOLDING))), None)]
    #[case::fold(b"\r\n\tnext", Ok((b!("next"), (b!("\r\n"), b!("\t"), Flags::FOLDING))), None)]
    #[case::fold(b"\r\n\t next", Ok((b!("next"), (b!("\r\n"), b!("\t "), Flags::FOLDING))), None)]
    #[case::fold_not_res(b"\r\n\t\t\r\n", Ok((b!("\r\n"), (b!("\r\n"), b!("\t\t"), Flags::FOLDING))), Some(Err(Error(NomError::new(b!("\r\n\t\t\r\n"), Not)))))]
    #[case::fold_not_res(b"\r\n\t \t\r", Ok((b!("\r"), (b!("\r\n"), b!("\t \t"), Flags::FOLDING))), Some(Err(Error(NomError::new(b!("\r\n\t \t\r"), Not)))))]
    #[case::fold_not_res(b"\r\n     \n", Ok((b!("\n"), (b!("\r\n"), b!("     "), Flags::FOLDING))), Some(Err(Error(NomError::new(b!("\r\n     \n"), Not)))))]
    #[case::special_fold_not_res(b"\n\r     \n", Ok((b!("\n"), (b!("\n"), b!("\r     "), Flags::FOLDING_SPECIAL_CASE))), Some(Err(Error(NomError::new(b!("\n\r     \n"), Not)))))]
    #[case::special_fold(b"\r\n\rnext", Ok((b!("next"), (b!("\r\n"), b!("\r"), Flags::FOLDING_SPECIAL_CASE))), None)]
    #[case::special_fold(b"\r\n\r\t next", Ok((b!("next"), (b!("\r\n"), b!("\r\t "), Flags::FOLDING_SPECIAL_CASE))), None)]
    #[case::fold_res(b"\r    hello \n", Err(Error(NomError::new(b!("\r    hello \n"), Tag))), Some(Ok((b!("hello \n"), (b!("\r"), b!("    "), Flags::FOLDING)))))]
    fn test_folding(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], FoldingBytes>,
        #[case] diff_res_expected: Option<IResult<&[u8], FoldingBytes>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.folding()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.folding()(input), res_expected);
        } else {
            assert_eq!(res_parser.folding()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b"\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"\r\n\t", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"\r\n ", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"\r\n\r", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"\r\n ", Err(Incomplete(Needed::new(1))), None)]
    #[case::crcr(b"\r\r", Err(Error(NomError::new(b!("\r\r"), Tag))), Some(Err(Incomplete(Needed::new(1)))))]
    #[case::fold(b"\r\n\ta", Ok((b!("a"), ((b!("\r\n"), Flags::FOLDING), Some(b!("\t"))))), None)]
    #[case::special_fold(b"\r\n\ra", Ok((b!("a"),((b!("\r\n"), Flags::FOLDING | Flags::FOLDING_SPECIAL_CASE), Some(b!("\r"))))), None)]
    #[case::fold(b"\r\n a", Ok((b!("a"), ((b!("\r\n"), Flags::FOLDING), Some(b!(" "))))), None)]
    #[case::crlf_eol(b"\r\na", Ok((b!("a"), ((b!("\r\n"), 0), None))), None)]
    #[case::lflf_eol(b"\n\na", Ok((b!("\na"), ((b!("\n"), 0), None))), None)]
    #[case::crlfcrlf_eol(b"\r\n\r\na", Ok((b!("\r\na"), ((b!("\r\n"), 0), None))), None)]
    #[case::req_deformed_eol(b"\n\r\r\na", Ok((b!("\r\na"), ((b!("\n\r"), Flags::DEFORMED_EOL), None))), Some(Ok((b!("\r\na"), ((b!("\n\r"), 0), None)))))]
    #[case::null_terminated(b"\0a", Ok((b!("a"), ((b!("\0"), Flags::NULL_TERMINATED), None))), None)]
    #[case::res_fold(b"\r a", Err(Error(NomError::new(b!("\r a"), Tag))), Some(Ok((b!("a"), ((b!("\r"), Flags::FOLDING), Some(b!(" ")))))))]
    #[case::req_fold_res_empty(b"\n\r \na:b", Ok((b!("\na:b"), ((b!("\n"), Flags::FOLDING_SPECIAL_CASE), Some(b!("\r "))))), Some(Ok((b!("a:b"), ((b!("\n\r \n"), Flags::FOLDING_EMPTY), None)))))]
    #[case::req_fold_res_empty(b"\n \na:b", Ok((b!("\na:b"), ((b!("\n"), Flags::FOLDING), Some(b!(" "))))), Some(Ok((b!("a:b"), ((b!("\n \n"), Flags::FOLDING_EMPTY), None)))))]
    #[case::req_fold_res_empty(b"\r\n \na:b", Ok((b!("\na:b"), ((b!("\r\n"), Flags::FOLDING), Some(b!(" "))))), Some(Ok((b!("a:b"), ((b!("\r\n \n"), Flags::FOLDING_EMPTY), None)))))]
    #[case::req_fold_res_empty(b"\r\n \r\na:b", Ok((b!("\r\na:b"), ((b!("\r\n"), Flags::FOLDING), Some(b!(" "))))), Some(Ok((b!("a:b"), ((b!("\r\n \r\n"), Flags::FOLDING_EMPTY), None)))))]
    #[case::req_fold_res_term(b"\n \r\na\n", Ok((b!("\r\na\n"), ((b!("\n"), Flags::FOLDING), Some(b!(" "))))), Some(Ok((b!("\r\na\n"), ((b!("\n "), Flags::TERMINATOR_SPECIAL_CASE), None)))))]
    #[case::req_fold_res_empty_term(b"\n \r\n\n", Ok((b!("\r\n\n"), ((b!("\n"), Flags::FOLDING), Some(b!(" "))))), Some(Ok((b!("\n"), ((b!("\n \r\n"), Flags::FOLDING_EMPTY | Flags::TERMINATOR_SPECIAL_CASE), None)))))]
    #[case::req_fold_special_res_empty(b"\n\r \na:b", Ok((b!("\na:b"), ((b!("\n"), Flags::FOLDING_SPECIAL_CASE), Some(b!("\r "))))), Some(Ok((b!("a:b"), ((b!("\n\r \n"), Flags::FOLDING_EMPTY), None)))))]
    fn test_folding_or_terminator(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], FoldingOrTerminator>,
        #[case] diff_res_expected: Option<IResult<&[u8], FoldingOrTerminator>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.folding_or_terminator()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.folding_or_terminator()(input), res_expected);
        } else {
            assert_eq!(res_parser.folding_or_terminator()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b" ", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"value", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"\tvalue", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b" value", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"value\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"\r\r", Err(Incomplete(Needed::new(1))), None)]
    #[case::diff_values(b"www.google.com\rName: Value\r\n\r\n", Ok((b!("\r\n"), (b!("www.google.com\rName: Value"), ((b!("\r\n"), 0), None)))), Some(Ok((b!("Name: Value\r\n\r\n"), (b!("www.google.com"), ((b!("\r"), 0), None))))))]
    #[case::diff_values(b"www.google.com\rName: Value\n\r\n", Ok((b!("\r\n"), (b!("www.google.com\rName: Value"), ((b!("\n"), 0), None)))), Some(Ok((b!("Name: Value\n\r\n"), (b!("www.google.com"), ((b!("\r"), 0), None))))))]
    #[case::diff_values(b"www.google.com\rName: Value\r\n\n", Ok((b!("\n"), (b!("www.google.com\rName: Value"), ((b!("\r\n"), 0), None)))), Some(Ok((b!("Name: Value\r\n\n"), (b!("www.google.com"), ((b!("\r"), 0), None))))))]
    #[case::value(b"\r\nnext", Ok((b!("next"), (b!(""), ((b!("\r\n"), 0), None)))), None)]
    #[case::value(b"value\r\nname2", Ok((b!("name2"), (b!("value"), ((b!("\r\n"), 0), None)))), None)]
    #[case::fold_value(b"value\n more", Ok((b!("more"), (b!("value"), ((b!("\n"), Flags::FOLDING), Some(b!(" ")))))), None)]
    #[case::fold_value(b"value\r\n\t more", Ok((b!("more"), (b!("value"), ((b!("\r\n"), Flags::FOLDING), Some(b!("\t ")))))), None)]
    #[case::req_special_fold_res_value(b"value\r\n\t more", Ok((b!("more"), (b!("value"), ((b!("\r\n"), Flags::FOLDING), Some(b!("\t ")))))), None)]
    #[case::req_special_fold_res_value(b"value\n\rmore", Ok((b!("more"), (b!("value"), ((b!("\n"), Flags::FOLDING_SPECIAL_CASE), Some(b!("\r")))))), Some(Ok((b!("more"), (b!("value"), ((b!("\n\r"), 0), None))))))]
    #[case::special_fold(b"value\r\n\rmore", Ok((b!("more"), (b!("value"), ((b!("\r\n"), Flags::FOLDING_SPECIAL_CASE), Some(b!("\r")))))), None)]
    fn test_value_bytes(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], ValueBytes>,
        #[case] diff_res_expected: Option<IResult<&[u8], ValueBytes>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.value_bytes()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.value_bytes()(input), res_expected);
        } else {
            assert_eq!(res_parser.value_bytes()(input), expected);
        }
    }

    #[rstest]
    #[case::incomplete(b"value\r\n more\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"value\r\n ", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"value\r\n more", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"value\r\n more\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::incomplete(b"value\n more\r\n", Err(Incomplete(Needed::new(1))), None)]
    #[case::fold(b"\r\n value    \r\nnext:", Ok((b!("next:"), Value {value: b"value".to_vec(), flags: Flags::FOLDING})), None)]
    #[case::fold(b"\r\n value\r\nnext:", Ok((b!("next:"), Value {value: b"value".to_vec(), flags: Flags::FOLDING})), None)]
    #[case::fold(b"value\r\n more\r\n\r\n", Ok((b!("\r\n"), Value {value: b"value more".to_vec(), flags: Flags::FOLDING})), None)]
    #[case::fold(b"value\r\n more\r\n\tand more\r\nnext:", Ok((b!("next:"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING})), None)]
    #[case::fold(b"value\n\t\tmore\r\n  and\r\n more\r\nnext:", Ok((b!("next:"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING})), None)]
    #[case::req_special_res_fold(b"value\n more\n\r\tand more\r\n\r\n", Ok((b!("\r\n"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING_SPECIAL_CASE})), Some(Ok((b!("\r\n"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING}))))]
    #[case::req_special_res_fold(b"value\n\r\t\tmore\r\n  and\r\n more\r\nnext:", Ok((b!("next:"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING_SPECIAL_CASE})), Some(Ok((b!("next:"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING}))))]
    #[case::req_special_res_value(b"value\n\r\t\tmore\r\n  and\r\n more\r\nnext:", Ok((b!("next:"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING_SPECIAL_CASE})), Some(Ok((b!("next:"), Value {value: b"value more and more".to_vec(), flags: Flags::FOLDING}))))]
    #[case::req_special_deformed_res_fold(b"value1\n\r next: value2\r\n  and\r\n more\r\nnext3:", Ok((b!("next3:"), Value {value: b"value1 next: value2 and more".to_vec(), flags: Flags::FOLDING_SPECIAL_CASE})), Some(Ok((b!("next: value2\r\n  and\r\n more\r\nnext3:"), Value {value: b"value1".to_vec(), flags: 0}))))]
    #[case::value(b"value\r\nnext:", Ok((b!("next:"), Value {value: b"value".to_vec(), flags: 0})), None)]
    #[case::value_empty(b"\r\nnext:", Ok((b!("next:"), Value {value: b"".to_vec(), flags: Flags::VALUE_EMPTY})), None)]
    fn test_value(
        #[case] input: &[u8],
        #[case] expected: IResult<&[u8], Value>,
        #[case] diff_res_expected: Option<IResult<&[u8], Value>>,
    ) {
        let req_parser = Parser::new(Side::Request);
        assert_eq!(req_parser.value()(input), expected);

        let res_parser = Parser::new(Side::Response);
        if let Some(res_expected) = diff_res_expected {
            assert_eq!(res_parser.value()(input), res_expected);
        } else {
            assert_eq!(res_parser.value()(input), expected);
        }
    }
}
