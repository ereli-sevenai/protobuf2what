# Protobuf to Zod (WIP)

## Overview

The Protobuf to Zod Converter is a Rust-based tool designed to transform Protocol Buffer (protobuf) definitions into Zod schemas. This project aims to bridge the gap between the widely used protobuf format and the TypeScript-first schema validation library, Zod.

## Features and Design Goals

- [x] Parse Protocol Buffer (version 3) files
- [x] Support for messages, enums, nested types, and more
- [ ] Generate corresponding Zod schemas
- [ ] Robust error handling and reporting

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Project Structure](#project-structure)
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

```
RUST_LOG=trace cargo run
```

## Usage

To use the Protobuf to Zod Converter:

1. Place your `.proto` file in the `files` directory. The project currently includes a sample `simple.proto` file.

2. Run the converter:
   ```
   cargo run
   ```

3. The program will read the `simple.proto` file, parse it, and output "Success!" if the parsing is successful.

Note: The current implementation focuses on parsing the protobuf file. Generation of Zod schemas is a planned feature.

## Project Structure

The project is structured as follows:

- `/src`: Contains the main source code
  - `main.rs`: Entry point of the application
  - `lib.rs`: Defines the library and error types
  - `/parser`: Contains the protobuf parser implementation
    - `mod.rs`: Defines the parser module
    - `ast.rs`: Abstract Syntax Tree definitions
    - `lexer.rs`: Tokenizer for protobuf files
    - `error.rs`: Error handling for the parser
  - `/intermediate`: Contains intermediate representation (currently a placeholder)
- `/files`: Contains sample protobuf files
- `Cargo.toml`: Rust package manifest

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
