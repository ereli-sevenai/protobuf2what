use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

use super::{error::Location, ParseError};
use log::{debug, trace};

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
    Comment(&'a str),
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
            Token::Comment(s) => format!("Comment({})", s),
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
        |s: &str| Token::Identifier(s),
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Token> {
    delimited(
        char('"'),
        map(
            recognize(many0(alt((
                take_while1(|c| c != '"' && c != '\\'),
                tag("\\\""),
                tag("\\\\"),
                preceded(char('\\'), take(1usize)),
            )))),
            Token::StringLiteral,
        ),
        char('"'),
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

fn parse_comment(input: &str) -> IResult<&str, Token> {
    alt((
        // Single-line comment
        map(
            recognize(pair(tag("//"), take_while(|c| c != '\n'))),
            Token::Comment,
        ),
        // Multi-line comment
        map(
            recognize(delimited(
                tag("/*"),
                take_while(|c| c != '*' || input.chars().next() != Some('/')),
                tag("*/"),
            )),
            Token::Comment,
        ),
    ))(input)
}

fn parse_token(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            parse_keyword,
            parse_string_literal,
            parse_identifier,
            parse_float_literal,
            parse_int_literal,
            parse_symbol,
            parse_comment,
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

    // Handle initial whitespace
    let (new_remaining, _) =
        match take_while::<_, _, nom::error::Error<&str>>(|c: char| c.is_whitespace())(remaining) {
            Ok(result) => result,
            Err(_) => {
                return Err(ParseError::LexerError(
                    "Failed to parse initial whitespace".to_string(),
                    Location::new(line, column),
                ))
            }
        };

    // Update line and column based on initial whitespace
    for c in remaining[..remaining.len() - new_remaining.len()].chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    remaining = new_remaining;

    while !remaining.is_empty() {
        trace!(
            "Processing remaining input at line {}, column {}",
            line,
            column
        );
        debug!("Remaining input: {:?}", remaining);

        let (new_remaining, token_opt) = match alt((
            map(parse_token, Some),
            map(recognize(parse_comment), |_| None),
            map(take_while1(char::is_whitespace), |_| None),
        ))(remaining)
        {
            Ok(result) => result,
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
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
        };

        let token_len = remaining.len() - new_remaining.len();
        let token_str = &remaining[..token_len];

        if let Some(token) = token_opt {
            let location = Location::new(line, column);
            let token_with_location = TokenWithLocation {
                token: token.clone(),
                location,
            };
            debug!("Tokenized: {:?} at {:?}", token, location);
            tokens.push(token_with_location);
        }

        // Update line and column
        for c in token_str.chars() {
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        remaining = new_remaining;
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
                    location: Location::new(2, 17)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(2, 23)
                },
                TokenWithLocation {
                    token: Token::StringLiteral("proto3"),
                    location: Location::new(2, 25)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(2, 34)
                },
                TokenWithLocation {
                    token: Token::Message,
                    location: Location::new(2, 35)
                },
                TokenWithLocation {
                    token: Token::Identifier("Person"),
                    location: Location::new(4, 24)
                },
                TokenWithLocation {
                    token: Token::OpenBrace,
                    location: Location::new(4, 31)
                },
                TokenWithLocation {
                    token: Token::Identifier("string"),
                    location: Location::new(4, 33)
                },
                TokenWithLocation {
                    token: Token::Identifier("name"),
                    location: Location::new(5, 27)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(5, 32)
                },
                TokenWithLocation {
                    token: Token::IntLiteral(1),
                    location: Location::new(5, 34)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(5, 36)
                },
                TokenWithLocation {
                    token: Token::Identifier("int32"),
                    location: Location::new(5, 37)
                },
                TokenWithLocation {
                    token: Token::Identifier("age"),
                    location: Location::new(6, 26)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(6, 30)
                },
                TokenWithLocation {
                    token: Token::IntLiteral(2),
                    location: Location::new(6, 32)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(6, 34)
                },
                TokenWithLocation {
                    token: Token::Identifier("float"),
                    location: Location::new(6, 35)
                },
                TokenWithLocation {
                    token: Token::Identifier("height"),
                    location: Location::new(7, 26)
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location::new(7, 33)
                },
                TokenWithLocation {
                    token: Token::IntLiteral(3),
                    location: Location::new(7, 35)
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location::new(7, 37)
                },
                TokenWithLocation {
                    token: Token::CloseBrace,
                    location: Location::new(7, 38)
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
        let input = r#""" "abc" "123" "a b c" "a\"b" "a\\b" "\n\t" "a\b""#;
        let tokens = tokenize(input).unwrap();

        let actual_tokens: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        println!("Actual tokens: {:?}", actual_tokens);

        assert_eq!(
            actual_tokens,
            vec![
                &Token::StringLiteral(""),
                &Token::StringLiteral("abc"),
                &Token::StringLiteral("123"),
                &Token::StringLiteral("a b c"),
                &Token::StringLiteral("a\\\"b"),
                &Token::StringLiteral("a\\\\b"),
                &Token::StringLiteral("\\n\\t"),
                &Token::StringLiteral("a\\b"),
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

        let expected_tokens = vec![
            Token::Comment("// Single line comment"),
            Token::Message,
            Token::Comment("/* Multi-line comment */"),
            Token::Identifier("Person"),
            Token::OpenBrace,
            Token::Identifier("string"),
            Token::Identifier("name"),
            Token::Equals,
            Token::IntLiteral(1),
            Token::Semicolon,
            Token::Comment("// Inline comment"),
            Token::CloseBrace,
        ];

        assert_eq!(tokens.len(), expected_tokens.len());

        for (actual, expected) in tokens.iter().zip(expected_tokens.iter()) {
            match (&actual.token, expected) {
                (Token::Comment(actual_comment), Token::Comment(expected_comment)) => {
                    let actual_normalized = normalize_comment(actual_comment);
                    let expected_normalized = normalize_comment(expected_comment);
                    assert_eq!(
                        actual_normalized, expected_normalized,
                        "Comments don't match after normalization"
                    );
                }
                (actual_token, expected_token) => {
                    assert_eq!(actual_token, expected_token, "Tokens don't match");
                }
            }
        }
    }

    fn normalize_comment(comment: &str) -> String {
        comment
            .replace("/*", "")
            .replace("*/", "")
            .replace("//", "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
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

        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?} at {:?}", i, token.token, token.location);
        }

        assert_eq!(tokens[0].location, Location::new(2, 13)); // syntax
        assert_eq!(tokens[1].location, Location::new(2, 19)); // =
        assert_eq!(tokens[2].location, Location::new(2, 21)); // "proto3"
        assert_eq!(tokens[3].location, Location::new(2, 30)); // ;
        assert_eq!(tokens[4].location, Location::new(2, 31)); // message
        assert_eq!(tokens[5].location, Location::new(3, 20)); // Person
        assert_eq!(tokens[6].location, Location::new(3, 27)); // {
        assert_eq!(tokens[7].location, Location::new(3, 29)); // string
        assert_eq!(tokens[8].location, Location::new(4, 23)); // name
        assert_eq!(tokens[9].location, Location::new(4, 28)); // =
        assert_eq!(tokens[10].location, Location::new(4, 30)); // 1
        assert_eq!(tokens[11].location, Location::new(4, 32)); // ;
        assert_eq!(tokens[12].location, Location::new(4, 33)); // }
    }

    #[test]
    fn test_float_in_field_number() {
        let input = "message Person { int32 age = 2.5; }";
        let result = tokenize(input);

        assert!(result.is_ok());
        let tokens = result.unwrap();

        // Check that we have the expected number of tokens
        assert_eq!(tokens.len(), 9);

        // Check that the float is correctly tokenized
        assert_eq!(tokens[6].token, Token::FloatLiteral(2.5));
        assert_eq!(tokens[6].location, Location::new(1, 29));
    }
}
