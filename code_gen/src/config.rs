use crate::lex::{self, TokenStream};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ParseError<'a> {
    #[error("Unable to process input")]
    Lex(#[from] lex::LexError),
    #[error("Unexpected token expected {expected:?}, found {found:?}")]
    UnexpectedToken {
        found: lex::Token<'a>,
        expected: lex::Token<'a>,
    },
    #[error("Unexpected field: {0:?}")]
    UnexpectedField(&'a str),
    #[error("Unexpected end of input")]
    EndOfInput,
    #[error("Missing field: {0:?}")]
    MissingField(&'a str),
    #[error("Expected end of input, but found {0:?}")]
    ExpectedEndOfInput(lex::Token<'a>),
}

fn expect_token<'a>(
    tokens: &mut lex::TokenStream<'a>,
    expected: lex::Token<'a>,
) -> Result<(), ParseError<'a>> {
    match tokens.next() {
        Some(t) => {
            if t == expected {
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken { found: t, expected })
            }
        }
        None => Err(ParseError::EndOfInput),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipelineConfig {
    name: String,
    vs_entry: String,
    fs_entry: String,
}

impl RenderPipelineConfig {
    /// This method will create a [RenderPipelineConfig] from the given string.
    /// This method assumes that the string only contains the config tokens. It
    /// should not be used on shader code directly.
    ///
    /// # Errors
    /// - Lex: occurs when failing to convert `src` to a [crate::lex::TokenStream]
    ///
    pub fn parse<'a>(src: &'a str) -> Result<Self, ParseError<'_>> {
        let mut tokens = lex::TokenStream::new(src)?;

        expect_token(&mut tokens, lex::Token::Hash)?;
        expect_token(&mut tokens, lex::Token::Ident("render_pipeline"))?;

        let mut name = None;
        let mut vs_entry = None;
        let mut fs_entry = None;

        let parse_ident = |tokens: &mut TokenStream<'a>| -> Result<&'a str, ParseError<'a>> {
            match tokens.next() {
                Some(lex::Token::Ident(id)) => Ok(id),
                Some(t) => Err(ParseError::UnexpectedToken {
                    found: t,
                    expected: lex::Token::Ident("ident_name"),
                }),
                None => Err(ParseError::EndOfInput),
            }
        };

        let mut parse_field = |tokens: &mut TokenStream<'a>| -> Result<(), ParseError<'a>> {
            let ident = parse_ident(tokens)?;
            // These fields are simple so we can just use an &mut. If
            // the fields get more complicated (which is likely) then:
            // TODO: make this handle nested structures/arrays
            let field = match ident {
                "name" => &mut name,
                "vs_entry" => &mut vs_entry,
                "fs_entry" => &mut fs_entry,
                f => return Err(ParseError::UnexpectedField(f)),
            };

            expect_token(tokens, lex::Token::Colon)?;

            *field = match tokens.next() {
                Some(lex::Token::String(s)) => Some(s),
                Some(t) => {
                    return Err(ParseError::UnexpectedToken {
                        found: t,
                        expected: lex::Token::String("Some String"),
                    })
                }
                None => return Err(ParseError::EndOfInput),
            };

            Ok(())
        };

        let mut parse_struct = || -> Result<(), ParseError<'_>> {
            expect_token(&mut tokens, lex::Token::LeftParen)?;

            if let Some(lex::Token::Ident(_)) = tokens.peek() {
                parse_field(&mut tokens)?;
    
                while let Some(lex::Token::Comma) = tokens.peek() {
                    let _ = tokens.next();
                    if let Some(lex::Token::RightParen) = tokens.peek() {
                        break;
                    }
                    parse_field(&mut tokens)?;
                }
            }

            expect_token(&mut tokens, lex::Token::RightParen)?;

            Ok(())
        };

        parse_struct()?;

        if let Some(t) = tokens.next() {
            return Err(ParseError::ExpectedEndOfInput(t));
        }

        Ok(Self {
            name: name
                .ok_or_else(|| ParseError::MissingField("name"))?
                .to_owned(),
            vs_entry: vs_entry
                .ok_or_else(|| ParseError::MissingField("vs_entry"))?
                .to_owned(),
            fs_entry: fs_entry
                .ok_or_else(|| ParseError::MissingField("fs_entry"))?
                .to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_pipeline_config_parse() {
        let configs = [
            r#"
                #render_pipeline(
                    name: "TexturedPipeline",
                    vs_entry: "vs_textured",
                    fs_entry: "fs_textured",
                )
            "#,
            r#"
                #render_pipeline(
                    name: "TexturedPipeline",
                    vs_entry: "vs_textured",
                    fs_entry: "fs_textured"
                )
            "#,
        ];
        for src in configs {
            assert_eq!(
                Ok(RenderPipelineConfig {
                    name: "TexturedPipeline".to_owned(),
                    vs_entry: "vs_textured".to_owned(),
                    fs_entry: "fs_textured".to_owned()
                }),
                RenderPipelineConfig::parse(src),
            )
        }
    }

    #[test]
    fn render_pipeline_config_parse_missing_fields() {
        let configs = [
            r#"#render_pipeline()"#,
            r#"#render_pipeline(name:"Name")"#,
            r#"#render_pipeline(name:"Name",vs_entry:"vs_entry")"#,
        ];
        for src in configs {
            match RenderPipelineConfig::parse(src) {
                Ok(_) => panic!("Parse succeeded when it should have failed: {:?}", src),
                Err(ParseError::MissingField(_)) => (),
                Err(e) => panic!("Expected `ParseError::MissingField` but found {:?}", e),
            }
        }
    }

}
