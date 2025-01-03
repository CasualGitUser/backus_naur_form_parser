use super::{
    symbol::{non_terminal_symbol::NonTerminalSymbol, Symbol},
    Choice, Expression,
};

//used to get a certain choice from a symbolized expression
fn get_choice(expr: &mut Expression, choice_index: usize) -> &mut Choice {
    match expr.get(choice_index) {
        Some(_) => expr.get_mut(choice_index).unwrap(),
        None => {
            expr.insert(choice_index, Vec::new());
            expr.get_mut(choice_index).unwrap()
        }
    }
}

///creates a new rule from a string
///Rules are built like this: `<symbol>` ::= expression
///The expression may contain any ammoutn of symbols
pub(super) fn non_terminal_symbol_from_rule(string: &str) -> NonTerminalSymbol {
    let Some((symbol_name, expression)) = string.split_once("::=") else {
        panic!("the replacement operator (::=) is missing or invalid in the rule {string}");
    };
    //trim the whitespace
    let symbol_name = symbol_name.trim();
    //remove the angle brackets
    let symbol_name = &symbol_name[1..symbol_name.len() - 1];
    //indicates wether we are going through a string.
    //for example: (a "|" pipe indicates the current index)
    //"this is a se|ntence" would mean its in a string aka true
    //<sym|bol> "string literal" would mean its not in a string aka false
    //this is to ensure that brackets and pipes can be put into strings
    let mut in_string: bool = false;
    //used to indicate the beginning of a string if in_string is true
    let mut last_string_indice: usize = 0;
    //used to indicate the beginning of a symbol if in_string is false
    let mut last_opening_bracket_indice: usize = 0;
    //stores the symbolized expression
    let mut symbolized_expression: Expression = Vec::new();
    //stores the current choice
    //for example the expression: <symbol1> "abc" | "def"
    //if it was currently on the left side of the pipe, it would be choice_index 0
    //if it was currently on the right side of the pipe, it would be choice_index 1
    let mut choice_index: usize = 0;
    for (index, ch) in expression.char_indices() {
        match ch {
            //opening double quote
            '"' if !in_string => {
                last_string_indice = index;
                in_string = true
            }
            //closing double quote
            '"' if in_string => {
                let choice = get_choice(&mut symbolized_expression, choice_index);
                choice.push(Symbol::Terminal(
                    expression[last_string_indice + 1..index].to_string(),
                ));
                in_string = false
            }
            //opening bracket
            '<' if !in_string => last_opening_bracket_indice = index,
            //closing bracket
            '>' => {
                let choice = get_choice(&mut symbolized_expression, choice_index);
                choice.push(Symbol::NonTerminal(
                    expression[last_opening_bracket_indice + 1..index].to_string(),
                ));
            }
            //choice symbol
            '|' if !in_string => choice_index += 1,
            _ => (),
        }
    }
    NonTerminalSymbol::new(symbol_name.to_string(), symbolized_expression)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_non_terminal_symbol_from_rule() {
        let rule = r#"<test> ::= "a" "b" "c" | "c" "b" "a" | <abc>"#;
        let non_terminal_symbol = non_terminal_symbol_from_rule(rule);
        assert_eq!(
            non_terminal_symbol,
            NonTerminalSymbol::new(
                "test".to_string(),
                vec![
                    vec![
                        Symbol::Terminal("a".to_string()),
                        Symbol::Terminal("b".to_string()),
                        Symbol::Terminal("c".to_string())
                    ],
                    vec![
                        Symbol::Terminal("c".to_string()),
                        Symbol::Terminal("b".to_string()),
                        Symbol::Terminal("a".to_string())
                    ],
                    vec![Symbol::NonTerminal("abc".to_string())]
                ]
            )
        )
    }
}
