# backus_naur_form_parser
Uses backus naur forms to parse and compile strings.

## Example

The BackusNaurForm struct represents a backus naur form. To create one, the macro is recommended:
```rust
let bnf = backus_naur_form!(
            priority 0 => r#"<digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0""#
            priority 0 => r#"<number> ::= <digit> | <number> <number>"#
            priority 1 => r#"<quotient> ::= <number> "/" <number>
            | <expression> "/" <number>
            | <number> "/" <expression>
            | <expression> "/" <expression>"#
            priority 1 => r#"<product> ::= <number> "*" <number>
            | <expression> "*" <number>
            | <number> "*" <expression>
            | <expression> "*" <expression>"#
            priority 0 => r#"<sum> ::= <number> "+" <number>
            | <expression> "+" <number>
            | <number> "+" <expression>
            | <expression> "+" <expression>"#
            priority 0 => r#"<difference> ::= <number> "-" <number>
            | <expression> "-" <number>
            | <number> "-" <expression>
            | <expression> "-" <expression>"#
            priority 0 => r#"<expression> ::= <quotient> | <product> | <sum> | <difference>"#
        );
```
