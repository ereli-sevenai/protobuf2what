use crate::parser::lexer::Token;
use std::error::Error;

use std::fmt;
use std::iter::Peekable;

/// Represents all possible errors that can occur during parsing
#[derive(Debug)]
pub enum ParseError {
    /// Error occurred during lexical analysis
    LexerError(String),
    /// Unexpected token encountered
    UnexpectedToken(String),
    /// Expected token not found
    ExpectedToken(String),
    /// Invalid syntax
    InvalidSyntax(String),
    /// Unexpected end of input
    UnexpectedEndOfInput,
    /// Parser couldn't process entire input
    IncompleteParser(String),
    /// Duplicate definition
    DuplicateDefinition(String),
    /// Unknown type referenced
    UnknownType(String),
    /// Missing identifier
    MissingIdentifier(String),
    /// Invalid number range
    InvalidRange(i32, i32),
    /// Invalid field number
    InvalidFieldNumber(String),
    /// Generic error for other cases
    Other(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            ParseError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
            ParseError::ExpectedToken(msg) => write!(f, "Expected token: {}", msg),
            ParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::IncompleteParser(msg) => write!(f, "Incomplete parsing: {}", msg),
            ParseError::DuplicateDefinition(msg) => write!(f, "Duplicate definition: {}", msg),
            ParseError::UnknownType(msg) => write!(f, "Unknown type: {}", msg),
            ParseError::InvalidFieldNumber(msg) => write!(f, "Invalid field number: {}", msg),
            ParseError::MissingIdentifier(msg) => write!(f, "Missing identifier: {}", msg),
            ParseError::InvalidRange(start, end) => {
                write!(f, "Invalid range: starg={}, end={}", start, end)
            }
            ParseError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for ParseError {}

/// A Result type specialized for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Represents the location in the source where an error occurred
#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// An error with associated source location
#[derive(Debug)]
pub struct LocationError {
    pub error: ParseError,
    pub location: SourceLocation,
}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.error, self.location)
    }
}

impl Error for LocationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Helper function to create a LocationError
pub fn error_at_location(error: ParseError, location: SourceLocation) -> LocationError {
    LocationError { error, location }
}

fn expect_token<'a, I>(tokens: &mut Peekable<I>, expected: &Token) -> ParseResult<()>
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

fn expect_identifier<'a, I>(tokens: &mut Peekable<I>) -> ParseResult<String>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::UnexpectedToken("Found 'int', expected 'string'".to_string());
        assert_eq!(
            format!("{}", error),
            "Unexpected token: Found 'int', expected 'string'"
        );
    }

    #[test]
    fn test_location_error() {
        let error = ParseError::InvalidSyntax("Missing semicolon".to_string());
        let location = SourceLocation {
            line: 10,
            column: 15,
        };
        let loc_error = error_at_location(error, location);
        assert_eq!(
            format!("{}", loc_error),
            "Invalid syntax: Missing semicolon at line 10, column 15"
        );
    }
}
