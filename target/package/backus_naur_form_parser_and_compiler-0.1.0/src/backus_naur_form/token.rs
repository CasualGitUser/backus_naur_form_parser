//!tokens contain the actual strings.
//!a token either contains another token, or a token is a string.
//!for example the backus naur form:
//!`
//!<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
//!<number> ::= <digit> | <number> <digit>
//!now we have the string: `3098 h 104`
//!this would first be converted into: `<digit><digit><digit><digit> h <digit><digit><digit>`
//!then, it would be converted into: `<number> h <number>`
//!this module offers functions to retrieve the singular digits from number symbols and the actual digits as strings from the digit symbols
//!the comments and documentation below will take the bnf listed above for examples

pub mod non_terminal_token;

use std::fmt::{Debug, Display};

use non_terminal_token::NonTerminalToken;

use super::symbol::Symbol;

///This is used to index into a NonTerminalToken, which is a Tree.  
///For example this Tree:
///```rust, ignore
///       a  
///      /  \  
///     /    \  
///    b      c  
///   / \    / \  
///  d   e  f   g
///```
///The TokenIndex([0, 1]) would index first into b (index 0 of a's sub tokens) and then into e (index 1 of b's sub tokens)
#[derive(PartialEq, Clone, Debug)]
pub struct TokenIndex(Vec<usize>);

///[TerminalToken]s are the leaves of the AST.  
///They contain the actual strings.  
#[derive(PartialEq, Clone, Debug)]
pub struct TerminalToken(String);

impl TerminalToken {
    ///Returns the terminals it contains as a &str.  
    ///For example, this may return be "2" or "b" or "hello".
    pub fn get_terminals(&self) -> &str {
        &self.0
    }
}

impl Display for TerminalToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.0.to_string();
        write!(f, "{string}")
    }
}

///A token can either be a [TerminalToken] or a [NonTerminalToken].  
///Tokens resemble a tree structure.
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
#[derive(PartialEq, Clone)]
pub enum Token {
    //a terminal token just is a slice of the string it represents
    Terminal(TerminalToken),
    //a non terminal token can have sub_tokens and is a certain non terminal symbol
    NonTerminalToken(NonTerminalToken),
}

impl Token {
    ///Omit the (nested) double quotes on the terminal or it will mess up stuff.
    pub fn from_terminal(terminal: &str) -> Self {
        Self::Terminal(TerminalToken(terminal.to_string()))
    }

    ///Omit the angle brackets on non_terminal_symbol or it will mess up stuff.
    pub fn from_non_terminal(non_terminal_symbol: &str, sub_tokens: Vec<Token>) -> Self {
        Self::NonTerminalToken(NonTerminalToken::new(non_terminal_symbol, sub_tokens))
    }

    ///Returns a reference to a token from a [TokenIndex]. More information can be found at [TokenIndex].  
    ///Returns None if the token is a [TerminalToken].
    pub fn get(&self, token_index: &TokenIndex) -> Option<&Token> {
        match self {
            Token::NonTerminalToken(non_terminal) => non_terminal.get_at_index(token_index),
            _ => None,
        }
    }

    ///Returns a mutable reference to a token from a [TokenIndex]. More information can be found at [TokenIndex].
    ///Returns None if the token is a [TerminalToken].
    pub fn get_mut(&mut self, token_index: TokenIndex) -> Option<&mut Token> {
        match self {
            Token::NonTerminalToken(non_terminal) => non_terminal.get_at_index_mut(token_index),
            _ => None,
        }
    }

    ///If self is a [NonTerminalToken] this will return a vector of [TokenIndex]es
    ///that can be used to index into the [NonTerminalToken]s descendant [Token]s.
    ///If self is a [TerminalToken] this will return None.
    pub fn get_child_indexes(&self) -> Option<Vec<TokenIndex>> {
        match self {
            Token::NonTerminalToken(non_terminal) => Some(non_terminal.get_child_indexes()),
            _ => None,
        }
    }

    ///This gets either the [NonTerminalToken]s symbol's name or if the token is a [TerminalToken] what the [TerminalToken] contains.  
    ///For example, this may get "number" as the [NonTerminalToken]s symbols name.  
    ///Another example: this may give "2" if self is a [TerminalToken] (`symbol.is_terminal() == true` in that case).  
    pub fn get_symbol(&self) -> &str {
        match self {
            Token::Terminal(terminal) => terminal.get_terminals(),
            Token::NonTerminalToken(inner) => &inner.non_terminal_symbol,
        }
    }

    ///Returns true if self is a [TerminalToken].
    ///Returns false if self is not a [TerminalToken] (aka self is a [NonTerminalToken]).
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Terminal(_) => true,
            Self::NonTerminalToken(_) => false,
        }
    }

    ///Pass in a &[Symbol] (for example Symbol::NonTerminal("number"))
    ///and it will return true if self is that symbol.
    ///Returns false if self is not that symbol.
    pub fn is_of_type(&self, t: &Symbol) -> bool {
        t == self
    }

    ///Returns the terminals it contains as a type of [String].
    ///For example, for a [Token] <number> this may return "239".
    ///If self is a [TerminalToken], it returns what the [TerminalToken] contains.
    ///For example, a [TerminalToken] may return "a" if the [TerminalToken] contains "a".
    pub fn get_terminals(&self) -> String {
        match self {
            Token::Terminal(string) => string.get_terminals().to_string(),
            Token::NonTerminalToken(non_terminal) => non_terminal.get_terminals(),
        }
    }

    ///Turns self into a [TerminalToken].
    ///Returns None if self is not a [TerminalToken].
    pub fn to_terminal(self) -> Option<TerminalToken> {
        match self {
            Token::Terminal(terminal) => Some(terminal),
            _ => None,
        }
    }

    ///Turns self into a [NonTerminalToken].
    ///Returns None if self is not a [NonTerminalToken].
    pub fn to_non_terminal(self) -> Option<NonTerminalToken> {
        match self {
            Token::NonTerminalToken(non_terminal) => Some(non_terminal),
            _ => None,
        }
    }

    ///Turns self into a &[TerminalToken].
    ///Returns None if self is not a &[TerminalToken].
    pub fn to_terminal_token_ref(&self) -> Option<&TerminalToken> {
        match self {
            Token::Terminal(terminal) => Some(terminal),
            _ => None,
        }
    }

    ///Turns self into a &[NonTerminalToken].
    ///Returns None if self is not a &[NonTerminalToken].
    pub fn to_non_terminal_ref(&self) -> Option<&NonTerminalToken> {
        match self {
            Token::NonTerminalToken(non_terminal) => Some(non_terminal),
            _ => None,
        }
    }

    ///Turns self into a [TerminalToken].
    ///Returns the terminal argument if self is not a [TerminalToken].
    pub fn to_terminal_or(self, terminal: TerminalToken) -> TerminalToken {
        match self {
            Token::Terminal(terminal) => terminal,
            _ => terminal,
        }
    }

    ///Turns self into a [NonTerminalToken].
    ///Returns the non_terminal argument if self is not a [NonTerminalToken].
    pub fn to_non_terminal_or(self, non_terminal: NonTerminalToken) -> NonTerminalToken {
        match self {
            Token::NonTerminalToken(non_terminal) => non_terminal,
            _ => non_terminal,
        }
    }

    ///This executes or_else if the token isn't a [TerminalToken].
    pub fn to_terminal_or_else<F>(self, or_else: F) -> TerminalToken
    where
        F: FnOnce(NonTerminalToken) -> TerminalToken,
    {
        match self {
            Token::Terminal(terminal) => terminal.clone(),
            Token::NonTerminalToken(non_terminal) => or_else(non_terminal),
        }
    }

    ///This executes or_else if the token isn't a [NonTerminalToken].
    pub fn to_non_terminal_or_else<F>(self, or_else: F) -> NonTerminalToken
    where
        F: FnOnce(TerminalToken) -> NonTerminalToken,
    {
        match self {
            Token::NonTerminalToken(non_terminal) => non_terminal,
            Token::Terminal(terminal) => or_else(terminal),
        }
    }
}

impl From<&TerminalToken> for Token {
    fn from(value: &TerminalToken) -> Self {
        Token::Terminal(value.clone())
    }
}

impl From<&NonTerminalToken> for Token {
    fn from(value: &NonTerminalToken) -> Self {
        Token::NonTerminalToken(value.clone())
    }
}

impl From<TerminalToken> for Token {
    fn from(value: TerminalToken) -> Self {
        Token::Terminal(value)
    }
}

impl From<NonTerminalToken> for Token {
    fn from(value: NonTerminalToken) -> Self {
        Token::NonTerminalToken(value)
    }
}

impl PartialEq<String> for TerminalToken {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl PartialEq<TerminalToken> for String {
    fn eq(&self, other: &TerminalToken) -> bool {
        *other == *self
    }
}

impl PartialEq<&str> for TerminalToken {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<TerminalToken> for &str {
    fn eq(&self, other: &TerminalToken) -> bool {
        *other == *self
    }
}

impl PartialEq<TerminalToken> for Symbol {
    fn eq(&self, other: &TerminalToken) -> bool {
        match self {
            Symbol::Terminal(inner) => inner == other,
            _ => false,
        }
    }
}

impl PartialEq<Symbol> for TerminalToken {
    fn eq(&self, other: &Symbol) -> bool {
        other == self
    }
}

impl PartialEq<Symbol> for Token {
    fn eq(&self, other: &Symbol) -> bool {
        match other {
            Symbol::Terminal(inner) => match self {
                Token::Terminal(token_inner) => inner == token_inner,
                Token::NonTerminalToken(_) => false,
            },
            Symbol::NonTerminal(inner) => match self {
                Token::Terminal(_) => false,
                Token::NonTerminalToken(token_inner) => inner == &token_inner.non_terminal_symbol,
            },
        }
    }
}

impl PartialEq<Token> for Symbol {
    fn eq(&self, other: &Token) -> bool {
        other == self
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Terminal(inner) => write!(f, "Terminal Token({:?})", inner.get_terminals()),
            Token::NonTerminalToken(inner) => write!(
                f,
                "Token {:?}({:?})",
                inner.non_terminal_symbol,
                inner.get_child_tokens()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_terminals() {
        let terminal = |str: &str| Token::from_terminal(str);
        let non_terminal = |sym: &str, vec| Token::from_non_terminal(sym, vec);
        let a = |vec| non_terminal("a", vec);
        let b = |vec| non_terminal("b", vec);
        let c = |vec| non_terminal("c", vec);

        let token_tree = c(vec![
            b(vec![a(vec![terminal("1")]), a(vec![terminal("2")])]),
            b(vec![a(vec![terminal("4")]), a(vec![terminal("3")])]),
        ]);

        assert_eq!(token_tree.get_terminals(), "1243".to_string())
    }
}
