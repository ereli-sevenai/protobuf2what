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

Examples:

```protobuf
message User // @zod { description: "Represents a user in the system" }
{
  string username = 1; // @zod { min: 3, max: 50, description: "User's username" }
  string email = 2; // @zod { email: true, description: "User's email address" }
  int32 age = 3; // @zod { min: 0, max: 120, description: "User's age in years" }
  string password = 4; // @zod { regex: "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d]{8,}$", description: "User's password" }
  repeated string tags = 5; // @zod { array: { min: 1, max: 10 }, description: "User's tags" }
  UserType type = 6; // @zod { description: "User type", default: "STANDARD" }
  string website = 7; // @zod { url: true, optional: true, description: "User's website" }
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
```
