# Protobuf to Zod (WIP)

## Overview

The Protobuf to Zod Converter is a Rust-based tool designed to transform Protocol Buffer (protobuf) definitions into Zod schemas. This project aims to bridge the gap between the widely used protobuf format and the TypeScript-first schema validation library, Zod.

## Features and Design Goals

- [x] Parse Protocol Buffer (version 3) files
- [x] Support for messages, enums, nested types, and more
- [x] Generate corresponding Zod schemas
- [x] Extract validation metadata from special comment annotations
- [x] Supports TypeScript/Zod schema generation
- [x] Robust error handling and reporting
- [x] Integration with Buf plugin system
- [ ] Support for Python/Pydantic schema generation

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [Buf Plugin Integration](#buf-plugin-integration)
- [Contributing](#contributing)
- [License](#license)

## Installation

To set up the Protobuf to Zod Converter, follow these steps:

1. Ensure you have Rust installed on your system. If not, [install Rust](https://www.rust-lang.org/tools/install).

2. Clone the repository:
   ```
   git clone https://github.com/olegakbarov/protobuf_to_zod.git
   cd protobuf_to_zod
   ```

3. Build the project:
   ```
   cargo build --release
   ```

## Developing

```bash
# Run with trace logging
RUST_LOG=trace cargo run -- --input files/with-zod-comments.proto

# Run tests
cargo test

# Run linting
cargo clippy

# Build as Buf plugin and test
cargo build
buf generate
```

## Usage

To use the Protobuf to Zod Converter:

1. You can convert your `.proto` files in several ways:

   a. Run directly with a specific input file:
   ```
   cargo run -- --input files/with-zod-comments.proto
   ```

   b. Run with debug logging for more detailed output:
   ```
   RUST_LOG=debug cargo run -- --input files/with-zod-comments.proto
   ```

   c. Use as a Buf plugin (recommended for larger projects):
   ```
   # Configure your buf.gen.yaml file first
   buf generate
   ```

2. The converter will generate TypeScript/Zod schemas in the specified output directory.

### Configuration Options

The tool supports several configuration options via command line:

```
--input FILE              Input proto file
--output-dir DIRECTORY    Output directory for generated files
--typescript              Generate TypeScript/Zod schemas
--python                  Generate Python/Pydantic schemas (not yet implemented)
--import-style STYLE      Import style for Zod (default, named, namespace)
--config FILE             Custom config file
```

### Zod Comment Format

You can add special comments in your proto files to add validation metadata:

```protobuf
// File-level annotation
syntax = "proto3"; // @zod-version: 1.0

// Message-level annotation
message User // @zod { description: "Represents a user in the system" }
{
  // Field-level annotations
  string username = 1; // @zod { min: 3, max: 50, description: "User's username" }
  string email = 2; // @zod { email: true, description: "User's email address" }
  int32 age = 3; // @zod { min: 0, max: 120, description: "User's age in years" }
  repeated string tags = 5; // @zod { array: { min: 1, max: 10 }, description: "User's tags" }
  UserType type = 6; // @zod { description: "User type", default: "STANDARD" }
}
```

## Project Structure

The project is structured as follows:

- `/src`: Contains the main source code
  - `main.rs`: Entry point for the CLI application
  - `lib.rs`: Core library functionality
  - `/buf`: Contains Buf plugin integration code
  - `/parser`: Contains the protobuf parser implementation
    - `mod.rs`: Main parser functionality
    - `ast.rs`: Abstract Syntax Tree definitions
    - `lexer.rs`: Tokenizer for protobuf files
    - `error.rs`: Error handling for the parser
    - `tests.rs`: Parser tests
  - `/visitor`: Visitor pattern implementation for traversing the AST
    - `visitor.rs`: Visitor trait and implementation
  - `/zod`: Zod schema generation
    - `mod.rs`: Module definitions
    - `metadata.rs`: Zod metadata structures
    - `parser.rs`: Parser for Zod annotations
    - `generator.rs`: Zod schema generator
    - `writer.rs`: Output writer
    - `config.rs`: Configuration system
    - `tests.rs`: Zod-related tests
- `/files`: Contains sample protobuf files
  - Sample files with and without Zod annotations
- `/proto`: Contains protocol definitions
  - `/buf`: Contains Buf plugin protocol definitions
- `build.rs`: Build script for protocol compilation
- `buf.yaml`: Buf workspace configuration
- `buf.gen.yaml`: Buf generation configuration
- `Cargo.toml`: Rust package manifest

## Buf Plugin Integration

This tool can be used as a [Buf](https://buf.build) plugin, which makes it easy to integrate with the broader Protocol Buffer ecosystem.

### Setup

1. Configure your `buf.gen.yaml` file:

```yaml
version: v1
plugins:
  - name: protobuf-to-zod
    path: ./target/debug/protobuf_to_zod  # Path to the compiled binary
    out: generated                       # Output directory
    opt: 
      - import_style=named              # Options for the plugin
      - typescript=true
      - single_file=true
```

2. Run the Buf generation command:

```bash
buf generate
```

This will process all the protocol files in your project according to your Buf configuration and output the Zod schemas to the specified directory.

## Contributing

Contributions to the Protobuf to Zod Converter are welcome! Here's how you can contribute:

1. Fork the repository
2. Create a new branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Commit your changes (`git commit -m 'Add some amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

Please ensure your code adheres to the existing style and passes all tests.

## License

This project is licensed under the [MIT License](LICENSE).

---

Built with ❤️ by [Oleg Akbarov](https://github.com/olegakbarov) in a beautiful city of San Franciso
