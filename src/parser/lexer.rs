use super::{error::Location, ParseError};
use log::{debug, trace};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

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
    Stream,
    Public,
    Extensions,
    FullyQualifiedIdentifier(&'a str),
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
    Optional,
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
            Token::Stream => "stream".to_string(),
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
            Token::FullyQualifiedIdentifier(s) => s.to_string(),
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
            Token::Optional => "optional".to_string(),
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

#[allow(dead_code)]
fn parse_syntax_keywords(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("syntax"), |_| Token::Syntax),
        map(tag("proto2"), |_| Token::Proto2),
        map(tag("proto3"), |_| Token::Proto3),
    ))(input)
}

#[allow(dead_code)]
fn parse_import_keywords(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("import"), |_| Token::Import),
        map(tag("weak"), |_| Token::Weak),
        map(tag("public"), |_| Token::Public),
    ))(input)
}

#[allow(dead_code)]
fn parse_message_keywords(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("message"), |_| Token::Message),
        map(tag("enum"), |_| Token::Enum),
        map(tag("oneof"), |_| Token::Oneof),
        map(tag("map"), |_| Token::Map),
    ))(input)
}

#[allow(dead_code)]
fn parse_field_keywords(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("repeated"), |_| Token::Repeated),
        map(tag("optional"), |_| Token::Optional),
        map(tag("required"), |_| Token::Required),
        map(tag("string"), |_| Token::StringType),
    ))(input)
}

#[allow(dead_code)]
fn parse_service_keywords(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("service"), |_| Token::Service),
        map(tag("rpc"), |_| Token::Rpc),
        map(tag("returns"), |_| Token::Returns),
    ))(input)
}

#[allow(dead_code)]
fn parse_option_keywords(input: &str) -> IResult<&str, Token> {
    map(tag("option"), |_| Token::Option)(input)
}

#[allow(dead_code)]
fn parse_misc_keywords(input: &str) -> IResult<&str, Token> {
    alt((
        map(tag("package"), |_| Token::Package),
        map(tag("reserved"), |_| Token::Reserved),
        map(tag("to"), |_| Token::To),
        map(tag("extensions"), |_| Token::Extensions),
    ))(input)
}

#[allow(dead_code)]
fn parse_keyword(input: &str) -> IResult<&str, Token> {
    alt((
        parse_syntax_keywords,
        parse_import_keywords,
        parse_message_keywords,
        parse_field_keywords,
        parse_service_keywords,
        parse_option_keywords,
        parse_misc_keywords,
    ))(input)
}

#[allow(dead_code)]
fn parse_identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| match s {
            "syntax" => Token::Syntax,
            "proto2" => Token::Proto2,
            "proto3" => Token::Proto3,
            "import" => Token::Import,
            "package" => Token::Package,
            "message" => Token::Message,
            "enum" => Token::Enum,
            "service" => Token::Service,
            "rpc" => Token::Rpc,
            "returns" => Token::Returns,
            "option" => Token::Option,
            "repeated" => Token::Repeated,
            "oneof" => Token::Oneof,
            "map" => Token::Map,
            "reserved" => Token::Reserved,
            "to" => Token::To,
            "weak" => Token::Weak,
            "public" => Token::Public,
            "extensions" => Token::Extensions,
            "required" => Token::Required,
            "optional" => Token::Optional,
            "string" => Token::StringType,
            _ => Token::Identifier(s),
        },
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
                tag("\\n"),
                tag("\\r"),
                tag("\\t"),
                tag("\\b"),
                tag("\\f"),
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
            parse_compound_identifier,
            parse_identifier,
            parse_string_literal,
            parse_float_literal,
            parse_int_literal,
            parse_symbol,
            parse_comment,
            map(take(1usize), |c: &str| Token::Unknown(c.to_string())),
        )),
    )(input)
}

fn parse_compound_identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            alt((
                tag("message"),
                tag("map"),
                tag("repeated"),
                tag("required"),
                tag("optional"),
                // Add other keyword prefixes here if needed
                alpha1,
            )),
            many0(alt((
                recognize(pair(
                    tag("_"),
                    take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                )),
                recognize(take_while1(|c: char| c.is_alphanumeric())),
            ))),
        )),
        |s: &str| match s {
            "message" => Token::Message,
            "map" => Token::Map,
            "repeated" => Token::Repeated,
            "required" => Token::Required,
            "optional" => Token::Optional,
            "syntax" => Token::Syntax,
            "proto2" => Token::Proto2,
            "proto3" => Token::Proto3,
            "import" => Token::Import,
            "package" => Token::Package,
            "enum" => Token::Enum,
            "service" => Token::Service,
            "rpc" => Token::Rpc,
            "returns" => Token::Returns,
            "option" => Token::Option,
            "oneof" => Token::Oneof,
            "reserved" => Token::Reserved,
            "to" => Token::To,
            "weak" => Token::Weak,
            "public" => Token::Public,
            "extensions" => Token::Extensions,
            "string" => Token::StringType,
            _ => Token::Identifier(s),
        },
    )(input)
}

pub fn tokenize(input: &str) -> Result<Vec<TokenWithLocation>, ParseError> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut line = 1;
    let mut column = 1;

    while pos < input.len() {
        let current_char = input[pos..].chars().next().unwrap();
        let start_column = column;

        match current_char {
            ' ' | '\t' | '\r' => {
                pos += 1;
                column += 1;
            }
            '\n' => {
                pos += 1;
                line += 1;
                column = 1;
            }
            '/' => {
                if input[pos..].starts_with("//") {
                    let end = pos + input[pos..].find('\n').unwrap_or(input.len() - pos);
                    let comment = &input[pos..end];
                    tokens.push(TokenWithLocation {
                        token: Token::Comment(comment),
                        location: Location {
                            line,
                            column: start_column,
                        },
                    });
                    pos = end;
                    line += 1;
                    column = 1;
                } else if input[pos..].starts_with("/*") {
                    let end = pos + input[pos..].find("*/").map_or(input.len() - pos, |i| i + 2);
                    let comment = &input[pos..end];
                    let newlines = comment.chars().filter(|&c| c == '\n').count();
                    tokens.push(TokenWithLocation {
                        token: Token::Comment(comment),
                        location: Location {
                            line,
                            column: start_column,
                        },
                    });
                    pos = end;
                    line += newlines;
                    if newlines > 0 {
                        column = comment.chars().rev().take_while(|&c| c != '\n').count() + 1;
                    } else {
                        column += comment.len();
                    }
                } else {
                    return Err(ParseError::UnexpectedCharacter(
                        '/',
                        Location { line, column },
                    ));
                }
            }
            '"' => {
                let (token, len) = tokenize_string_literal(&input[pos..])?;
                tokens.push(TokenWithLocation {
                    token,
                    location: Location {
                        line,
                        column: start_column,
                    },
                });
                pos += len;
                column += len;
            }
            '0'..='9' | '-' | '+' | '.' => {
                let (token, len) = tokenize_number(&input[pos..]);
                tokens.push(TokenWithLocation {
                    token,
                    location: Location {
                        line,
                        column: start_column,
                    },
                });
                pos += len;
                column += len;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let (token, len) = tokenize_identifier(&input[pos..]);
                tokens.push(TokenWithLocation {
                    token,
                    location: Location {
                        line,
                        column: start_column,
                    },
                });
                pos += len;
                column += len;
            }
            '=' | ';' | '{' | '}' | '(' | ')' | '[' | ']' | '<' | '>' | ',' | '.' => {
                let token = match current_char {
                    '=' => Token::Equals,
                    ';' => Token::Semicolon,
                    '{' => Token::OpenBrace,
                    '}' => Token::CloseBrace,
                    '(' => Token::OpenParen,
                    ')' => Token::CloseParen,
                    '[' => Token::OpenBracket,
                    ']' => Token::CloseBracket,
                    '<' => Token::LessThan,
                    '>' => Token::GreaterThan,
                    ',' => Token::Comma,
                    '.' => Token::Dot,
                    _ => unreachable!(),
                };
                tokens.push(TokenWithLocation {
                    token,
                    location: Location {
                        line,
                        column: start_column,
                    },
                });
                pos += 1;
                column += 1;
            }
            c => {
                return Err(ParseError::UnexpectedCharacter(
                    c,
                    Location { line, column },
                ));
            }
        }
    }

    Ok(tokens)
}

fn tokenize_string_literal(input: &str) -> Result<(Token, usize), ParseError> {
    let mut end = 1;
    let mut escaped = false;

    for ch in input[1..].chars() {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Ok((Token::StringLiteral(&input[1..end]), end + 1));
        }
        end += 1;
    }

    Err(ParseError::UnterminatedStringLiteral(Location::new(0, 0))) // You might want to pass proper location here
}

fn tokenize_number(input: &str) -> (Token, usize) {
    let mut end = 0;
    let mut is_float = false;
    let mut has_digit = false;

    // Handle optional sign
    if input.starts_with('-') || input.starts_with('+') {
        end += 1;
    }

    for (i, ch) in input[end..].char_indices() {
        match ch {
            '0'..='9' => {
                end += i + 1;
                has_digit = true;
            }
            '.' if !is_float => {
                is_float = true;
                end += i + 1;
            }
            'e' | 'E'
                if has_digit
                    && !input[end..end + i].contains('e')
                    && !input[end..end + i].contains('E') =>
            {
                is_float = true;
                end += i + 1;
                if end < input.len()
                    && (input[end..].starts_with('-') || input[end..].starts_with('+'))
                {
                    end += 1;
                }
            }
            _ => break,
        }
    }

    let number_str = &input[..end];
    if is_float {
        (Token::FloatLiteral(number_str.parse().unwrap()), end)
    } else {
        (Token::IntLiteral(number_str.parse().unwrap()), end)
    }
}

fn tokenize_identifier(input: &str) -> (Token, usize) {
    let mut end = 0;
    for (i, ch) in input.char_indices() {
        if ch.is_alphanumeric() || ch == '_' {
            end = i + 1;
        } else {
            break;
        }
    }
    let identifier = &input[..end];
    match identifier {
        "syntax" => (Token::Syntax, end),
        "proto2" => (Token::Proto2, end),
        "proto3" => (Token::Proto3, end),
        "import" => (Token::Import, end),
        "package" => (Token::Package, end),
        "message" => (Token::Message, end),
        "enum" => (Token::Enum, end),
        "service" => (Token::Service, end),
        "rpc" => (Token::Rpc, end),
        "returns" => (Token::Returns, end),
        "option" => (Token::Option, end),
        "repeated" => (Token::Repeated, end),
        "oneof" => (Token::Oneof, end),
        "map" => (Token::Map, end),
        "reserved" => (Token::Reserved, end),
        "to" => (Token::To, end),
        "weak" => (Token::Weak, end),
        "public" => (Token::Public, end),
        "extensions" => (Token::Extensions, end),
        "stream" => (Token::Stream, end),
        "string" => (Token::StringType, end),
        "int32" => (Token::Identifier("int32"), end),
        "int64" => (Token::Identifier("int64"), end),
        "uint32" => (Token::Identifier("uint32"), end),
        "uint64" => (Token::Identifier("uint64"), end),
        "sint32" => (Token::Identifier("sint32"), end),
        "sint64" => (Token::Identifier("sint64"), end),
        "fixed32" => (Token::Identifier("fixed32"), end),
        "fixed64" => (Token::Identifier("fixed64"), end),
        "sfixed32" => (Token::Identifier("sfixed32"), end),
        "sfixed64" => (Token::Identifier("sfixed64"), end),
        "bool" => (Token::Identifier("bool"), end),
        "bytes" => (Token::Identifier("bytes"), end),
        "double" => (Token::Identifier("double"), end),
        "float" => (Token::Identifier("float"), end),
        "true" => (Token::Identifier("true"), end),
        "false" => (Token::Identifier("false"), end),
        "inf" => (Token::Identifier("inf"), end),
        "nan" => (Token::Identifier("nan"), end),
        _ => (Token::Identifier(identifier), end),
    }
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
                    location: Location {
                        line: 2,
                        column: 17
                    }
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location {
                        line: 2,
                        column: 24
                    }
                },
                TokenWithLocation {
                    token: Token::StringLiteral("proto3"),
                    location: Location {
                        line: 2,
                        column: 26
                    }
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location {
                        line: 2,
                        column: 34
                    }
                },
                TokenWithLocation {
                    token: Token::Message,
                    location: Location {
                        line: 4,
                        column: 17
                    }
                },
                TokenWithLocation {
                    token: Token::Identifier("Person"),
                    location: Location {
                        line: 4,
                        column: 25
                    }
                },
                TokenWithLocation {
                    token: Token::OpenBrace,
                    location: Location {
                        line: 4,
                        column: 32
                    }
                },
                TokenWithLocation {
                    token: Token::StringType,
                    location: Location {
                        line: 5,
                        column: 21
                    }
                },
                TokenWithLocation {
                    token: Token::Identifier("name"),
                    location: Location {
                        line: 5,
                        column: 28
                    }
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location {
                        line: 5,
                        column: 33
                    }
                },
                TokenWithLocation {
                    token: Token::IntLiteral(1),
                    location: Location {
                        line: 5,
                        column: 35
                    }
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location {
                        line: 5,
                        column: 36
                    }
                },
                TokenWithLocation {
                    token: Token::Identifier("int32"),
                    location: Location {
                        line: 6,
                        column: 21
                    }
                },
                TokenWithLocation {
                    token: Token::Identifier("age"),
                    location: Location {
                        line: 6,
                        column: 27
                    }
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location {
                        line: 6,
                        column: 31
                    }
                },
                TokenWithLocation {
                    token: Token::IntLiteral(2),
                    location: Location {
                        line: 6,
                        column: 33
                    }
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location {
                        line: 6,
                        column: 34
                    }
                },
                TokenWithLocation {
                    token: Token::Identifier("float"),
                    location: Location {
                        line: 7,
                        column: 21
                    }
                },
                TokenWithLocation {
                    token: Token::Identifier("height"),
                    location: Location {
                        line: 7,
                        column: 27
                    }
                },
                TokenWithLocation {
                    token: Token::Equals,
                    location: Location {
                        line: 7,
                        column: 34
                    }
                },
                TokenWithLocation {
                    token: Token::IntLiteral(3),
                    location: Location {
                        line: 7,
                        column: 36
                    }
                },
                TokenWithLocation {
                    token: Token::Semicolon,
                    location: Location {
                        line: 7,
                        column: 37
                    }
                },
                TokenWithLocation {
                    token: Token::CloseBrace,
                    location: Location {
                        line: 8,
                        column: 17
                    }
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
            Token::Comment("/* Multi-line\n            comment */"),
            Token::Identifier("Person"),
            Token::OpenBrace,
            Token::StringType,
            Token::Identifier("name"),
            Token::Equals,
            Token::IntLiteral(1),
            Token::Semicolon,
            Token::Comment("// Inline comment"),
            Token::CloseBrace,
        ];

        assert_eq!(
            tokens.len(),
            expected_tokens.len(),
            "Number of tokens mismatch"
        );

        for (i, (actual, expected)) in tokens.iter().zip(expected_tokens.iter()).enumerate() {
            match (&actual.token, expected) {
                (Token::Comment(actual_comment), Token::Comment(expected_comment)) => {
                    let actual_normalized = normalize_comment(actual_comment);
                    let expected_normalized = normalize_comment(expected_comment);
                    assert_eq!(
                        actual_normalized, expected_normalized,
                        "Comment mismatch at index {}: expected '{}', got '{}'",
                        i, expected_normalized, actual_normalized
                    );
                }
                (actual_token, expected_token) => {
                    assert_eq!(
                        actual_token, expected_token,
                        "Token mismatch at index {}: expected {:?}, got {:?}",
                        i, expected_token, actual_token
                    );
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

        let expected_locations = vec![
            (
                Token::Syntax,
                Location {
                    line: 2,
                    column: 17,
                },
            ),
            (
                Token::Equals,
                Location {
                    line: 2,
                    column: 24,
                },
            ),
            (
                Token::StringLiteral("proto3"),
                Location {
                    line: 2,
                    column: 26,
                },
            ),
            (
                Token::Semicolon,
                Location {
                    line: 2,
                    column: 34,
                },
            ),
            (
                Token::Message,
                Location {
                    line: 3,
                    column: 17,
                },
            ),
            (
                Token::Identifier("Person"),
                Location {
                    line: 3,
                    column: 25,
                },
            ),
            (
                Token::OpenBrace,
                Location {
                    line: 3,
                    column: 32,
                },
            ),
            (
                Token::StringType,
                Location {
                    line: 4,
                    column: 21,
                },
            ),
            (
                Token::Identifier("name"),
                Location {
                    line: 4,
                    column: 28,
                },
            ),
            (
                Token::Equals,
                Location {
                    line: 4,
                    column: 33,
                },
            ),
            (
                Token::IntLiteral(1),
                Location {
                    line: 4,
                    column: 35,
                },
            ),
            (
                Token::Semicolon,
                Location {
                    line: 4,
                    column: 36,
                },
            ),
            (
                Token::CloseBrace,
                Location {
                    line: 5,
                    column: 17,
                },
            ),
        ];

        assert_eq!(
            tokens.len(),
            expected_locations.len(),
            "Number of tokens mismatch"
        );

        for (i, (expected_token, expected_location)) in expected_locations.into_iter().enumerate() {
            assert_eq!(
                tokens[i].token, expected_token,
                "Token mismatch at index {}",
                i
            );
            assert_eq!(
                tokens[i].location, expected_location,
                "Location mismatch at token {} ({:?})",
                i, expected_token
            );
        }
    }

    #[test]
    fn test_float_in_field_number() {
        let input = "message Person { int32 age = 2.5; }";
        let result = tokenize(input);

        assert!(result.is_ok());
        let tokens = result.unwrap();

        // Check that we have the expected number of tokens
        assert_eq!(tokens.len(), 9);

        // Check that the float is correctly tokenized as a FloatLiteral
        assert_eq!(tokens[6].token, Token::FloatLiteral(2.5));
        assert_eq!(tokens[6].location, Location::new(1, 30));

        // Note: The parser should later catch this as an error, not the lexer
    }

    #[test]
    fn test_custom_message_field() {
        let input = "message TestMessage { CustomMessage message_field = 1; }";
        let tokens = tokenize(input).unwrap();

        let actual_tokens: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        let expected_tokens = vec![
            &Token::Message,
            &Token::Identifier("TestMessage"),
            &Token::OpenBrace,
            &Token::Identifier("CustomMessage"),
            &Token::Identifier("message_field"),
            &Token::Equals,
            &Token::IntLiteral(1),
            &Token::Semicolon,
            &Token::CloseBrace,
        ];

        for (i, (actual, expected)) in actual_tokens.iter().zip(expected_tokens.iter()).enumerate()
        {
            assert_eq!(
                actual, expected,
                "Mismatch at token {}: actual {:?}, expected {:?}",
                i, actual, expected
            );
        }

        assert_eq!(
            actual_tokens.len(),
            expected_tokens.len(),
            "Token count mismatch"
        );
    }

    #[test]
    fn test_parse_misc_keywords() {
        assert_eq!(parse_misc_keywords("package"), Ok(("", Token::Package)));
        assert_eq!(parse_misc_keywords("reserved"), Ok(("", Token::Reserved)));
        assert_eq!(parse_misc_keywords("to"), Ok(("", Token::To)));
        assert_eq!(
            parse_misc_keywords("extensions"),
            Ok(("", Token::Extensions))
        );
    }

    #[test]
    fn test_parse_misc_keywords_with_suffix() {
        assert_eq!(
            parse_misc_keywords("package_name"),
            Ok(("_name", Token::Package))
        );
        assert_eq!(parse_misc_keywords("reserved1"), Ok(("1", Token::Reserved)));
        assert_eq!(parse_misc_keywords("to_field"), Ok(("_field", Token::To)));
        assert_eq!(
            parse_misc_keywords("extensions_list"),
            Ok(("_list", Token::Extensions))
        );
    }

    #[test]
    fn test_parse_misc_keywords_failure() {
        assert!(parse_misc_keywords("other").is_err());
        assert!(parse_misc_keywords("packag").is_err());
        assert!(parse_misc_keywords("reserve").is_err());
        assert!(parse_misc_keywords("t").is_err());
        assert!(parse_misc_keywords("extension").is_err());
    }

    #[test]
    fn test_parse_syntax_keywords() {
        assert_eq!(parse_syntax_keywords("syntax"), Ok(("", Token::Syntax)));
        assert_eq!(parse_syntax_keywords("proto2"), Ok(("", Token::Proto2)));
        assert_eq!(parse_syntax_keywords("proto3"), Ok(("", Token::Proto3)));
    }

    #[test]
    fn test_parse_syntax_keywords_with_suffix() {
        assert_eq!(parse_syntax_keywords("syntax="), Ok(("=", Token::Syntax)));
        assert_eq!(parse_syntax_keywords("proto2;"), Ok((";", Token::Proto2)));
        assert_eq!(parse_syntax_keywords("proto3 "), Ok((" ", Token::Proto3)));
    }

    #[test]
    fn test_parse_syntax_keywords_failure() {
        assert!(parse_syntax_keywords("syntx").is_err());
        assert!(parse_syntax_keywords("proto").is_err());
        assert!(parse_syntax_keywords("proto4").is_err());
    }
}
