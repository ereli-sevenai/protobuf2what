use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Location { line, column }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Represents all possible errors that can occur during parsing
#[derive(Debug)]
pub enum ParseError {
    /// Error occurred during lexical analysis
    LexerError(String, Location),
    /// Unexpected charachter encountered
    UnexpectedCharacter(char, Location),
    /// Unexpected token encountered
    UnexpectedToken(String, Location),
    /// Expected token not found
    ExpectedToken(String, Location),
    /// Invalid syntax
    InvalidSyntax(String, Location),
    /// Unexpected end of input
    UnexpectedEndOfInput(Location),
    /// Unterminated string literal
    UnterminatedStringLiteral(Location),
    /// Parser couldn't process entire input
    IncompleteParser(String, Location),
    /// Duplicate definition
    DuplicateDefinition(String, Location),
    /// Unknown type referenced
    UnknownType(String, Location),
    /// Missing identifier
    MissingIdentifier(String, Location),
    /// Invalid number range
    InvalidRange(i32, i32, Location),
    /// Invalid field number
    InvalidFieldNumber(String, Location),
    /// Tokenization error
    NomError(String, Location),
    /// Generic error for other cases
    Other(String, Location),
}

impl ParseError {
    pub fn location(&self) -> Location {
        match self {
            ParseError::LexerError(_, loc) => *loc,
            ParseError::UnexpectedToken(_, loc) => *loc,
            ParseError::UnexpectedCharacter(_, loc) => *loc,
            ParseError::ExpectedToken(_, loc) => *loc,
            ParseError::InvalidSyntax(_, loc) => *loc,
            ParseError::UnexpectedEndOfInput(loc) => *loc,
            ParseError::IncompleteParser(_, loc) => *loc,
            ParseError::UnterminatedStringLiteral(loc) => *loc,
            ParseError::DuplicateDefinition(_, loc) => *loc,
            ParseError::UnknownType(_, loc) => *loc,
            ParseError::MissingIdentifier(_, loc) => *loc,
            ParseError::InvalidRange(_, _, loc) => *loc,
            ParseError::InvalidFieldNumber(_, loc) => *loc,
            ParseError::NomError(_, loc) => *loc,
            ParseError::Other(_, loc) => *loc,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ParseError::LexerError(msg, _) => format!("Lexer error: {}", msg),
            ParseError::UnexpectedToken(token, _) => format!("Unexpected token: {}", token),
            ParseError::UnexpectedCharacter(char, _) => format!("Unexpected token: {}", char),
            ParseError::ExpectedToken(token, _) => format!("Expected token: {}", token),
            ParseError::InvalidSyntax(msg, _) => format!("Invalid syntax: {}", msg),
            ParseError::UnterminatedStringLiteral(loc) => {
                format!("Unterminated string literal at {}", loc)
            }
            ParseError::UnexpectedEndOfInput(_) => "Unexpected end of input".to_string(),
            ParseError::IncompleteParser(_, loc) => format!("Incomplete parser: {}", loc),
            ParseError::DuplicateDefinition(name, _) => format!("Duplicate definition: {}", name),
            ParseError::UnknownType(type_name, _) => format!("Unknown type: {}", type_name),
            ParseError::MissingIdentifier(msg, _) => format!("Missing identifier: {}", msg),
            ParseError::InvalidRange(start, end, _) => {
                format!("Invalid range: {} to {}", start, end)
            }
            ParseError::InvalidFieldNumber(msg, _) => format!("Invalid field number: {}", msg),
            ParseError::NomError(msg, _) => format!("Nom error: {}", msg),
            ParseError::Other(msg, _) => format!("Other error: {}", msg),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::LexerError(msg, _) => write!(f, "Lexer error: {}", msg),
            ParseError::UnexpectedCharacter(c, loc) => {
                write!(f, "Unexpected character '{}' at {}", c, loc)
            }
            ParseError::ExpectedToken(token, loc) => {
                write!(f, "Expected token: {} at {}", token, loc)
            }
            ParseError::UnexpectedToken(token, loc) => {
                write!(f, "Unexpected token '{}' at {}", token, loc)
            }
            ParseError::UnexpectedEndOfInput(loc) => {
                write!(f, "Unexpected end of input at {}", loc)
            }
            ParseError::InvalidSyntax(msg, loc) => {
                write!(f, "Invalid syntax, found: {} at {}", msg, loc)
            }
            ParseError::UnterminatedStringLiteral(loc) => {
                write!(f, "Unterminated string literal at {}", loc)
            }
            ParseError::IncompleteParser(_, loc) => {
                write!(
                    f,
                    "Parser couldn't process entire input. Stopped at {}",
                    loc
                )
            }
            ParseError::DuplicateDefinition(name, loc) => {
                write!(f, "Duplicate definition: {} at {}", name, loc)
            }
            ParseError::UnknownType(type_name, loc) => {
                write!(f, "Unknown type: {} at {}", type_name, loc)
            }
            ParseError::MissingIdentifier(expected, loc) => {
                write!(f, "Missing identifier. Expected {} at {}", expected, loc)
            }
            ParseError::InvalidRange(start, end, loc) => {
                write!(f, "Invalid range: {} to {} at {}", start, end, loc)
            }
            ParseError::InvalidFieldNumber(msg, loc) => {
                write!(f, "Invalid field number: {} at {}", msg, loc)
            }
            ParseError::NomError(msg, loc) => {
                write!(f, "Nom error: {} at {}", msg, loc)
            }
            ParseError::Other(msg, loc) => {
                write!(f, "Unknown error: {} at {}", msg, loc)
            }
        }
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for ParseError {
    fn from(error: nom::Err<nom::error::Error<&str>>) -> Self {
        match error {
            nom::Err::Incomplete(_) => {
                ParseError::IncompleteParser("Incomplete input".to_string(), Location::new(0, 0))
            }
            nom::Err::Error(e) | nom::Err::Failure(e) => ParseError::NomError(
                format!("Failed to parse token: {:?}", e.code),
                Location::new(0, 0),
            ),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

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
        write!(f, "Error at {}: {}", self.location, self.error)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::UnexpectedToken(
            "Found 'int', expected 'string'".to_string(),
            Location { line: 1, column: 1 },
        );
        assert_eq!(
            format!("{}", error),
            "Unexpected token: Found 'int', expected 'string'"
        );
    }

    #[test]
    fn test_location_error() {
        let error = ParseError::InvalidSyntax(
            "Missing semicolon".to_string(),
            Location {
                line: 10,
                column: 15,
            },
        );
        let location = SourceLocation {
            line: 10,
            column: 15,
        };
        let loc_error = error_at_location(error, location);
        assert_eq!(
            format!("{}", loc_error),
            "Error at line 10, column 15: Invalid syntax: Missing semicolon"
        );
    }

    #[test]
    fn test_lexer_error_display() {
        let error = ParseError::LexerError(
            "Unexpected character '#'".to_string(),
            Location {
                line: 5,
                column: 20,
            },
        );
        assert_eq!(
            format!("{}", error),
            "Lexer error: Unexpected character '#'"
        );
    }

    #[test]
    fn test_unexpected_end_of_input() {
        let error = ParseError::UnexpectedEndOfInput(Location {
            line: 15,
            column: 1,
        });
        assert_eq!(format!("{}", error), "Unexpected end of input");
    }
}
