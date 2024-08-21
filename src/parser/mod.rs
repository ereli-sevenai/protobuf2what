//! Parser module for Protobuf to Zod converter
//!
//! This module contains the lexer, AST definitions, and parsing logic
//! for processing Protobuf files.

pub mod ast;
pub mod error;
mod lexer;

use crate::parser::ast::Reserved::{FieldName, Number};
use crate::parser::ast::{
    Enum, EnumValue, Field, FieldLabel, Import, ImportKind, Message, Method, OptionValue,
    ProtoFile, ProtoOption, Service, Syntax,
};

pub use error::{ParseError, ParseResult};
pub use lexer::{tokenize, Token};

use std::iter::Peekable;

/// Parse a Protobuf file content into an AST representation
///
/// # Arguments
///
/// * `input` - A string slice containing the Protobuf file content
///
/// # Returns
///
/// * `Result<ProtoFile, ParseError>` - The parsed AST or an error if parsing failed
pub fn parse_proto_file(input: &str) -> Result<ProtoFile, ParseError> {
    let tokens = tokenize(input).map_err(|e| e)?;

    // if !remaining.trim().is_empty() {
    //     println!("Unparsed input: '{}'", remaining);
    //     return Err(ParseError::IncompleteParser(format!(
    //         "Unparsed input remaining: {}",
    //         remaining
    //     )));
    // }

    let mut token_iter = tokens.into_iter().peekable();
    parse_proto(&mut token_iter)
}

fn parse_proto<'a, I>(tokens: &mut Peekable<I>) -> Result<ProtoFile, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let mut proto_file = ProtoFile::new();

    while let Some(token) = tokens.peek() {
        println!("Processing token: {:?}", token);
        match token {
            Token::Syntax => parse_syntax(tokens, &mut proto_file)?,
            Token::Package => parse_package(tokens, &mut proto_file)?,
            Token::Import => parse_import(tokens, &mut proto_file)?,
            Token::Message => {
                tokens.next(); // consume 'message' token
                let message = parse_message(tokens)?;
                proto_file.messages.push(message);
            }
            Token::Enum => {
                tokens.next(); // consume 'enum' token
                let enum_def = parse_enum(tokens)?;
                proto_file.enums.push(enum_def);
            }
            Token::Service => {
                tokens.next(); // consume 'service' token
                let service = parse_service(tokens)?;
                proto_file.services.push(service);
            }
            Token::Option => parse_option(tokens, &mut proto_file.options)?,
            _ => return Err(ParseError::UnexpectedToken(format!("{:?}", token))),
        }
    }

    Ok(proto_file)
}

fn parse_syntax<'a, I>(
    tokens: &mut Peekable<I>,
    proto_file: &mut ProtoFile,
) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.next(); // Consume 'syntax' token
    expect_token(tokens, &Token::Equals)?;

    match tokens.next() {
        Some(Token::StringLiteral("proto2")) => proto_file.syntax = Syntax::Proto2,
        Some(Token::StringLiteral("proto3")) => proto_file.syntax = Syntax::Proto3,
        _ => {
            return Err(ParseError::InvalidSyntax(
                "Expected \"proto2\" or \"proto3\"".to_string(),
            ))
        }
    }

    expect_token(tokens, &Token::Semicolon)?;
    Ok(())
}

fn parse_package<'a, I>(
    tokens: &mut Peekable<I>,
    proto_file: &mut ProtoFile,
) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.next(); // Consume 'package' token

    if let Some(Token::Identifier(package_name)) = tokens.next() {
        proto_file.package = Some(package_name.to_string());
        expect_token(tokens, &Token::Semicolon)?;
        Ok(())
    } else {
        Err(ParseError::ExpectedToken("package name".to_string()))
    }
}

fn parse_import<'a, I>(
    tokens: &mut Peekable<I>,
    proto_file: &mut ProtoFile,
) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.next(); // Consume 'import' token

    let kind = match tokens.peek() {
        Some(Token::Public) => {
            tokens.next();
            ImportKind::Public
        }
        Some(Token::Weak) => {
            tokens.next();
            ImportKind::Weak
        }
        _ => ImportKind::Default,
    };

    if let Some(Token::StringLiteral(path)) = tokens.next() {
        proto_file.imports.push(Import {
            path: path.to_string(),
            kind,
        });
        expect_token(tokens, &Token::Semicolon)?;
        Ok(())
    } else {
        Err(ParseError::ExpectedToken("import path".to_string()))
    }
}

fn parse_message<'a, I>(tokens: &mut Peekable<I>) -> Result<Message, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let name = expect_identifier(tokens)?;
    expect_token(tokens, &Token::OpenBrace)?;

    let mut message = Message::new(name);

    while let Some(token) = tokens.peek() {
        match token {
            Token::CloseBrace => {
                tokens.next();
                return Ok(message);
            }
            Token::Message => {
                tokens.next();
                let nested_message = parse_message(tokens)?;
                message.nested_messages.push(nested_message);
            }
            Token::Enum => {
                tokens.next();
                let nested_enum = parse_enum(tokens)?;
                message.nested_enums.push(nested_enum);
            }
            Token::Option => {
                parse_option(tokens, &mut message.options)?;
            }
            Token::Reserved => {
                parse_reserved(tokens, &mut message.reserved)?;
            }
            _ => {
                let field = parse_field(tokens)?;
                message.fields.push(field);
            }
        }
    }

    Err(ParseError::UnexpectedEndOfInput)
}

fn parse_enum<'a, I>(tokens: &mut Peekable<I>) -> Result<Enum, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let name = expect_identifier(tokens)?;
    expect_token(tokens, &Token::OpenBrace)?;

    let mut enum_def = Enum::new(name);

    while let Some(token) = tokens.peek() {
        match token {
            Token::CloseBrace => {
                tokens.next();
                return Ok(enum_def);
            }
            Token::Option => {
                parse_option(tokens, &mut enum_def.options)?;
            }
            Token::Identifier(_) => {
                let value = parse_enum_value(tokens)?;
                enum_def.values.push(value);
            }
            _ => return Err(ParseError::UnexpectedToken(format!("{:?}", token))),
        }
    }

    Err(ParseError::UnexpectedEndOfInput)
}

fn parse_service<'a, I>(tokens: &mut Peekable<I>) -> Result<Service, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let name = expect_identifier(tokens)?;
    expect_token(tokens, &Token::OpenBrace)?;

    let mut service = Service::new(name);

    while let Some(token) = tokens.peek() {
        match token {
            Token::CloseBrace => {
                tokens.next();
                return Ok(service);
            }
            Token::Option => {
                parse_option(tokens, &mut service.options)?;
            }
            Token::Rpc => {
                tokens.next();
                let method = parse_method(tokens)?;
                service.methods.push(method);
            }
            _ => return Err(ParseError::UnexpectedToken(format!("{:?}", token))),
        }
    }

    Err(ParseError::UnexpectedEndOfInput)
}

fn parse_option<'a, I>(
    tokens: &mut Peekable<I>,
    options: &mut Vec<ProtoOption>,
) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.next(); // Consume 'option' token
    let name = expect_identifier(tokens)?;
    expect_token(tokens, &Token::Equals)?;
    let value = parse_constant(tokens)?;
    expect_token(tokens, &Token::Semicolon)?;

    options.push(ProtoOption {
        name,
        value: OptionValue::String(value),
    });
    Ok(())
}

fn parse_field<'a, I>(tokens: &mut Peekable<I>) -> Result<Field, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let label = match tokens.peek() {
        Some(&Token::Repeated) => {
            tokens.next();
            FieldLabel::Repeated
        }
        Some(&Token::Required) => {
            tokens.next(); // Consume the `Required` token
            if matches!(tokens.peek(), Some(&Token::Proto2)) {
                tokens.next(); // Consume the `Proto2` token
                FieldLabel::Required
            } else {
                FieldLabel::Required // or handle an error/missing case
            }
        }
        _ => FieldLabel::Optional,
    };

    let name = match tokens.next() {
        Some(Token::Identifier(name)) => name.to_string(), // Convert &str to String
        _ => {
            return Err(ParseError::MissingIdentifier(
                "Expected field name".to_string(),
            ))
        }
    };

    let number = match tokens.next() {
        Some(Token::IntLiteral(number)) => number,
        _ => return Err(ParseError::MissingIdentifier("Expected number".to_string())),
    };

    match tokens.next() {
        Some(Token::Equals) => (),
        _ => return Err(ParseError::UnexpectedToken("Unexpected token".to_string())),
    }

    match tokens.next() {
        Some(Token::Semicolon) => (),
        _ => return Err(ParseError::UnexpectedToken("Unexpected token".to_string())),
    }

    Ok(Field {
        name,
        number,
        label,
        typ: "FieldType".to_string(),
        options: Vec::new(), // TODO: Parse field options
    })
}

// fn parse_field_type<'a, I>(tokens: &mut Peekable<I>) -> Result<FieldType, ParseError>
// where
//     I: Iterator<Item = Token<'a>>,
// {
//     match tokens.next() {
//         Some(Token::Identifier(typ)) => match typ {
//             "double" => Ok(FieldType::Double),
//             "float" => Ok(FieldType::Float),
//             "int32" => Ok(FieldType::Int32),
//             "int64" => Ok(FieldType::Int64),
//             "uint32" => Ok(FieldType::UInt32),
//             "uint64" => Ok(FieldType::UInt64),
//             "sint32" => Ok(FieldType::SInt32),
//             "sint64" => Ok(FieldType::SInt64),
//             "fixed32" => Ok(FieldType::Fixed32),
//             "fixed64" => Ok(FieldType::Fixed64),
//             "sfixed32" => Ok(FieldType::SFixed32),
//             "sfixed64" => Ok(FieldType::SFixed64),
//             "bool" => Ok(FieldType::Bool),
//             "string" => Ok(FieldType::String),
//             "bytes" => Ok(FieldType::Bytes),
//             _ => Ok(FieldType::MessageOrEnum(typ.to_string())),
//         },
//         Some(Token::Map) => {
//             expect_token(tokens, &Token::LessThan)?;
//             let key_type = parse_field_type(tokens)?;
//             expect_token(tokens, &Token::Comma)?;
//             let value_type = parse_field_type(tokens)?;
//             expect_token(tokens, &Token::GreaterThan)?;
//             Ok(FieldType::Map(Box::new(key_type), Box::new(value_type)))
//         }
//         _ => Err(ParseError::ExpectedToken("field type".to_string())),
//     }
// }

fn parse_enum_value<'a, I>(tokens: &mut Peekable<I>) -> Result<EnumValue, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let name = expect_identifier(tokens)?;
    expect_token(tokens, &Token::Equals)?;
    let number = parse_integer(tokens)?;
    expect_token(tokens, &Token::Semicolon)?;

    Ok(EnumValue {
        name,
        number,
        options: Vec::new(), // TODO: Parse enum value options
    })
}

fn parse_method<'a, I>(tokens: &mut Peekable<I>) -> Result<Method, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let name = expect_identifier(tokens)?;
    expect_token(tokens, &Token::OpenParen)?;
    let input_type = expect_identifier(tokens)?;
    expect_token(tokens, &Token::CloseParen)?;
    expect_token(tokens, &Token::Returns)?;
    expect_token(tokens, &Token::OpenParen)?;
    let output_type = expect_identifier(tokens)?;
    expect_token(tokens, &Token::CloseParen)?;

    let mut method = Method {
        name,
        input_type,
        output_type,
        client_streaming: false,
        server_streaming: false,
        options: Vec::new(),
    };

    if let Some(Token::OpenBrace) = tokens.peek() {
        tokens.next();
        while let Some(token) = tokens.peek() {
            match token {
                Token::CloseBrace => {
                    tokens.next();
                    break;
                }
                Token::Option => {
                    parse_option(tokens, &mut method.options)?;
                }
                _ => return Err(ParseError::UnexpectedToken(format!("{:?}", token))),
            }
        }
    } else {
        expect_token(tokens, &Token::Semicolon)?;
    }

    Ok(method)
}

fn parse_reserved<'a, I>(
    tokens: &mut Peekable<I>,
    reserved: &mut Vec<crate::parser::ast::Reserved>,
) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.next().ok_or(ParseError::UnexpectedEndOfInput)?; // Consume 'reserved' token

    loop {
        match tokens.next() {
            Some(Token::IntLiteral(_)) => {
                let start = parse_integer(tokens)?;
                if let Some(Token::To) = tokens.peek() {
                    tokens.next(); // Consume 'to' token
                    let end = parse_integer(tokens)?;
                    if start <= end {
                        reserved.push(crate::parser::ast::Reserved::Range(start, end));
                    } else {
                        return Err(ParseError::InvalidRange(start, end));
                    }
                } else {
                    reserved.push(Number(start));
                }
            }
            Some(Token::StringLiteral(name)) => {
                reserved.push(FieldName(name.to_string()));
            }
            Some(Token::Semicolon) => break,
            Some(Token::Comma) => continue,
            Some(token) => return Err(ParseError::UnexpectedToken(format!("{:?}", token))),
            None => return Err(ParseError::UnexpectedEndOfInput),
        }
    }
    Ok(())
}

fn expect_token<'a, I>(tokens: &mut Peekable<I>, expected: &Token) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    match tokens.next() {
        Some(ref token) if token == expected => Ok(()),
        Some(token) => Err(ParseError::UnexpectedToken(format!(
            "Expected {:?}, found {:?}",
            expected, token
        ))),
        None => Err(ParseError::UnexpectedEndOfInput),
    }
}

fn expect_identifier<'a, I>(tokens: &mut Peekable<I>) -> Result<String, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    match tokens.next() {
        Some(Token::Identifier(name)) => Ok(name.to_string()),
        Some(token) => Err(ParseError::UnexpectedToken(format!(
            "Expected identifier, found {:?}",
            token
        ))),
        None => Err(ParseError::UnexpectedEndOfInput),
    }
}

fn parse_integer<'a, I>(tokens: &mut Peekable<I>) -> Result<i32, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    match tokens.next() {
        Some(Token::IntLiteral(value)) => Ok(value as i32),
        Some(token) => Err(ParseError::UnexpectedToken(format!(
            "Expected integer, found {:?}",
            token
        ))),
        None => Err(ParseError::UnexpectedEndOfInput),
    }
}

fn parse_constant<'a, I>(tokens: &mut Peekable<I>) -> Result<String, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    match tokens.next() {
        Some(Token::Identifier(value)) | Some(Token::StringLiteral(value)) => Ok(value.to_string()),
        Some(Token::IntLiteral(value)) => Ok(value.to_string()),
        Some(Token::FloatLiteral(value)) => Ok(value.to_string()),
        Some(token) => Err(ParseError::UnexpectedToken(format!(
            "Expected constant, found {:?}",
            token
        ))),
        None => Err(ParseError::UnexpectedEndOfInput),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proto_file() {
        let input = r#"
            syntax = "proto3";
            package example;

            message Person {
                string name = 1;
                int32 age = 2;
                repeated string hobbies = 3;
            }

            enum Gender {
                UNKNOWN = 0;
                MALE = 1;
                FEMALE = 2;
            }
        "#;

        let result = parse_proto_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse proto file: {:?}",
            result.err()
        );

        let proto_file = result.unwrap();
        assert_eq!(proto_file.syntax, crate::parser::ast::Syntax::Proto3);
        assert_eq!(proto_file.package, Some("example".to_string()));
        assert_eq!(proto_file.messages.len(), 1);
        assert_eq!(proto_file.enums.len(), 1);

        // Add more specific assertions based on the expected structure of your AST
    }

    // Add more tests for individual parsing functions
}
