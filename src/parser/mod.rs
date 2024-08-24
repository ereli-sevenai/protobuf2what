//! Parser module for Protobuf to Zod converter
//!
//! This module contains the lexer, AST definitions, and parsing logic
//! for processing Protobuf files.

pub mod ast;
pub mod error;
mod lexer;

use crate::parser::ast::{
    Enum, EnumValue, Field, FieldLabel, Import, ImportKind, Message, Method, OptionValue,
    ProtoFile, ProtoOption, Service, Syntax,
};

use ast::{EnumValueOption, EnumValueOptionValue, FieldOptionValue, FieldType};
use error::Location;
pub use error::{ParseError, ParseResult};
pub use lexer::{tokenize, Token, TokenWithLocation};

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
    let tokens = tokenize(input).map_err(ParseError::from)?;

    let mut token_iter = tokens.into_iter().peekable();

    parse_tokenized_input(&mut token_iter)
}

fn parse_tokenized_input<'a, I>(tokens: &mut Peekable<I>) -> Result<ProtoFile, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    let mut proto_file = ProtoFile::new();

    while let Some(current_token) = tokens.peek() {
        match &current_token.token {
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
            _ => {
                let loc = current_token.location.clone();
                return Err(ParseError::UnexpectedToken(
                    format!("{:?}", current_token.token),
                    loc,
                ));
            }
        }
    }

    Ok(proto_file)
}

fn parse_syntax<'a, I>(
    tokens: &mut Peekable<I>,
    proto_file: &mut ProtoFile,
) -> Result<(), ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Consume 'syntax' token
    let syntax_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;
    if syntax_token.token != Token::Syntax {
        return Err(ParseError::UnexpectedToken(
            format!("Expected 'syntax', found {:?}", syntax_token.token),
            syntax_token.location,
        ));
    }

    // Expect '=' token
    let equals_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(syntax_token.location))?;
    if equals_token.token != Token::Equals {
        return Err(ParseError::UnexpectedToken(
            format!("Expected '=', found {:?}", equals_token.token),
            equals_token.location,
        ));
    }

    // Parse syntax version
    let version_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(equals_token.location))?;
    match version_token.token {
        Token::StringLiteral("proto2") => proto_file.syntax = Syntax::Proto2,
        Token::StringLiteral("proto3") => proto_file.syntax = Syntax::Proto3,
        _ => {
            return Err(ParseError::InvalidSyntax(
                "Expected \"proto2\" or \"proto3\"".to_string(),
                version_token.location,
            ))
        }
    }

    // Expect semicolon
    let semicolon_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(version_token.location))?;
    if semicolon_token.token != Token::Semicolon {
        return Err(ParseError::UnexpectedToken(
            format!("Expected ';', found {:?}", semicolon_token.token),
            semicolon_token.location,
        ));
    }

    Ok(())
}

fn parse_package<'a, I>(
    tokens: &mut Peekable<I>,
    proto_file: &mut ProtoFile,
) -> Result<(), ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Consume 'package' token
    let package_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Package)?;

    // Parse package name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(package_token.location))?;
    match name_token.token {
        Token::Identifier(package_name) => {
            proto_file.package = Some(package_name.to_string());
        }
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected package name, found {:?}", name_token.token),
                name_token.location,
            ));
        }
    }

    // Expect semicolon
    let semicolon_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?;
    if semicolon_token.token != Token::Semicolon {
        return Err(ParseError::UnexpectedToken(
            format!("Expected ';', found {:?}", semicolon_token.token),
            semicolon_token.location,
        ));
    }

    Ok(())
}

fn parse_import<'a, I>(
    tokens: &mut Peekable<I>,
    proto_file: &mut ProtoFile,
) -> Result<(), ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    let import_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;
    if import_token.token != Token::Import {
        return Err(ParseError::UnexpectedToken(
            format!("Expected 'import', found {:?}", import_token.token),
            import_token.location,
        ));
    }

    let mut kind = ImportKind::Default;
    let next_token = tokens
        .peek()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(import_token.location))?;

    match next_token.token {
        Token::Public => {
            tokens.next(); // Consume 'public' token
            kind = ImportKind::Public;
        }
        Token::Weak => {
            tokens.next(); // Consume 'weak' token
            kind = ImportKind::Weak;
        }
        _ => {} // Default import, no modifier
    }

    // Parse import path
    let path_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(import_token.location))?;
    let path = match path_token.token {
        Token::StringLiteral(path) => path.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!(
                    "Expected string literal for import path, found {:?}",
                    path_token.token
                ),
                path_token.location,
            ));
        }
    };

    // Expect semicolon
    let _semicolon_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(path_token.location))?
        .expect(Token::Semicolon)?;

    // Add the import to the proto file
    proto_file.imports.push(Import { path, kind });

    Ok(())
}

fn parse_message<'a, I>(tokens: &mut Peekable<I>) -> Result<Message, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Expect 'message' keyword
    let message_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Message)?;

    // Parse message name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(message_token.location))?;

    let name = match name_token.token {
        Token::Identifier(name) => name.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected message name, found {:?}", name_token.token),
                name_token.location,
            ))
        }
    };

    // Expect opening brace
    let open_brace_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::OpenBrace)?;

    let mut message = Message::new(name);

    // Parse message body
    while let Some(token_with_location) = tokens.peek() {
        match &token_with_location.token {
            Token::CloseBrace => {
                tokens.next(); // Consume closing brace
                return Ok(message);
            }
            Token::Message => {
                let nested_message = parse_message(tokens)?;
                message.nested_messages.push(nested_message);
            }
            Token::Enum => {
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

    Err(ParseError::UnexpectedEndOfInput(open_brace_token.location))
}

fn parse_field<'a, I>(tokens: &mut Peekable<I>) -> Result<Field, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    let start_location = tokens
        .peek()
        .map(|t| t.location)
        .unwrap_or(Location::new(0, 0));

    // Parse field label (optional, repeated, required)
    let label = match tokens.peek().map(|t| &t.token) {
        Some(Token::Repeated) => {
            tokens.next();
            FieldLabel::Repeated
        }
        Some(Token::Required) => {
            tokens.next();
            FieldLabel::Required
        }
        _ => FieldLabel::Optional,
    };

    // Parse field type
    let type_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(start_location))?;
    let typ = parse_field_type(&type_token)?;

    // Parse field name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(type_token.location))?;
    let name = match name_token.token {
        Token::Identifier(name) => name.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected field name, found {:?}", name_token.token),
                name_token.location,
            ))
        }
    };

    // Expect '='
    let equals_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Equals)?;

    // Parse field number
    let number_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(equals_token.location))?;
    let number = match number_token.token {
        Token::IntLiteral(num) => num,
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected field number, found {:?}", number_token.token),
                number_token.location,
            ))
        }
    };

    // Expect semicolon
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(number_token.location))?
        .expect(Token::Semicolon)?;

    Ok(Field {
        name,
        number,
        label,
        typ,
        options: Vec::new(), // TODO: Parse field options
    })
}

fn parse_enum<'a, I>(tokens: &mut Peekable<I>) -> Result<Enum, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Expect 'enum' keyword
    let enum_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Enum)?;

    // Parse enum name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(enum_token.location))?;
    let name = match &name_token.token {
        Token::Identifier(name) => name.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected enum name, found {:?}", name_token.token),
                name_token.location,
            ))
        }
    };

    // Expect opening brace
    let open_brace_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::OpenBrace)?;

    let mut enum_def = Enum::new(name);

    while let Some(token_with_location) = tokens.peek() {
        match &token_with_location.token {
            Token::CloseBrace => {
                tokens.next(); // Consume closing brace
                return Ok(enum_def);
            }
            Token::Identifier(_) => {
                // Parse enum value
                let value = parse_enum_value(tokens)?;
                enum_def.values.push(value);
            }
            Token::Option => {
                parse_enum_option(tokens, &mut enum_def.options)?;
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    format!(
                        "Unexpected token in enum body: {:?}",
                        token_with_location.token
                    ),
                    token_with_location.location,
                ));
            }
        }
    }

    Err(ParseError::UnexpectedEndOfInput(open_brace_token.location))
}

fn parse_enum_value<'a, I>(tokens: &mut Peekable<I>) -> Result<EnumValue, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Parse enum value name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;
    let name = match &name_token.token {
        Token::Identifier(name) => name.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected enum value name, found {:?}", name_token.token),
                name_token.location,
            ))
        }
    };

    // Expect '='
    let equals_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?;
    if equals_token.token != Token::Equals {
        return Err(ParseError::UnexpectedToken(
            format!("Expected '=', found {:?}", equals_token.token),
            equals_token.location,
        ));
    }

    // Parse enum value number
    let number_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(equals_token.location))?;
    let number = match &number_token.token {
        Token::IntLiteral(num) => *num,
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected integer, found {:?}", number_token.token),
                number_token.location,
            ))
        }
    };

    // Parse options if present
    let mut options = Vec::new();
    if let Some(TokenWithLocation {
        token: Token::OpenBracket,
        ..
    }) = tokens.peek()
    {
        tokens.next(); // Consume '['
        options = parse_enum_value_options(tokens)?;
    }

    // Expect semicolon
    let semicolon_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(number_token.location))?;
    if semicolon_token.token != Token::Semicolon {
        return Err(ParseError::UnexpectedToken(
            format!("Expected ';', found {:?}", semicolon_token.token),
            semicolon_token.location,
        ));
    }

    Ok(EnumValue {
        name,
        number: number.try_into().unwrap(),
        options,
    })
}

fn parse_enum_value_options<'a, I>(
    tokens: &mut Peekable<I>,
) -> Result<Vec<EnumValueOption>, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    let mut options = Vec::new();

    loop {
        let option = parse_enum_value_option(tokens)?;
        options.push(option);

        match tokens.peek() {
            Some(TokenWithLocation {
                token: Token::Comma,
                ..
            }) => {
                tokens.next(); // Consume comma
            }
            Some(TokenWithLocation {
                token: Token::CloseBracket,
                ..
            }) => {
                tokens.next(); // Consume closing bracket
                break;
            }
            Some(t) => {
                return Err(ParseError::UnexpectedToken(
                    format!("Expected ',' or ']', found {:?}", t.token),
                    t.location,
                ))
            }
            None => return Err(ParseError::UnexpectedEndOfInput(Location::new(0, 0))),
        }
    }

    Ok(options)
}

fn parse_enum_value_option<'a, I>(tokens: &mut Peekable<I>) -> Result<EnumValueOption, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Parse option name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;
    let name = match &name_token.token {
        Token::Identifier(name) => name.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected option name, found {:?}", name_token.token),
                name_token.location,
            ))
        }
    };

    // Expect '='
    let equals_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?;
    if equals_token.token != Token::Equals {
        return Err(ParseError::UnexpectedToken(
            format!("Expected '=', found {:?}", equals_token.token),
            equals_token.location,
        ));
    }

    // Parse option value
    let value_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(equals_token.location))?;
    let value = match &value_token.token {
        Token::StringLiteral(s) => s.to_string(),
        Token::IntLiteral(i) => i.to_string(),
        Token::FloatLiteral(f) => f.to_string(),
        Token::Identifier(id) => id.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected option value, found {:?}", value_token.token),
                value_token.location,
            ))
        }
    };

    Ok(EnumValueOption {
        name,
        value: EnumValueOptionValue::Identifier(value),
    })
}

fn parse_service<'a, I>(tokens: &mut Peekable<I>) -> Result<Service, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Expect 'service' keyword
    let service_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Service)?;

    // Parse service name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(service_token.location))?;

    // Expect opening brace
    let open_brace_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::OpenBrace)?;

    let mut service = Service::new(name_token.token.to_string());

    while let Some(token_with_location) = tokens.peek() {
        match &token_with_location.token {
            Token::CloseBrace => {
                tokens.next(); // Consume closing brace
                return Ok(service);
            }
            Token::Option => {
                parse_option(tokens, &mut service.options)?;
            }
            Token::Rpc => {
                tokens.next(); // Consume 'rpc' token
                let method = parse_method(tokens)?;
                service.methods.push(method);
            }
            _ => {
                return Err(ParseError::UnexpectedToken(
                    format!(
                        "Unexpected token in service body: {:?}",
                        token_with_location.token
                    ),
                    token_with_location.location,
                ));
            }
        }
    }

    Err(ParseError::UnexpectedEndOfInput(open_brace_token.location))
}

fn parse_enum_option<'a, I>(
    tokens: &mut Peekable<I>,
    options: &mut Vec<EnumValueOption>,
) -> Result<(), ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Consume 'option' token
    let option_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Option)?;

    // Parse option name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(option_token.location))?;

    // Expect equals sign
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Equals)?;

    // Parse option value
    let value = parse_constant(tokens)?;

    // Expect semicolon
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Semicolon)?;

    options.push(EnumValueOption {
        name: name_token.token.to_string(),
        value: EnumValueOptionValue::Identifier(value),
    });

    Ok(())
}

fn parse_option<'a, I>(
    tokens: &mut Peekable<I>,
    options: &mut Vec<ProtoOption>,
) -> Result<(), ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Consume 'option' token
    let option_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Option)?;

    // Parse option name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(option_token.location))?;

    // Expect equals sign
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Equals)?;

    // Parse option value
    let value = parse_constant(tokens)?;

    // Expect semicolon
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Semicolon)?;

    options.push(ProtoOption {
        name: name_token.token.to_string(),
        value: OptionValue::String(value),
    });

    Ok(())
}

fn parse_field_type(token: &TokenWithLocation) -> Result<FieldType, ParseError> {
    match &token.token {
        Token::Identifier(typ) => match *typ {
            "double" => Ok(FieldType::Double),
            "float" => Ok(FieldType::Float),
            "int32" => Ok(FieldType::Int32),
            "int64" => Ok(FieldType::Int64),
            "uint32" => Ok(FieldType::UInt32),
            "uint64" => Ok(FieldType::UInt64),
            "sint32" => Ok(FieldType::SInt32),
            "sint64" => Ok(FieldType::SInt64),
            "fixed32" => Ok(FieldType::Fixed32),
            "fixed64" => Ok(FieldType::Fixed64),
            "sfixed32" => Ok(FieldType::SFixed32),
            "sfixed64" => Ok(FieldType::SFixed64),
            "bool" => Ok(FieldType::Bool),
            "string" => Ok(FieldType::String),
            "bytes" => Ok(FieldType::Bytes),
            _ => Ok(FieldType::MessageOrEnum(typ.to_string())),
        },
        Token::Map => {
            // Handle map type parsing
            // This is a placeholder and should be implemented based on your specific needs
            Ok(FieldType::Map(
                Box::new(FieldType::String),
                Box::new(FieldType::Int32),
            ))
        }
        _ => Err(ParseError::UnexpectedToken(
            format!("Expected field type, found {:?}", token.token),
            token.location,
        )),
    }
}

fn parse_field_option<'a, I>(tokens: &mut Peekable<I>) -> Result<FieldOptionValue, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Parse option name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;
    match &name_token.token {
        Token::Identifier(name) => name.to_string(),
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected option name, found {:?}", name_token.token),
                name_token.location,
            ))
        }
    };

    // Expect '='
    let equals_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Equals)?;

    // Parse option value
    let value_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(equals_token.location))?;

    let value = match &value_token.token {
        Token::StringLiteral(s) => FieldOptionValue {
            name: name_token.token.to_string(),
            value: OptionValue::String(s.to_string()),
        },
        Token::IntLiteral(i) => FieldOptionValue {
            name: name_token.token.to_string(),
            value: OptionValue::Int(*i),
        },
        Token::FloatLiteral(f) => FieldOptionValue {
            name: name_token.token.to_string(),
            value: OptionValue::Float(*f),
        },
        Token::Identifier(id) => FieldOptionValue {
            name: name_token.token.to_string(),
            value: OptionValue::Identifier(id.to_string()),
        },
        _ => {
            return Err(ParseError::UnexpectedToken(
                format!("Expected option value, found {:?}", value_token.token),
                value_token.location,
            ))
        }
    };

    // Return the FieldOptionValue
    Ok(value)
}
fn parse_method<'a, I>(tokens: &mut Peekable<I>) -> Result<Method, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Parse method name
    let name_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;

    // Expect opening parenthesis
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::OpenParen)?;

    // Parse input type
    let input_type_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(name_token.location))?
        .expect(Token::Identifier(&name_token.token.to_string()))?;

    // Expect closing parenthesis
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(input_type_token.location))?
        .expect(Token::CloseParen)?;

    // Expect 'returns' keyword
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(input_type_token.location))?
        .expect(Token::Returns)?;

    // Expect opening parenthesis
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(input_type_token.location))?
        .expect(Token::OpenParen)?;

    // Parse output type
    let output_type_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(input_type_token.location))?;

    // Expect closing parenthesis
    tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(output_type_token.location))?
        .expect(Token::CloseParen)?;

    let mut method = Method {
        name: name_token.token.to_string(),
        input_type: input_type_token.token.to_string(),
        output_type: output_type_token.token.to_string(),
        client_streaming: false,
        server_streaming: false,
        options: Vec::new(),
    };

    if let Some(token_with_location) = tokens.peek() {
        if token_with_location.token == Token::OpenBrace {
            tokens.next(); // Consume open brace
            while let Some(token_with_location) = tokens.peek() {
                match &token_with_location.token {
                    Token::CloseBrace => {
                        tokens.next(); // Consume closing brace
                        break;
                    }
                    Token::Option => {
                        parse_option(tokens, &mut method.options)?;
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            format!(
                                "Unexpected token in method body: {:?}",
                                token_with_location.token
                            ),
                            token_with_location.location,
                        ))
                    }
                }
            }
        } else {
            tokens
                .next()
                .ok_or_else(|| ParseError::UnexpectedEndOfInput(output_type_token.location))?
                .expect(Token::Semicolon)?;
        }
    } else {
        return Err(ParseError::UnexpectedEndOfInput(output_type_token.location));
    }

    Ok(method)
}

fn parse_reserved<'a, I>(
    tokens: &mut Peekable<I>,
    reserved: &mut Vec<crate::parser::ast::Reserved>,
) -> Result<(), ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    // Consume 'reserved' token
    let reserved_token = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?
        .expect(Token::Reserved)?;

    let mut last_location = reserved_token.location;

    loop {
        match tokens.next() {
            Some(token_with_location) => {
                match token_with_location.token {
                    Token::IntLiteral(start) => {
                        let start = start as i32;
                        if let Some(TokenWithLocation {
                            token: Token::To, ..
                        }) = tokens.peek()
                        {
                            tokens.next(); // Consume 'to' token
                            if let Some(TokenWithLocation {
                                token: Token::IntLiteral(end),
                                ..
                            }) = tokens.next()
                            {
                                let end = end as i32;
                                if start <= end {
                                    reserved.push(crate::parser::ast::Reserved::Range(start, end));
                                } else {
                                    return Err(ParseError::InvalidRange(
                                        start,
                                        end,
                                        token_with_location.location,
                                    ));
                                }
                            } else {
                                return Err(ParseError::UnexpectedEndOfInput(last_location));
                            }
                        } else {
                            reserved.push(crate::parser::ast::Reserved::Number(start));
                        }
                    }
                    Token::StringLiteral(name) => {
                        reserved.push(crate::parser::ast::Reserved::FieldName(name.to_string()));
                    }
                    Token::Semicolon => break,
                    Token::Comma => continue,
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            format!(
                                "Unexpected token in reserved: {:?}",
                                token_with_location.token
                            ),
                            token_with_location.location,
                        ))
                    }
                }
                last_location = token_with_location.location;
            }
            None => return Err(ParseError::UnexpectedEndOfInput(last_location)),
        }
    }
    Ok(())
}

fn parse_integer<'a, I>(tokens: &mut Peekable<I>) -> Result<i32, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    let token_with_location = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;
    match token_with_location.token {
        Token::IntLiteral(value) => Ok(value as i32),
        _ => Err(ParseError::UnexpectedToken(
            format!("Expected integer, found {:?}", token_with_location.token),
            token_with_location.location,
        )),
    }
}

fn parse_constant<'a, I>(tokens: &mut Peekable<I>) -> Result<String, ParseError>
where
    I: Iterator<Item = TokenWithLocation<'a>>,
{
    let token_with_location = tokens
        .next()
        .ok_or_else(|| ParseError::UnexpectedEndOfInput(Location::new(0, 0)))?;

    match token_with_location.token {
        Token::Identifier(value) | Token::StringLiteral(value) => Ok(value.to_string()),
        Token::IntLiteral(value) => Ok(value.to_string()),
        Token::FloatLiteral(value) => Ok(value.to_string()),
        _ => Err(ParseError::UnexpectedToken(
            format!("Expected constant, found {:?}", token_with_location.token),
            token_with_location.location,
        )),
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
    }

    #[test]
    fn test_parse_reserved() {
        let input = r#"
            message TestReserved {
                reserved 2, 15, 9 to 11;
                reserved "foo", "bar";
            }
        "#;

        let result = parse_proto_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse proto file: {:?}",
            result.err()
        );

        let proto_file = result.unwrap();
        assert_eq!(proto_file.messages.len(), 1);

        let message = &proto_file.messages[0];
        assert_eq!(message.name, "TestReserved");
        assert_eq!(message.reserved.len(), 5);

        assert!(message
            .reserved
            .contains(&crate::parser::ast::Reserved::Number(2)));
        assert!(message
            .reserved
            .contains(&crate::parser::ast::Reserved::Number(15)));
        assert!(message
            .reserved
            .contains(&crate::parser::ast::Reserved::Range(9, 11)));
        assert!(message
            .reserved
            .contains(&crate::parser::ast::Reserved::FieldName("foo".to_string())));
        assert!(message
            .reserved
            .contains(&crate::parser::ast::Reserved::FieldName("bar".to_string())));
    }

    #[test]
    fn test_parse_field_types() {
        let input = r#"
            message TestFieldTypes {
                double double_field = 1;
                float float_field = 2;
                int32 int32_field = 3;
                int64 int64_field = 4;
                uint32 uint32_field = 5;
                uint64 uint64_field = 6;
                sint32 sint32_field = 7;
                sint64 sint64_field = 8;
                fixed32 fixed32_field = 9;
                fixed64 fixed64_field = 10;
                sfixed32 sfixed32_field = 11;
                sfixed64 sfixed64_field = 12;
                bool bool_field = 13;
                string string_field = 14;
                bytes bytes_field = 15;
                CustomMessage message_field = 16;
                map<string, int32> map_field = 17;
                repeated int32 repeated_field = 18;
            }
        "#;

        let result = parse_proto_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse proto file: {:?}",
            result.err()
        );

        let proto_file = result.unwrap();
        assert_eq!(proto_file.messages.len(), 1);

        let message = &proto_file.messages[0];
        assert_eq!(message.name, "TestFieldTypes");
        assert_eq!(message.fields.len(), 18);

        let expected_types = vec![
            FieldType::Double,
            FieldType::Float,
            FieldType::Int32,
            FieldType::Int64,
            FieldType::UInt32,
            FieldType::UInt64,
            FieldType::SInt32,
            FieldType::SInt64,
            FieldType::Fixed32,
            FieldType::Fixed64,
            FieldType::SFixed32,
            FieldType::SFixed64,
            FieldType::Bool,
            FieldType::String,
            FieldType::Bytes,
            FieldType::MessageOrEnum("CustomMessage".to_string()),
            FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::Int32)),
            FieldType::Int32,
        ];

        for (field, expected_type) in message.fields.iter().zip(expected_types.iter()) {
            assert_eq!(
                &field.typ, expected_type,
                "Field {} has unexpected type",
                field.name
            );
        }

        // Check labels
        assert_eq!(message.fields[17].label, FieldLabel::Repeated);
        for field in &message.fields[0..17] {
            assert_eq!(field.label, FieldLabel::Optional);
        }
    }
}
