use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1},
    combinator::{map, map_res, opt, recognize},
    error::Error,
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

use super::ParseError;

type ParseResult<'a, T> = IResult<&'a str, T, Error<&'a str>>;

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
        )),
    )(input)
}

fn parse_whitespace(input: &str) -> IResult<&str, Token> {
    map(multispace1, |_| Token::Whitespace)(input) // Add a new Token variant for whitespace
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    many0(alt((parse_token, parse_comment, parse_whitespace)))(input)
        .map(|(_, tokens)| tokens)
        .map_err(ParseError::from)
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

        let (remaining, tokens) = tokenize(input).unwrap();
        assert!(remaining.trim().is_empty());

        assert_eq!(
            tokens,
            vec![
                Token::Syntax,
                Token::Equals,
                Token::StringLiteral("proto3"),
                Token::Semicolon,
                Token::Message,
                Token::Identifier("Person"),
                Token::OpenBrace,
                Token::Identifier("string"),
                Token::Identifier("name"),
                Token::Equals,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Identifier("int32"),
                Token::Identifier("age"),
                Token::Equals,
                Token::IntLiteral(2),
                Token::Semicolon,
                Token::Identifier("float"),
                Token::Identifier("height"),
                Token::Equals,
                Token::IntLiteral(3),
                Token::Semicolon,
                Token::CloseBrace,
            ]
        );
    }
}
