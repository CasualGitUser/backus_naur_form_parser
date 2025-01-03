# backus_naur_form_parser
Uses backus naur forms to parse and compile strings.

## Example

The BackusNaurForm struct represents a backus naur form. To create one, the macro is recommended.
A rule consists of up to 3 things:
1. A priority. Rules with higher priority get applied first. (denoted by "priority number] =>")
2. The rule. This is a raw string literal (r#"string comes here"#) so it can contain strings. The form is just like bnf: r#"<symbol> ::= "terminal string" "terminal string2" | "abc" <non_terminal | ..."#
3. OPTIONAL: A function that compiles this token if its at the uppermost level in the AST. More info in the code documentation. This is a closure that takes a token and the backus naur form as arguments and returns a String. (denoted by a => |token, bnf| {})

The below code is a example that doubles the digits of a equation <digit><operator><digit> and replaces the <operator> with "<here comes the operator>"
```rust
let bnf = backus_naur_form!(
            priority 0 => r#"<digit> ::= "1" | "2" | "3""# => |digit_token, _bnf| {
                    (digit_token
                        .get_terminals()
                        .parse::<usize>()
                        .expect("failed to parse <digit> to usize")
                        * 2)
                    .to_string()
                }
            priority 0 => r#"<operator> ::= "+" | "-" | "*" | "/""#
            priority 0 => r#"<expression> ::= <digit> <operator> <digit>"# => |token, _bnf| {
                    let digits =
                        token.get_child_tokens_of_type(&Symbol::NonTerminal("digit".to_string()));
                    let _operator =
                        token.get_child_tokens_of_type(&Symbol::NonTerminal("operator".to_string()));
                    // let t = token
                    let parsed_and_doubled = digits
                        .into_iter()
                        .map(|digit| {
                            (digit
                                .get_terminals()
                                .parse::<usize>()
                                .expect("failed to parse <digit> to usize")
                                * 2)
                            .to_string()
                        })
                        .collect::<Vec<_>>();

                    let a = parsed_and_doubled
                        .first()
                        .expect("failed to parse first digit")
                        .to_owned()
                        + "<here comes the operator>"
                        + parsed_and_doubled
                            .last()
                            .expect("failed to parse last digit");
                    println!("{a}");
                    a
                }
        );
assert_eq!(
            bnf.compile_string("2+3"),
            "4<here comes the operator>6".to_string()
        );
```
