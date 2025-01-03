pub mod non_terminal_symbol;

///A [Symbol] can be the following:  
/// - A terminal. For example `"abc"`.
/// - A non_terminal. This is the name between the angle brackets of a non terminal symbol. For example `"number"`.
///   
///This is intended to be used as a "type" to filter for specific [Token](super::token::Token)s.  
///In the case of a [Symbol::NonTerminal] the angle brackets here are excluded.  
///For example, if you filter the [Token](super::token::Token) tree for a non terminal symbols of type `<number>` you would use `Symbol::NonTerminal("number".to_string())`.  
///Another example: If you filter the [Token](super::token::Token) tree for terminals "a" you would use `Symbol::Terminal("a".to_string())`.  
#[derive(PartialEq, Debug, Clone)]
pub enum Symbol {
    Terminal(String),
    NonTerminal(String),
}
