use std::ops::Range;

use crate::backus_naur_form::{
    range_from_slice, replace_ranges, rule::non_terminal_symbol_from_rule, token::Token, Choice,
    Expression,
};

use super::Symbol;

///Represents a non terminal symbol.
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct NonTerminalSymbol {
    pub name: String,
    rule: Expression,
}

impl NonTerminalSymbol {
    pub fn new(name: String, rule: Expression) -> Self {
        Self { name, rule }
    }

    ///Creates a [NonTerminalSymbol] from a rule String.
    ///The String is recommended to be a raw string literal if the expression contains [super::super::token::TerminalToken]s.
    pub(crate) fn from_rule(rule: &str) -> Self {
        non_terminal_symbol_from_rule(rule)
    }

    ///Returns the choices that contain the [NonTerminalSymbol] itself.
    fn get_recursive_choices(&self) -> Vec<&Choice> {
        self.rule
            .iter()
            .filter(|choice| choice.contains(&Symbol::NonTerminal(self.name.to_string())))
            .collect()
    }

    ///Returns the choices that don't contain the [NonTerminalSymbol] itself.
    fn get_non_recursive_choices(&self) -> Vec<&Choice> {
        self.rule
            .iter()
            .filter(|choice| !choice.contains(&Symbol::NonTerminal(self.name.to_string())))
            .collect()
    }

    ///this will symbolize the vec
    ///beware with recursive types though, for example `<number> ::= <digit> | <digit> <number>` doesnt work for example
    ///use `<number> ::= <digit> | <number> <number>` instead.
    ///In the number example, some numbers may contain only 1 other number aka its useless stuff so dont depend on some
    ///structure of the symbolized vec.
    ///The only thing you can really be sure of is that if you terminalize the vec it will turn back into its original string.
    ///if you have a symbol (like number) where one is choice is just a different name for a symbol, always use <symbol> <symbol> as recursive option.
    ///otherwise it wont match.
    pub(crate) fn symbolize_vec(&self, vec: &mut Vec<Token>) {
        //this is for non_recursive cases
        let mut ranges = self.get_ranges_of_possible_non_recursive_symbolization(vec);
        replace_ranges(vec, &mut ranges, |replaced_tokens| {
            Token::from_non_terminal(&self.name, replaced_tokens)
        });

        let mut recursive_ranges = self.get_ranges_of_possible_recursive_symbolization(vec);

        loop {
            replace_ranges(vec, &mut recursive_ranges, |replaced_tokens| {
                Token::from_non_terminal(&self.name, replaced_tokens)
            });
            //get new recursive ranges after the ranges in the vec have been replaced
            recursive_ranges = self.get_ranges_of_possible_recursive_symbolization(vec);
            //if there is no more recursive symbolization possible, then stop recursive symbolization
            if recursive_ranges.is_empty() {
                break;
            }
        }
    }

    ///Returns a vector of [Range]s where the [Token]s of the tokenized_vec could be turned into a [NonTerminalToken](super::super::NonTerminalToken)
    ///which is of the type of this [NonTerminalSymbol].  
    ///Each range would index into atleast one [Token] which is of the type of this [NonTerminalSymbol]
    fn get_ranges_of_possible_recursive_symbolization(
        &self,
        tokenized_vec: &[Token],
    ) -> Vec<Range<usize>> {
        let recursive_choices = self.get_recursive_choices();

        Self::get_ranges_from_choices(tokenized_vec, &recursive_choices)
    }

    ///Returns a vector of [Range]s where the [Token]s of the tokenized_vec could be turned into a [NonTerminalToken](super::super::NonTerminalToken)
    ///which is of the type of this [NonTerminalSymbol].  
    ///If indexes into tokenized_vec using the [Range]s, no indexed [Token] would be of the type of this [NonTerminalSymbol].
    fn get_ranges_of_possible_non_recursive_symbolization(
        &self,
        tokenized_vec: &[Token],
    ) -> Vec<Range<usize>> {
        Self::get_ranges_from_choices(tokenized_vec, &self.get_non_recursive_choices())
    }

    ///Returns a vector of [Range]s where the [Token]s of the tokenized_vec could be turned into a [NonTerminalToken](super::super::NonTerminalToken)
    ///which is of the type of this [NonTerminalSymbol].  
    fn get_ranges_of_possible_symbolization(&self, tokenized_vec: &[Token]) -> Vec<Range<usize>> {
        Self::get_ranges_from_choices(tokenized_vec, &self.rule.iter().collect::<Vec<&Choice>>())
    }

    ///Returns true if the a range of [Token]s in the tokenized_vec could be turned into a [NonTerminalToken](super::super::NonTerminalToken)
    ///which is of the type of this [NonTerminalSymbol].
    pub(crate) fn further_symbolization_possible(&self, tokenized_vec: &[Token]) -> bool {
        !self
            .get_ranges_of_possible_symbolization(tokenized_vec)
            .is_empty()
    }

    ///Returns a vector of [Range]s where the [Token]s of each [Range] of it could be summarized using one of the choices.
    fn get_ranges_from_choices(
        tokenized_vec: &[Token],
        choices: &[&Vec<Symbol>],
    ) -> Vec<Range<usize>> {
        choices
            .iter()
            .flat_map(|choice| {
                tokenized_vec
                    .windows(choice.len())
                    .filter(move |window| window == choice)
                    .map(|slice| range_from_slice(tokenized_vec, slice))
            })
            .collect()
    }

    ///Gets the rule that contains the choices that contain the [Symbol]s that can be turned into this [NonTerminalSymbol].
    pub fn get_rule(&self) -> &Expression {
        &self.rule
    }

    ///Returns the name of the [NonTerminalSymbol] aka the string between the angle brackets (<>).  
    ///For example if the [NonTerminalSymbol] is `<number>` this would return "number".
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl PartialEq<NonTerminalSymbol> for Symbol {
    fn eq(&self, other: &NonTerminalSymbol) -> bool {
        match self {
            Symbol::Terminal(_) => false,
            Symbol::NonTerminal(inner) => inner == &other.name,
        }
    }
}

impl PartialEq<Symbol> for NonTerminalSymbol {
    fn eq(&self, other: &Symbol) -> bool {
        other == self
    }
}

#[cfg(test)]
mod tests {
    use crate::backus_naur_form::characterize_string;
    use crate::backus_naur_form::rule::non_terminal_symbol_from_rule;

    use super::*;

    #[test]
    fn test_get_ranges_of_possible_symbolization() {
        let digit = non_terminal_symbol_from_rule(r#"<digit> ::= "1" | "2" | "3""#);
        let tokenized_string = characterize_string("12 3");
        assert_eq!(
            digit.get_ranges_of_possible_symbolization(&tokenized_string),
            vec![0..1, 1..2, 3..4]
        );
    }

    #[test]
    fn test_symbolization() {
        let digit = non_terminal_symbol_from_rule(r#"<digit> ::= "1" | "2" | "3""#);
        //a simple case
        let mut tokenized_string = characterize_string("12 3");
        //characterized string aka every character is a terminal token
        assert_eq!(
            tokenized_string,
            vec![
                Token::from_terminal("1"),
                Token::from_terminal("2"),
                Token::from_terminal(" "),
                Token::from_terminal("3")
            ]
        );
        //a simple non recursive case
        digit.symbolize_vec(&mut tokenized_string);
        assert_eq!(
            tokenized_string,
            vec![
                Token::from_non_terminal("digit", vec![Token::from_terminal("1")]),
                Token::from_non_terminal("digit", vec![Token::from_terminal("2")]),
                Token::from_terminal(" "),
                Token::from_non_terminal("digit", vec![Token::from_terminal("3")])
            ]
        )
    }

    #[test]
    fn test_recursive_symbolization() {
        let digit = non_terminal_symbol_from_rule(r#"<digit> ::= "1" | "2" | "3""#);
        let number = non_terminal_symbol_from_rule("<number> ::= <digit> | <number> <number>");
        //a simple case
        let mut tokenized_string = characterize_string("12 3");
        digit.symbolize_vec(&mut tokenized_string);
        assert_eq!(
            tokenized_string,
            vec![
                Token::from_non_terminal("digit", vec![Token::from_terminal("1")]),
                Token::from_non_terminal("digit", vec![Token::from_terminal("2")]),
                Token::from_terminal(" "),
                Token::from_non_terminal("digit", vec![Token::from_terminal("3")])
            ]
        );

        number.symbolize_vec(&mut tokenized_string);

        assert_eq!(
            tokenized_string,
            vec![
                Token::from_non_terminal(
                    "number",
                    vec![
                        Token::from_non_terminal(
                            "number",
                            vec![Token::from_non_terminal(
                                "digit",
                                vec![Token::from_terminal("1")]
                            )]
                        ),
                        Token::from_non_terminal(
                            "number",
                            vec![Token::from_non_terminal(
                                "digit",
                                vec![Token::from_terminal("2")]
                            )]
                        )
                    ]
                ),
                Token::from_terminal(" "),
                Token::from_non_terminal(
                    "number",
                    vec![Token::from_non_terminal(
                        "digit",
                        vec![Token::from_terminal("3")]
                    )]
                )
            ]
        )
    }

    #[test]
    fn test_characterization() {
        let string = "ab c";
        assert_eq!(
            characterize_string(string),
            vec![
                Token::from_terminal("a"),
                Token::from_terminal("b"),
                Token::from_terminal(" "),
                Token::from_terminal("c")
            ]
        )
    }
}
