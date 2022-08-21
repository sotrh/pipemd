use std::str::CharIndices;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Token<'a> {
    Ident(&'a str),
    String(&'a str),
    Hash,
    Comma,
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Span {
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpannedStr<'a> {
    src: &'a str,
    span: Span,
}

impl<'a> SpannedStr<'a> {
    pub fn new(src: &'a str, start_byte: usize, end_byte: usize) -> Self {
        Self {
            src,
            span: Span {
                start_byte,
                end_byte,
            },
        }
    }

    pub fn substring(&self) -> &'a str {
        if self.span.start_byte >= self.span.end_byte {
            ""
        } else {
            &self.src[self.span.start_byte..self.span.end_byte]
        }
    }

    pub fn remaining(self) -> Option<SpannedStr<'a>> {
        if self.span.end_byte < self.src.len() {
            Some(SpannedStr::new(
                self.src,
                self.span.end_byte,
                self.src.len(),
            ))
        } else {
            None
        }
    }

    pub fn first_char(&self) -> Option<char> {
        self.substring().chars().next()
    }

    pub fn skip(self, n: usize) -> Option<SpannedStr<'a>> {
        let src = self.substring();
        let start_byte = self.span.start_byte;
        let mut new_start = start_byte;
        let mut iter = src.char_indices().take(n + 1);
        let mut num = 0;
        while let Some((i, _)) = iter.next()
        {
            new_start = start_byte + i;
            num += 1;
        }

        if num <= n {
            return None;
        }

        Some(SpannedStr::new(self.src, new_start, self.src.len()))
    }
}

impl<'a> From<&'a str> for SpannedStr<'a> {
    fn from(other: &'a str) -> Self {
        Self::new(other, 0, other.len())
    }
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    #[error("Reached end of input")]
    EndOfInput,
    #[error("Encountered invalid character: {0}")]
    InvalidChar(char),
    #[error("String didn't terminate")]
    NonterminatedString,
}

fn parse<'a>(src: &'a str, matcher: impl Fn(char, usize) -> bool) -> SpannedStr<'a> {
    let mut chars = src.char_indices();
    let mut span = Span {
        start_byte: 0,
        end_byte: 0,
    };
    let mut char_index = 0;
    loop {
        if let Some((i, c)) = chars.next() {
            if !matcher(c, char_index) {
                span.end_byte = i;
                break;
            }
        } else {
            span.end_byte = src.len();
            break;
        }
        char_index += 1;
    }

    SpannedStr { src, span }
}

fn parse_token<'a>(src: &'a str) -> Result<(Token<'a>, Option<SpannedStr<'a>>), ParseError> {
    let span = parse(src, |c, _| c.is_whitespace());
    let span = span.remaining().ok_or(ParseError::EndOfInput)?;

    match span.first_char().ok_or(ParseError::EndOfInput)? {
        c if c.is_alphabetic() || c == '_' => {
            let data = parse(span.substring(), |c, _| c.is_alphanumeric() || c == '_');
            Ok((Token::Ident(data.substring()), data.remaining()))
        }
        c if c == '#' => Ok((Token::Hash, span.skip(1))),
        c if c == '(' => Ok((Token::LeftParen, span.skip(1))),
        c if c == ')' => Ok((Token::RightParen, span.skip(1))),
        c if c == ',' => Ok((Token::Comma, span.skip(1))),
        c if c == '"' => {
            let data = span.skip(1).ok_or(ParseError::NonterminatedString)?;
            let data = parse(data.substring(), |c, _| {
                println!("c = {}", c);
                c != '"' && c != '\n'
            });
            let remaining = data.remaining().ok_or(ParseError::NonterminatedString)?;
            if remaining.first_char() != Some('"') {
                return Err(ParseError::NonterminatedString);
            }
            
            Ok((Token::String(data.substring()), remaining.skip(1)))
        }
        c => Err(ParseError::InvalidChar(c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spanned_str_substring() {
        assert_eq!("sub", SpannedStr::new("substring", 0, 3).substring());
        assert_eq!("string", SpannedStr::new("substring", 3, 9).substring());
        assert_eq!("🚀", SpannedStr::new("🚀substring", 0, 4).substring());
        assert_eq!("", SpannedStr::new("substring", 0, 0).substring());
        assert_eq!("", SpannedStr::new("substring", 10, 0).substring());
    }

    #[test]
    fn spanned_str_remaining() {
        assert_eq!(
            "string",
            SpannedStr::new("substring", 0, 3)
                .remaining()
                .unwrap()
                .substring()
        );
        assert_eq!(None, SpannedStr::new("substring", 3, 9).remaining());
        assert_eq!(
            "substring",
            SpannedStr::new("🚀substring", 0, 4)
                .remaining()
                .unwrap()
                .substring()
        );
        assert_eq!(
            "substring",
            SpannedStr::new("substring", 0, 0)
                .remaining()
                .unwrap()
                .substring()
        );
        // This should probably never happen as the start index should always be greater than
        // the end index.
        assert_eq!(
            "substring",
            SpannedStr::new("substring", 10, 0)
                .remaining()
                .unwrap()
                .substring()
        );
    }

    #[test]
    fn spanned_str_first_char() {
        assert_eq!(Some('🚀'), SpannedStr::from("🚀substring").first_char());
        assert_eq!(Some('s'), SpannedStr::new("🚀substring", 4, 9).first_char());
    }

    #[test]
    fn spanned_str_skip() {
        let original = "abc🚀def";
        assert_eq!("abc🚀def", SpannedStr::from(original).skip(0).unwrap().substring());
        assert_eq!("bc🚀def", SpannedStr::from(original).skip(1).unwrap().substring());
        assert_eq!("c🚀def", SpannedStr::from(original).skip(2).unwrap().substring());
        let data = SpannedStr::from(original).skip(1).unwrap();
        println!("data = {}", data.substring());
        assert_eq!("c🚀def", data.skip(1).unwrap().substring());
        assert_eq!("🚀def", SpannedStr::from(original).skip(3).unwrap().substring());
        assert_eq!("def", SpannedStr::from(original).skip(4).unwrap().substring());
        assert_eq!("ef", SpannedStr::from(original).skip(5).unwrap().substring());
        assert_eq!("f", SpannedStr::from(original).skip(6).unwrap().substring());
        assert_eq!(None, SpannedStr::from(original).skip(7));
        assert_eq!(None, SpannedStr::from(original).skip(8));
        assert_eq!(None, SpannedStr::from("").skip(5));
    }

    #[test]
    fn test_parse() {
        assert_eq!("   ", parse("   abc", |c, _| c == ' ').substring());
        assert_eq!("   ", parse("   ", |c, _| c == ' ').substring());
        assert_eq!("", parse("abc   ", |c, _| c == ' ').substring());
        assert_eq!("🚀🚀🚀", parse("🚀🚀🚀   ", |c, _| c == '🚀').substring());
        assert_eq!("🚀🚀🚀", parse("🚀🚀🚀", |c, _| c == '🚀').substring());
        assert_eq!(
            "🚀a🚀b🚀c",
            parse("🚀a🚀b🚀c", |c, _| c == '🚀'
                || c == 'a'
                || c == 'b'
                || c == 'c')
            .substring()
        );
    }

    #[test]
    fn test_parse_token() {
        assert_eq!(Token::Ident("test"), parse_token("  test   ").unwrap().0);
        assert_eq!(Token::Hash, parse_token("  #   ").unwrap().0);
        assert_eq!(Token::LeftParen, parse_token("  (   ").unwrap().0);
        assert_eq!(Token::RightParen, parse_token("  )   ").unwrap().0);
        assert_eq!(Token::Comma, parse_token("  ,   ").unwrap().0);
        assert_eq!(Token::String("test()a;sldkfj"), parse_token("  \"test()a;sldkfj\"   ").unwrap().0);
        assert_eq!(Err(ParseError::EndOfInput), parse_token("     "));
        assert_eq!(Err(ParseError::InvalidChar('$')), parse_token("   $  "));
        assert_eq!(Err(ParseError::NonterminatedString), parse_token("  \""));
        assert_eq!(Err(ParseError::NonterminatedString), parse_token("  \"\n\""));
    }
}
