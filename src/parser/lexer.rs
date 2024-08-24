use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

use super::{error::Location, ParseError};
use log::{debug, trace};
use nom::error::Error as NomError;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Syntax,
    Proto2,
    Proto3,
    Import,
    Package,
    Message,
    Enum,
    Service,
    Rpc,
    Returns,
    Option,
    Repeated,
    Oneof,
    Map,
    Reserved,
    To,
    Weak,
    Public,
    Extensions,
    Identifier(&'a str),
    StringLiteral(&'a str),
    StringType,
    IntLiteral(i64),
    FloatLiteral(f64),
    Equals,
    Semicolon,
    Comma,
    Dot,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    LessThan,
    GreaterThan,
    Required,
    Comment,
    Whitespace,
    Unknown(String),
}

impl<'a> ToString for Token<'a> {
    fn to_string(&self) -> String {
        match self {
            Token::Syntax => "syntax".to_string(),
            Token::Proto2 => "proto2".to_string(),
            Token::Proto3 => "proto3".to_string(),
            Token::Import => "import".to_string(),
            Token::Package => "package".to_string(),
            Token::Message => "message".to_string(),
            Token::Enum => "enum".to_string(),
            Token::Service => "service".to_string(),
            Token::Rpc => "rpc".to_string(),
            Token::Returns => "returns".to_string(),
            Token::Option => "option".to_string(),
            Token::Repeated => "repeated".to_string(),
            Token::Oneof => "oneof".to_string(),
            Token::Map => "map".to_string(),
            Token::Reserved => "reserved".to_string(),
            Token::To => "to".to_string(),
            Token::Weak => "weak".to_string(),
            Token::Public => "public".to_string(),
            Token::Extensions => "extensions".to_string(),
            Token::Identifier(s) => s.to_string(),
            Token::StringLiteral(s) => format!("\"{}\"", s),
            Token::StringType => "string".to_string(),
            Token::IntLiteral(i) => i.to_string(),
            Token::FloatLiteral(f) => f.to_string(),
            Token::Equals => "=".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::Comma => ",".to_string(),
            Token::Dot => ".".to_string(),
            Token::OpenBrace => "{".to_string(),
            Token::CloseBrace => "}".to_string(),
            Token::OpenParen => "(".to_string(),
            Token::CloseParen => ")".to_string(),
            Token::OpenBracket => "[".to_string(),
            Token::CloseBracket => "]".to_string(),
            Token::LessThan => "<".to_string(),
            Token::GreaterThan => ">".to_string(),
            Token::Required => "required".to_string(),
            Token::Comment => "comment".to_string(),
            Token::Whitespace => "whitespace".to_string(),
            Token::Unknown(s) => s.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenWithLocation<'a> {
    pub token: Token<'a>,
    pub location: Location,
}

impl<'a> TokenWithLocation<'a> {
    pub fn expect(&self, expected: Token) -> Result<TokenWithLocation<'a>, ParseError> {
        if self.token != expected {
            Err(ParseError::UnexpectedToken(
                format!("Expected {:?}, found {:?}", expected, self.token),
                self.location,
            ))
        } else {
            Ok(TokenWithLocation {
                token: self.token.clone(),
                location: self.location,
            })
        }
    }
}

impl<'a> PartialEq for TokenWithLocation<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.location == other.location
    }
}

fn parse_keyword(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("syntax"), |_| Token::Syntax),
        map(tag("proto2"), |_| Token::Proto2),
        map(tag("proto3"), |_| Token::Proto3),
        map(tag("import"), |_| Token::Import),
        map(tag("package"), |_| Token::Package),
        map(tag("message"), |_| Token::Message),
        map(tag("enum"), |_| Token::Enum),
        map(tag("service"), |_| Token::Service),
        map(tag("rpc"), |_| Token::Rpc),
        map(tag("returns"), |_| Token::Returns),
        map(tag("option"), |_| Token::Option),
        map(tag("repeated"), |_| Token::Repeated),
        map(tag("oneof"), |_| Token::Oneof),
        map(tag("map"), |_| Token::Map),
        map(tag("reserved"), |_| Token::Reserved),
        map(tag("to"), |_| Token::To),
        map(tag("weak"), |_| Token::Weak),
        map(tag("public"), |_| Token::Public),
        map(tag("extensions"), |_| Token::Extensions),
    ))(input)
}

fn parse_identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        Token::Identifier,
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Token> {
    map(
        delimited(char('"'), take_while(|c| c != '"'), char('"')),
        Token::StringLiteral,
    )(input)
}

fn parse_int_literal(input: &str) -> IResult<&str, Token> {
    map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| {
        s.parse().map(Token::IntLiteral)
    })(input)
}

fn parse_float_literal(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            opt(char('-')),
            alt((
                recognize(pair(digit1, pair(char('.'), opt(digit1)))),
                recognize(pair(char('.'), digit1)),
            )),
        )),
        |s: &str| Token::FloatLiteral(s.parse().unwrap()),
    )(input)
}

fn parse_symbol(input: &str) -> IResult<&str, Token> {
    alt((
        map(char('='), |_| Token::Equals),
        map(char(';'), |_| Token::Semicolon),
        map(char(','), |_| Token::Comma),
        map(char('.'), |_| Token::Dot),
        map(char('{'), |_| Token::OpenBrace),
        map(char('}'), |_| Token::CloseBrace),
        map(char('('), |_| Token::OpenParen),
        map(char(')'), |_| Token::CloseParen),
        map(char('['), |_| Token::OpenBracket),
        map(char(']'), |_| Token::CloseBracket),
        map(char('<'), |_| Token::LessThan),
        map(char('>'), |_| Token::GreaterThan),
    ))(input)
}

fn parse_comment(input: &str) -> IResult<&str, ()> {
    alt((
        // Single-line comment
        map(pair(tag("//"), take_while(|c| c != '\n')), |_| ()),
        // Multi-line comment
        map(
            delimited(
                tag("/*"),
                take_while(|c| c != '*' || input.chars().next() != Some('/')),
                tag("*/"),
            ),
            |_| (),
        ),
    ))(input)
}

fn parse_token(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            parse_keyword,
            parse_identifier,
            parse_string_literal,
            parse_float_literal,
            parse_int_literal,
            parse_symbol,
            map(take(1usize), |c: &str| Token::Unknown(c.to_string())),
        )),
    )(input)
}

pub fn tokenize(input: &str) -> Result<Vec<TokenWithLocation>, ParseError> {
    debug!(
        "Starting tokenization of input with length: {}",
        input.len()
    );
    let mut tokens = Vec::new();
    let mut remaining = input;
    let mut line = 1;
    let mut column = 1;

    while !remaining.is_empty() {
        trace!(
            "Processing remaining input at line {}, column {}",
            line,
            column
        );
        debug!("Remaining input: {:?}", remaining);

        let (new_remaining, whitespace) =
            match recognize(multispace0::<&str, NomError<&str>>)(remaining) {
                Ok(result) => result,
                Err(e) => {
                    return Err(ParseError::LexerError(
                        format!(
                            "Failed to parse whitespace at line {}, column {}: {:?}",
                            line, column, e
                        ),
                        Location::new(line, column),
                    ));
                }
            };
        let start_line = line;
        let start_column = column;

        // Update line and column based on whitespace
        for c in whitespace.chars() {
            if c == '\n' {
                line += 1;
                column = 1;
                trace!("New line detected, now at line {}", line);
            } else {
                column += 1;
            }
        }

        remaining = new_remaining;
        if remaining.is_empty() {
            break;
        }

        let token_result = alt((
            map(parse_token, Some),
            map(recognize(parse_comment), |_| None),
        ))(remaining);

        match token_result {
            Ok((new_remaining, token_opt)) => {
                if let Some(token) = token_opt {
                    let location = Location::new(start_line, start_column);
                    let token_with_location = TokenWithLocation {
                        token: token.clone(),
                        location,
                    };
                    debug!("Tokenized: {:?} at {:?}", token, location);
                    tokens.push(token_with_location);

                    // Update column for the next token
                    column += token.to_string().len();
                } else {
                    trace!(
                        "Skipped comment or whitespace at line {}, column {}",
                        line,
                        column
                    );
                }
                remaining = new_remaining;
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                debug!("Error occurred with remaining input: {:?}", remaining);
                return Err(ParseError::LexerError(
                    format!(
                        "Failed to parse token at line {}, column {}: {:?}",
                        line, column, e
                    ),
                    Location::new(line, column),
                ));
            }
            Err(nom::Err::Incomplete(_)) => {
                return Err(ParseError::LexerError(
                    format!("Incomplete input at line {}, column {}", line, column),
                    Location::new(line, column),
                ));
            }
        }
    }

    debug!("Tokenization complete. Total tokens: {}", tokens.len());
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = r#"
            syntax = "proto3";

            message Person {
                string name = 1;
                int32 age = 2;
                float height = 3;
            }
        "#;

        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens,
            vec![
                TokenWithLocation {
                    token: Token::Syntax,
                    location: Location::new(1, 1)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(1, 8)
                },
                TokenWithLocation {
                    token: Token::StringLiteral("proto3"),
                    location: Location::new(1, 10)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(1, 17)
                },
                TokenWithLocation {
                    token: Token::Message,
                    location: Location::new(3, 1)
                },
                TokenWithLocation {
                    token: Token::Identifier("Person"),
                    location: Location::new(3, 9)
                },
                TokenWithLocation {
                    token: Token::OpenBracket,
                    location: Location::new(3, 16)
                },
                TokenWithLocation {
                    token: Token::StringType,
                    location: Location::new(4, 3)
                },
                TokenWithLocation {
                    token: Token::Identifier("name"),
                    location: Location::new(4, 10)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(4, 15)
                },
                TokenWithLocation {
                    token: Token::IntLiteral(1),
                    location: Location::new(4, 17)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(4, 18)
                },
                TokenWithLocation {
                    token: Token::IntLiteral(2),
                    location: Location::new(5, 3)
                },
                TokenWithLocation {
                    token: Token::Identifier("age"),
                    location: Location::new(5, 9)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(5, 13)
                },
                TokenWithLocation {
                    token: Token::IntLiteral(3),
                    location: Location::new(5, 15)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(5, 16)
                },
                TokenWithLocation {
                    token: Token::CloseBracket,
                    location: Location::new(6, 1)
                },
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let input = "syntax proto2 proto3 import package message enum service rpc returns option repeated oneof map reserved to weak public extensions";
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens.iter().map(|t| &t.token).collect::<Vec<_>>(),
            vec![
                &Token::Syntax,
                &Token::Proto2,
                &Token::Proto3,
                &Token::Import,
                &Token::Package,
                &Token::Message,
                &Token::Enum,
                &Token::Service,
                &Token::Rpc,
                &Token::Returns,
                &Token::Option,
                &Token::Repeated,
                &Token::Oneof,
                &Token::Map,
                &Token::Reserved,
                &Token::To,
                &Token::Weak,
                &Token::Public,
                &Token::Extensions,
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let input = "abc ABC _abc abc123 _123";
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens.iter().map(|t| &t.token).collect::<Vec<_>>(),
            vec![
                &Token::Identifier("abc"),
                &Token::Identifier("ABC"),
                &Token::Identifier("_abc"),
                &Token::Identifier("abc123"),
                &Token::Identifier("_123"),
            ]
        );
    }

    #[test]
    fn test_string_literals() {
        let input = r#""" "abc" "123" "a b c" "a\"b""#;
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens.iter().map(|t| &t.token).collect::<Vec<_>>(),
            vec![
                &Token::StringLiteral(""),
                &Token::StringLiteral("abc"),
                &Token::StringLiteral("123"),
                &Token::StringLiteral("a b c"),
                &Token::StringLiteral("a\\\"b"),
            ]
        );
    }

    #[test]
    fn test_number_literals() {
        let input = "0 123 -456 3.14 -2.718 .5";
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens.iter().map(|t| &t.token).collect::<Vec<_>>(),
            vec![
                &Token::IntLiteral(0),
                &Token::IntLiteral(123),
                &Token::IntLiteral(-456),
                &Token::FloatLiteral(3.14),
                &Token::FloatLiteral(-2.718),
                &Token::FloatLiteral(0.5),
            ]
        );
    }

    #[test]
    fn test_symbols() {
        let input = "= ; , . { } ( ) [ ] < >";
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens.iter().map(|t| &t.token).collect::<Vec<_>>(),
            vec![
                &Token::Equals,
                &Token::Semicolon,
                &Token::Comma,
                &Token::Dot,
                &Token::OpenBrace,
                &Token::CloseBrace,
                &Token::OpenParen,
                &Token::CloseParen,
                &Token::OpenBracket,
                &Token::CloseBracket,
                &Token::LessThan,
                &Token::GreaterThan,
            ]
        );
    }

    #[test]
    fn test_comments() {
        let input = r#"
                // Single line comment
                message /* Multi-line
                comment */ Person {
                    string name = 1; // Inline comment
                }
            "#;

        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens.iter().map(|t| &t.token).collect::<Vec<_>>(),
            vec![
                &Token::Message,
                &Token::Identifier("Person"),
                &Token::OpenBrace,
                &Token::Identifier("string"),
                &Token::Identifier("name"),
                &Token::Equals,
                &Token::IntLiteral(1),
                &Token::Semicolon,
                &Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn test_location_tracking() {
        let input = r#"
    syntax = "proto3";
    message Person {
        string name = 1;
    }
    "#;

        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens[0].location, Location::new(2, 1)); // syntax
        assert_eq!(tokens[1].location, Location::new(2, 8)); // =
        assert_eq!(tokens[2].location, Location::new(2, 10)); // "proto3"
        assert_eq!(tokens[3].location, Location::new(2, 18)); // ;
        assert_eq!(tokens[4].location, Location::new(3, 1)); // message
        assert_eq!(tokens[5].location, Location::new(3, 9)); // Person
        assert_eq!(tokens[6].location, Location::new(3, 16)); // {
        assert_eq!(tokens[7].location, Location::new(4, 5)); // string
        assert_eq!(tokens[8].location, Location::new(4, 12)); // name
        assert_eq!(tokens[9].location, Location::new(4, 17)); // =
        assert_eq!(tokens[10].location, Location::new(4, 19)); // 1
        assert_eq!(tokens[11].location, Location::new(4, 20)); // ;
        assert_eq!(tokens[12].location, Location::new(5, 1)); // }
    }

    #[test]
    fn test_error_handling() {
        let input = "message Person { int32 age = 2.5; }";
        let result = tokenize(input);

        assert!(result.is_err());
        if let Err(ParseError::LexerError(msg, location)) = result {
            assert!(msg.contains("Tokenization error"));
            assert_eq!(location, Location::new(1, 29));
        } else {
            panic!("Expected LexerError");
        }
    }
}
