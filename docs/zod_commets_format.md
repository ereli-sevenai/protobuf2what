# Specification for Zod Comments in Protobuf:

1. Format:
   The Zod comment should be on the same line as the message or field definition, following this format:
   ```protobuf
   <protobuf_definition> // @zod { <zod_options> }
   ```

2. Placement:
   - For messages: Immediately after the message name and before the opening brace.
   - For fields: At the end of the field definition, after the field number.
   - For enums: Immediately after the enum name and before the opening brace.

3. Zod Options:
   The `<zod_options>` part can include one or more of the following, separated by commas:
   - `min: <number>`
   - `max: <number>`
   - `regex: "<pattern>"`
   - `email: true`
   - `url: true`
   - `uuid: true`
   - `positive: true`
   - `negative: true`
   - `int: true`
   - `description: "<text>"`
   - `default: <value>`
   - `optional: true`
   - `nullable: true`
   - `array: { <array_options> }`

4. Array Options:
   When specifying array options, you can include:
   - `min: <number>`
   - `max: <number>`
   - `length: <number>`

5. Escaping:
   Use backslashes to escape special characters in strings, especially for regex patterns.

6. Nesting:
   Allow nesting of options for complex types like objects and arrays.

7. Version Control:
   Include a version specifier at the top of the Protobuf file:
   ```protobuf
   syntax = "proto3";
   // @zod-version: 1.0
   ```

8. Custom Validators:
   Support custom validation functions using a `custom` option:
   ```protobuf
   string custom_field = 1; // @zod { custom: "myCustomValidator" }
   ```
   The actual implementation of custom validators should be defined in the Zod schema generation process.

9. Documentation:
   Each Zod option should be documented as follows:

   - `min: <number>` - Specifies the minimum value for numbers or length for strings and arrays.
   - `max: <number>` - Specifies the maximum value for numbers or length for strings and arrays.
   - `regex: "<pattern>"` - Defines a regular expression pattern for string validation.
   - `email: true` - Validates that the string is a valid email address.
   - `url: true` - Validates that the string is a valid URL.
   - `uuid: true` - Validates that the string is a valid UUID.
   - `positive: true` - Ensures the number is positive.
   - `negative: true` - Ensures the number is negative.
   - `int: true` - Ensures the number is an integer.
   - `description: "<text>"` - Provides a description for the field or message.
   - `default: <value>` - Sets a default value for the field.
   - `optional: true` - Marks the field as optional.
   - `nullable: true` - Allows the field to be null.
   - `array: { <array_options> }` - Specifies options for array fields.
   - `custom: "<function_name>"` - Specifies a custom validation function.

10. Integration with Other Systems:
    To facilitate integration with other validation or documentation systems, consider the following:
    - OpenAPI: Include an option to generate OpenAPI specifications from Zod comments.
    - JSON Schema: Provide a way to output JSON Schema from Zod comments.
    - Documentation Generation: Support automatic generation of documentation from Zod comments.

Examples:

```protobuf
syntax = "proto3";
// @zod-version: 1.0

message User // @zod { description: "Represents a user in the system" }
{
  string username = 1; // @zod { min: 3, max: 50, description: "User's username" }
  string email = 2; // @zod { email: true, description: "User's email address" }
  int32 age = 3; // @zod { min: 0, max: 120, description: "User's age in years" }
  string password = 4; // @zod { regex: "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d]{8,}$", description: "User's password" }
  repeated string tags = 5; // @zod { array: { min: 1, max: 10 }, description: "User's tags" }
  UserType type = 6; // @zod { description: "User type", default: "STANDARD" }
  string website = 7; // @zod { url: true, optional: true, description: "User's website" }
  string custom_field = 8; // @zod { custom: "myCustomValidator", description: "Field with custom validation" }
}

enum UserType // @zod { description: "Types of users in the system" }
{
  STANDARD = 0;
  ADMIN = 1;
  MODERATOR = 2;
}

message ComplexObject // @zod { description: "A complex object example" }
{
  message NestedObject // @zod { description: "A nested object" }
  {
    string nested_field = 1; // @zod { min: 1, max: 100 }
  }

  NestedObject nested = 1; // @zod { description: "A nested object field" }
  repeated int32 numbers = 2; // @zod { array: { min: 5, max: 10 }, description: "A list of numbers" }
}
