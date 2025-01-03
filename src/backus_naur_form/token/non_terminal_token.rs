use crate::backus_naur_form::symbol::Symbol;

use super::{Token, TokenIndex};

type SubTokens = Vec<Token>;

impl FromIterator<usize> for TokenIndex {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        TokenIndex(iter.into_iter().collect::<Vec<usize>>())
    }
}

impl<'a> FromIterator<&'a usize> for TokenIndex {
    fn from_iter<T: IntoIterator<Item = &'a usize>>(iter: T) -> Self {
        TokenIndex(iter.into_iter().cloned().collect::<Vec<usize>>())
    }
}

///This represents a non terminal token, which consists of following things:  
/// - A name (for example "number" or "digit"). Unlike in the backus naur form, the angle brackets are excluded in the non_terminal_symbol property.
/// - The sub tokens that this non terminal encompasses. The sub tokens are only accesible through getter methods.
///
///[NonTerminalToken]s resemble a tree structure and are the nodes of the structure.
///For example:  
/// ```rust, ignore
///                             <expression>
///                            /      |     \
///                           /       |      \
///                          /        |       \
///                         /         |        \
///                   <expression> <operator> <expression>
///                   /    |    \      |      /     |     \
///                 "2"   "*"   "4"   "-"   "4"    "/"    "5"
/// ```
///In this case, `<expression>` is a [NonTerminalToken] that has the child [Token]s `<expression>`, `<operator>` and `<expression>`.  
///Those in turn contain [TerminalToken]s that is the actual string that got turned into syntax tree.
#[derive(PartialEq, Clone, Debug)]
pub struct NonTerminalToken {
    ///this is the non terminal it is (for example <number> or <digit>).  
    ///the angle brackets are excluded in this property.  
    pub non_terminal_symbol: String,
    sub_tokens: SubTokens,
}

impl NonTerminalToken {
    pub fn new(name: &str, sub_tokens: SubTokens) -> Self {
        Self {
            non_terminal_symbol: name.to_string(),
            sub_tokens,
        }
    }

    ///Returns the type of [NonTerminalSymbol](super::super::symbol::non_terminal_symbol::NonTerminalSymbol) this [NonTerminalToken] has.
    pub fn get_type(&self) -> Symbol {
        Symbol::NonTerminal(self.non_terminal_symbol.to_string())
    }

    ///Gets the [TokenIndex]es of the child tokens of this [NonTerminalToken] have relative to self.
    ///This returns always returns [TokenIndex]es of length 1.
    pub fn get_child_indexes(&self) -> Vec<TokenIndex> {
        let sub_tokens = self.get_child_tokens();
        let sub_tokens_indexes = sub_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| TokenIndex(vec![i]))
            .collect::<Vec<TokenIndex>>();
        sub_tokens_indexes
    }

    ///Gets a descendant of this [NonTerminalToken] by reference using a [TokenIndex].  
    ///The [TokenIndex] is assumed to be relative to self.  
    ///Returns None a [Token] at at the given [TokenIndex] does not exist.
    pub fn get_at_index(&self, token_index: &TokenIndex) -> Option<&Token> {
        let sub_tokens = self.get_child_tokens();

        match token_index.0.len() {
            //the target is one of the sub tokens
            1 => sub_tokens.get(token_index.0[0]),
            //the target is a descendant of one of the sub tokens
            len if len > 1 => match sub_tokens.get(token_index.0[0])? {
                Token::NonTerminalToken(non_terminal) => {
                    non_terminal.get_at_index(&TokenIndex(token_index.0[1..].to_vec()))
                }
                _ => None,
            },
            //if the index is empty, it cant return any value
            _ => None,
        }
    }

    ///The same as [NonTerminalToken::get] but returns a mutable reference.
    pub fn get_at_index_mut(&mut self, token_index: TokenIndex) -> Option<&mut Token> {
        let sub_tokens = self.get_child_tokens_mut();

        match token_index.0.len() {
            //the target is one of the sub tokens
            1 => sub_tokens.get_mut(token_index.0[0]),
            //the target is a descendant of one of the sub tokens
            len if len > 1 => match sub_tokens.get_mut(token_index.0[0])? {
                Token::NonTerminalToken(non_terminal) => {
                    non_terminal.get_at_index_mut(TokenIndex(token_index.0[1..].to_vec()))
                }
                _ => None,
            },
            //if the index is empty, it cant return any value
            _ => None,
        }
    }

    ///Returns a reference to the child [Token]s of self.
    pub fn get_child_tokens(&self) -> &SubTokens {
        &self.sub_tokens
    }

    ///Returns a mutable reference to the child [Token]s of self.
    pub fn get_child_tokens_mut(&mut self) -> &mut SubTokens {
        &mut self.sub_tokens
    }

    ///This function returns every descendant of the token.
    ///   
    ///This may have unintended behaviour.
    ///For example the token `<number> ::= <digit> | <number> <number>`
    ///may return several <numbers> because they are nested.  
    ///Generally, using this with a [NonTerminalToken] of a type of [super::super::symbol::non_terminal_symbol::NonTerminalSymbol]
    ///that has a choice where its recursive and atleast one choice contains only tokens of itself (like `... | <number> <number> | ...`)
    ///is not recommended.  
    ///   
    ///To get the actual terminals that the token consists of, use [NonTerminalToken::get_terminals] instead.
    pub fn get_descendant_tokens(&self) -> Vec<&Token> {
        self.get_child_tokens()
            .iter()
            .flat_map(|sub_token| match sub_token {
                Token::Terminal(_) => vec![sub_token],
                Token::NonTerminalToken(non_terminal) => {
                    let mut vec = vec![sub_token];
                    vec.append(&mut non_terminal.get_descendant_tokens());
                    vec
                }
            })
            .collect()
    }

    ///This function returns child [Token]s of self that are of a specific [Symbol].
    pub fn get_child_tokens_of_type(&self, symbol_type: &Symbol) -> Vec<&Token> {
        self.get_child_tokens()
            .iter()
            .filter(|&sub_token| sub_token == symbol_type)
            .collect()
    }

    ///This function returns descendant [Token]s of self that are of a specific [Symbol].
    pub fn get_descendant_tokens_of_type(&self, symbol_type: &Symbol) -> Vec<&Token> {
        self.get_descendant_tokens()
            .iter()
            .filter(|&&sub_token| sub_token == symbol_type)
            .cloned()
            .collect()
    }

    ///This function checks if any child of self is of type sub_token_type.
    pub fn contains_child(&self, sub_token_type: &Symbol) -> bool {
        self.get_child_tokens()
            .iter()
            .any(|sub_token| sub_token == sub_token_type)
    }

    ///This function checks if any of descendant of self is of type sub_token_type.
    pub fn contains_descendant(&self, sub_token_type: &Symbol) -> bool {
        self.get_child_tokens().iter().any(|sub_token| {
            sub_token == sub_token_type
                || match sub_token {
                    Token::NonTerminalToken(inner) => inner.contains_descendant(sub_token_type),
                    _ => false,
                }
        })
    }

    ///Returns a reference to the [Token] that is a child of self and is of type sub_token_type.  
    ///Returns None if no such [Token] exists.
    pub fn find_child(&self, sub_token_type: &Symbol) -> Option<&Token> {
        self.get_child_tokens()
            .iter()
            .find(|&sub_token| sub_token == sub_token_type)
    }

    ///Same as [NonTerminalToken::find_child] but returns a mutable reference.
    pub fn find_child_mut(&mut self, sub_token_type: &Symbol) -> Option<&mut Token> {
        self.get_child_tokens_mut()
            .iter_mut()
            .find(|sub_token| *sub_token == sub_token_type)
    }

    ///Same as [NonTerminalToken::find_child] but searches for descendants.
    pub fn find_descendant(&self, sub_token_type: &Symbol) -> Option<&Token> {
        self.get_child_tokens().iter().find(|&sub_token| {
            sub_token == sub_token_type
                || match sub_token {
                    Token::Terminal(inner) => inner == sub_token_type,
                    Token::NonTerminalToken(inner) => inner.contains_descendant(sub_token_type),
                }
        })
    }

    ///Same as [NonTerminalToken::find_child_mut] but searches for descendants.
    pub fn find_descendant_mut(&mut self, sub_token_type: &Symbol) -> Option<&mut Token> {
        self.get_child_tokens_mut().iter_mut().find(|sub_token| {
            *sub_token == sub_token_type
                || match sub_token {
                    Token::Terminal(inner) => inner == sub_token_type,
                    Token::NonTerminalToken(inner) => inner.contains_descendant(sub_token_type),
                }
        })
    }

    ///Returns the terminals that this [NonTerminalToken] consists of as a [String].  
    ///For example a `<function>` token may consist of a <function_name> and a <function_body>
    ///which in turn consist of a <word> or <instructions> respectively
    ///which in turn consist of more [NonTerminalToken]s and so on.  
    ///For example this could return for a `<function>` [NonTerminalToken] "add(x, y) = x + y" as a [String].
    pub fn get_terminals(&self) -> String {
        self.get_descendant_tokens()
            .iter()
            .filter_map(|token| match token {
                Token::Terminal(terminal) => Some(terminal.get_terminals()),
                _ => None,
            })
            .collect()
    }
}

// impl PartialEq<Symbol> for NonTerminalToken {
//     fn eq(&self, other: &Symbol) -> bool {
//         self.non_terminal_symbol.clone() == *other.get_inner()
//     }
// }

// impl PartialEq<NonTerminalToken> for Symbol {
//     fn eq(&self, other: &NonTerminalToken) -> bool {
//         *other == *self
//     }
// }

impl PartialEq<Symbol> for NonTerminalToken {
    fn eq(&self, other: &Symbol) -> bool {
        match other {
            Symbol::NonTerminal(non_terminal) => &self.non_terminal_symbol == non_terminal,
            _ => false,
        }
    }
}

impl PartialEq<NonTerminalToken> for Symbol {
    fn eq(&self, other: &NonTerminalToken) -> bool {
        other == self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backus_naur_form::token::{Token, TokenIndex};

    #[test]
    fn test_get_sub_tokens() {
        let t = Token::from_terminal("t");
        let a = Token::from_terminal("a");
        let nested = Token::from_non_terminal("t", vec![a.clone()]);

        let non_terminal = NonTerminalToken::new(
            "test",
            vec![
                t.clone(),
                a.clone(),
                a.clone(),
                nested.clone(),
                Token::from_non_terminal("a", vec![nested.clone()]),
            ],
        );

        assert_eq!(
            non_terminal.get_child_tokens(),
            &vec![
                t.clone(),
                a.clone(),
                a.clone(),
                nested.clone(),
                Token::from_non_terminal("a", vec![nested.clone()]),
            ]
        );
    }

    #[test]
    fn test_get_terminals() {
        let digit = |terminal_digit: &str| {
            Token::from_non_terminal("digit", vec![Token::from_terminal(terminal_digit)])
        };
        let operator = Token::from_non_terminal("operator", vec![Token::from_terminal("+")]);
        let expression =
            |lhs, operator, rhs| Token::from_non_terminal("expression", vec![lhs, operator, rhs]);

        let non_terminal = expression(
            digit("2"),
            operator.clone(),
            expression(digit("4"), operator.clone(), digit("3")),
        );

        match non_terminal {
            Token::NonTerminalToken(non_terminal) => {
                assert_eq!(non_terminal.get_terminals(), "2+4+3".to_string())
            }
            _ => panic!("this will never be happen"),
        }
    }

    #[test]
    fn test_get_sub_tokens_of_type() {
        let a = Token::from_terminal("a");
        let b = Token::from_terminal("b");
        let nested = Token::from_non_terminal("nested", vec![a.clone()]);
        let non_terminal = NonTerminalToken::new(
            "test",
            vec![
                a.clone(),
                b.clone(),
                nested.clone(),
                b.clone(),
                Token::from_non_terminal("c", vec![nested.clone()]),
                a.clone(),
            ],
        );
        assert_eq!(
            non_terminal.get_descendant_tokens(),
            vec![
                &a,
                &b,
                &nested,
                &a,
                &b,
                &Token::from_non_terminal("c", vec![nested.clone()]),
                &nested,
                &a,
                &a
            ]
        );
    }

    #[test]
    fn test_token_index() {
        let digit = |digit| Token::from_non_terminal("digit", vec![Token::from_terminal(digit)]);
        let operator =
            |operator| Token::from_non_terminal("operator", vec![Token::from_terminal(operator)]);
        let expression =
            |lhs, operator, rhs| Token::from_non_terminal("expression", vec![lhs, operator, rhs]);

        let product = |num1, num2| expression(digit(num1), operator("*"), digit(num2));
        let divide_by_sum_of_two_digits = |dividend, divisor1, divisor2| {
            expression(
                digit(dividend),
                operator("/"),
                expression(digit(divisor1), operator("+"), digit(divisor2)),
            )
        };

        //this will be (2*4)+(3/(4+5)) but tokenized
        let mut non_terminal = expression(
            product("2", "4"),
            operator("+"),
            divide_by_sum_of_two_digits("3", "4", "5"),
        );

        //test immutable borrows
        assert_eq!(non_terminal.get(&TokenIndex(vec![1])), Some(&operator("+")));
        assert_eq!(non_terminal.get(&TokenIndex(vec![2, 0])), Some(&digit("3")));
        assert_eq!(
            non_terminal.get(&TokenIndex(vec![2, 2, 1])),
            Some(&operator("+"))
        );
        assert_eq!(non_terminal.get(&TokenIndex(vec![2, 3, 4])), None);

        //test mut borrows
        assert_eq!(
            non_terminal.get_mut(TokenIndex(vec![1])),
            Some(&mut operator("+"))
        );
        assert_eq!(
            non_terminal.get_mut(TokenIndex(vec![2, 0])),
            Some(&mut digit("3"))
        );
        assert_eq!(
            non_terminal.get_mut(TokenIndex(vec![2, 2, 1])),
            Some(&mut operator("+"))
        );
        assert_eq!(non_terminal.get_mut(TokenIndex(vec![2, 3, 4])), None);
    }
}
