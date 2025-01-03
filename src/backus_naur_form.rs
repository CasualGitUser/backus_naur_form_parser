//! This module contains the necessary functions to symbolize strings (turn a string into a AST that consists of non terminal symbols).
//! and the necessary functions to compile those symbolized strings (turn a AST into another string).
//!
//! # Creating backus naur forms
//! A backus naur form is used to for the parser. It will turn a string into non terminal symbols.  
//! The backus_naur_form! macro is used to create backus naur forms:rust-analyzer-diagnostics-view:/diagnostic message [0]?0#file:///e%3A/Erik/Hobby%20Projekte/Rust/backus_naur_tokenizer/src/backus_naur_form.rs
//! ```rust, ignore
//! backus_naur_form!(
//!     priority 0 => r#"<digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9""#
//!     priority 0 => r#"<two-digits> ::= <digit> <digit>"#
//! )
//! ```
//! The rules are raw string literals. (aka r#"{here comes the string}"#) so it can contain other strings.  
//! Every rule has a priority. rules with higher priorites get evaluated first.
//! For example, this is nice for evaluating multiplications and divisions first before
//! evaluating additions and subtractions.
//!
//! ## Creating recursive rules
//! Some extra rules apply to recursive non terminal symbols that are supposed to be "arrays" of other non terminals.  
//! To clarify, i will use an example backus naur form:
//! ```rust, ignore
//! backus_naur_form!(
//!     priority 0 => r#"<digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9""#
//!     priority 0 => r#"<number> ::= <digit>
//!                      | <number> <digit>
//!                      | <digit> <number>
//!                      | <number> <number>"#
//! )
//! ```
//! I listed several options to create a recursive <number> non terminal.
//! In cases where a recursive symbol is basically a "array" of something (in this example a "array" of digits)
//! <ou need to use the `<token> ::= <token> <token>` rule.  
//! Following recursive cases don't work (<...> is used to denote some token. For example in the above backus naur form it would be <digit>):
//! - `<token> ::= <token> <...>`
//! - `<token> ::= <...> <token>`
//!
//! The reason for this is simple: The algorithm turns every <digit> into a <number> and therefore theres no `<number> <digit>` or `<digit> <number>`.

pub mod rule;
pub mod symbol;
pub mod token;
use std::{collections::HashMap, fmt::Debug, ops::Range};
use token::{non_terminal_token::NonTerminalToken, Token};

use symbol::{non_terminal_symbol::NonTerminalSymbol, Symbol};

///Rules are built like this: `<symbol> ::= expression`.  
///The body of a rule. It contains the different [Choice]s/ways to turn [Token] or [Token]s into a higher [NonTerminalToken].
pub type Expression = Vec<Choice>;
///A Choice contains a way to turn [Token] or [Token]s into a higher [NonTerminalToken].
pub type Choice = Vec<Symbol>;
///A function that compiles a [NonTerminalToken] by turning it into a [String].  
///Takes following arguments:
/// - The [NonTerminalToken] that should be compiled.
/// - The [BackusNaurForm] that contains the rules and other compile functions.
pub type CompileFunction<'a> = &'a dyn Fn(&NonTerminalToken, &BackusNaurForm) -> String;

#[derive(Default)]
pub struct BackusNaurForm<'a> {
    //contains the non terminal symbols which in turn contain the rules/expressions
    //the second value is the priority
    rules: Vec<(NonTerminalSymbol, usize)>,
    //The String is just a non terminal symbol name and the fn takes a token of that non terminal symbol and produces a string.
    //Essentially, this is for the translation from the tokenized vec to a new language.
    compile_functions: HashMap<String, CompileFunction<'a>>,
}

impl<'a> BackusNaurForm<'a> {
    ///Used to add a new [NonTerminalSymbol] to the backus naur form.
    fn add_non_terminal_symbol(&mut self, non_terminal_symbol: NonTerminalSymbol, priority: usize) {
        self.rules.push((non_terminal_symbol, priority));
    }

    pub fn add_non_terminal_symbol_from_rule(&mut self, rule: &str, priority: usize) {
        self.add_non_terminal_symbol(NonTerminalSymbol::from_rule(rule), priority);
    }

    ///Returns true if the [BackusNaurForm] contains a [NonTerminalSymbol]  with the specified name.  
    ///This function assumes that the angle brackets are not included in the name.
    pub fn contains_symbol(&self, name: &str) -> bool {
        self.rules.iter().any(|(non_terminal_symbol, _)| {
            non_terminal_symbol == &Symbol::NonTerminal(name.to_string())
        })
    }

    ///This parses a string into a vector of [Token].  
    ///The vector of [Token]s is essentially the AST.  
    ///   
    /// ## Example
    ///   
    /// Lets say we have this [BackusNaurForm]:
    /// ```rust, ignore
    /// priority 0 => <expression> ::= <digit> <operator> <digit>
    /// | <digit> <operator> <expression>
    /// | <expression> <operator> <digit>
    /// | <expression> <operator> <expression
    /// priority 0 => <digit> ::= "1" | "2" | "3" | "4" | "5"
    /// priority 0 => <operator> ::= "+" | "-" | "*" | "/"
    /// ```
    /// This has a valid root [Token]. For example the [String] "2*4-4/5" will be symbolized into:
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
    /// Notice the tree structure. This is the AST.
    pub fn symbolize_string(&self, string: &str) -> Vec<Token> {
        let mut tokenized_string = characterize_string(string);
        let mut modified_this_iteration;

        let mut sorted_rules = self.rules.clone();
        sorted_rules.sort_by_key(|(_, priority)| *priority);
        sorted_rules.reverse();

        loop {
            modified_this_iteration = false;
            sorted_rules.iter().for_each(|(non_terminal_symbol, _)| {
                if non_terminal_symbol.further_symbolization_possible(&tokenized_string) {
                    modified_this_iteration = true;
                }

                non_terminal_symbol.symbolize_vec(&mut tokenized_string);
            });

            if !modified_this_iteration {
                break;
            }
        }

        tokenized_string
    }

    ///This compiles a [String] using the backus naur form and the given Compilefunctions.  
    ///Only [Token]s at the uppermost level will be compiled.  
    ///
    /// Rules with higher priority will be applied first.  
    /// Choices that are specified before other choices will be applied first.  
    /// For example, in the bellow example "a" would be applied before "b" in the `<letter>` non terminal symbol.
    /// ## Example
    ///Lets take this backus naur form as first example:
    /// ```rust, ignore
    /// priority 0 => <number> ::= <digit> | <number> <number>
    /// priority 0 => <word> ::= <letter> | <word> <word>
    /// priority 0 => <digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0"
    /// priority 0 => <letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l"
    /// | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
    /// ```  
    ///Anything that consists of only digits or letters will be turned into a tree where the uppermost tokens are <word> and <number> non terminals.  
    ///The tree will look kind of like this where ... denotes something more (could be non terminals, terminals etc.)
    /// ```rust, ignore
    ///    <number>      <word>      //and so on, in any order, the only important thing is that a variable amount of <numbers> and <words>
    ///   /   |    \     /  |  \     //in any order are the uppermost tokens.
    ///  ... ...   ... ... ... ...
    /// ```
    /// This function would only compile the uppermost tokens - in this case **only** `<number>` and <word> tokens at the uppermost level
    /// aka those that are direct members of the vector returned from symbolize_string(string).
    ///
    ///
    /// If any of the tokens dont have CompileFunctions they will simply be mapped to the terminals they encompass.  
    /// In other words, either tokens get compiled or they won't be touched/modified at all.
    pub fn compile_string(&self, string: &str) -> String {
        let symbolized_string = self.symbolize_string(string);
        symbolized_string
            .into_iter()
            .map(|token| match token {
                Token::NonTerminalToken(non_terminal) => self
                    .compile_token(&non_terminal)
                    .unwrap_or(non_terminal.get_terminals()),
                Token::Terminal(terminal) => terminal.to_string(),
            })
            .collect()
    }

    ///Compiles a [NonTerminalToken] into a String.  
    ///Returns none if there is no function that compiles this [NonTerminalToken].
    pub fn compile_token(&self, non_terminal: &NonTerminalToken) -> Option<String> {
        let name = &non_terminal.non_terminal_symbol;
        self.compile_functions
            .get(name)
            .map(|f| f(non_terminal, self))
    }

    ///Used to add functions that compiles a [NonTerminalToken] into a [String].  
    pub fn add_compile_function(&mut self, non_terminal_symbol: &str, f: CompileFunction<'a>) {
        self.compile_functions
            .insert(non_terminal_symbol.to_string(), f);
    }

    ///This function tests wether the given [String] can be turned into exactly one [Token] - a root token.  
    ///This method returns false in the following case:  
    /// - There is no root [Token].   
    ///  
    ///To create a root [Token], the following must be true:
    /// - the [String] must be symbolized into exactly 1 [NonTerminalSymbol] (all info is stored in the root [Token]s descendants)
    ///
    /// # Examples
    ///
    /// ## A valid [BackusNaurForm]
    ///
    /// Lets say we have this [BackusNaurForm]:
    /// ```rust, ignore
    /// priority 0 => <expression> ::= <digit> <operator> <digit>
    /// | <digit> <operator> <expression>
    /// | <expression> <operator> <digit>
    /// | <expression> <operator> <expression
    /// priority 0 => <digit> ::= "1" | "2" | "3" | "4" | "5"
    /// priority 0 => <operator> ::= "+" | "-" | "*" | "/"
    /// ```
    /// This has a valid root [Token]. For example the [String] "2*4-4/5" will be symbolized into:
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
    /// The root [Token] in this example is the uppermost expression. Its exactly one [Token], so its a root [Token].
    ///
    /// ## A invalid backus naur form
    ///
    /// Lets say we have this [BackusNaurForm]:
    /// ```rust, ignore
    /// priority 0 => <word> ::= <letter> | <word> <word>
    /// priority 0 => <letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l"
    /// | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
    /// priority 0 => <whitespace> ::= " " | <whitespace> <whitespace>
    ///
    /// ```
    /// This is invalid: we know that the vector returned from [symbolize_string](BackusNaurForm::symbolize_string) will have a len != 1.  
    /// Therefore there is no root [Token].  
    /// ## Modified to return true
    /// We can modify it to create a root [Token]:
    /// ```rust, ignore
    /// priority 0 => <syntax> ::= <word> <whitespace> <word>
    /// //there might be whitespace before the first word is written
    ///               | <whitespace> <word>
    /// //there might be whitespace after the last word
    ///               | <word> <whitespace>
    /// //recursive case: this makes the <syntax> to a root token
    ///               | <syntax> <syntax>
    /// priority 0 => <word> ::= <letter> | <word> <word>
    /// priority 0 => <letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l"
    ///               | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
    /// priority 0 => <whitespace> ::= " " | <whitespace> <whitespace
    /// ```
    /// This would be valid since the vector returned form [symbolize_string](BackusNaurForm::symbolize_string) will have a len == 1
    /// since everything would be encompassed by one `<syntax>` [NonTerminalSymbol].
    pub fn compiles_to_root_token(&self, string: &str) -> bool {
        self.symbolize_string(string).len() == 1
    }
}

///Used to create [BackusNaurForm]s declaratively.  
///Following things need to be specified:
///- A priority. Rules with higher priority will be applied first.
///- A rule. A rule is simply a raw string literal (for example `<abc> ::= "a" | "b" | "c"`). It must be a valid [BackusNaurForm] rule.
///- A optional closure that takes in the specified [NonTerminalToken] by reference and outputs a [String].
///
/// ## Syntax
/// A new priority, rule and a optional function to compile that rule is specified like this:  
/// `priority [priority_number: usize] => <rule_name> ::= [tokens] => |[token_name: &NonTerminalToken] {[closure body]}`.  
/// The last arrow (the closure) is optional. So this is valid too:  
/// `priority [priority_number: usize] => <rule_name> ::= [tokens]`  
///
/// ## Example
///   
/// The following example shows a backus naur form that creates a AST from mathematical expressions.  
/// It uses the the priorities to turn multiplications and divsions into expressions before addition and subtractions are turned into expressions.  
/// The uppermost token (a `<expression>`) is compiled into the string "This is a expression".
/// ```rust, ignore
/// priority 0 => <expression> ::= <number> <operator> <number>
///               | <expression> <operator> <number>
///               | <number> <operator> <expression>
///               | <expression> <operator> <expression>
///               | <mul-or-div-expression> => |expression_token: &NonTerminalToken| {
///                 "This is a expression".to_string()
///               }
/// //this rule is applied before all others
/// priority 1 => <mul-or-div-expression> ::= <number> <mul-or-div-operator> <number>
///               | <expression> <mul-or-div-operator> <number>
///               | <number> <mul-or-div-operator> <expresion>
///               | <expression> <mul-or-div-operator> <expression>
/// priority 0 => <operator> ::= <mul-or-div-operator> | "+" | "-"
/// priority 0 => <mul-or-div-operator> ::= "*" | "/"
/// priority 0 => <number> ::= <digit> | <number> <number>
/// priority 0 => <digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0"
/// ```
#[macro_export]
macro_rules! backus_naur_form {
    ($(priority $priority:expr => $rule:expr $(=> $function_body:expr)?)+) => {{
        let mut bnf = $crate::backus_naur_form::BackusNaurForm::default();
        $(
            let _non_terminal_name = $crate::backus_naur_form::rule::get_name_from_rule($rule);

            $(
                bnf.add_compile_function(_non_terminal_name, &$function_body);
            )?

            bnf.add_non_terminal_symbol_from_rule($rule, $priority);
        )+
        bnf
    }};
}

impl Debug for BackusNaurForm<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rules = self
            .rules
            .iter()
            .fold(String::new(), |rule_set, (non_terminal_symbol, _)| {
                let stringified_expression = stringify_expression(non_terminal_symbol.get_rule());
                let name = non_terminal_symbol.get_name();
                format!("{rule_set} \n <{name}> ::= {stringified_expression}")
            });
        write!(f, "{rules}")
    }
}

//Used for the tests.
impl PartialEq for BackusNaurForm<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.rules == other.rules
    }
}

//The slice needs to be sliced from the vec, otherwise there is undefined behaviour.
//Returns the range that the slice is occupying. aka &vec[range.start..range..end] == slice.
fn range_from_slice<A>(vec: &[A], slice: &[A]) -> Range<usize> {
    let start = unsafe { slice.as_ptr().offset_from(vec.as_ptr()) as usize };
    let end = start + slice.len();
    start..end
}

//important: the ranges cant overlap, otherwise its undefined behaviour.
//This replaces the specified ranges using the specified function replace_with.
fn replace_ranges<A, B>(vec: &mut Vec<A>, ranges: &mut [Range<usize>], mut replace_with: B)
where
    B: FnMut(Vec<A>) -> A,
{
    //this is important for the reversing.
    ranges.sort_by_key(|range| range.start);
    ranges.reverse();
    ranges.iter().for_each(|range| {
        replace_range(vec, range, &mut replace_with);
    });
}

//helper function for replace_ranges.
//Simply replaces one range in a vector.
fn replace_range<A, B>(vec: &mut Vec<A>, range: &Range<usize>, replace_with: &mut B)
where
    B: FnMut(Vec<A>) -> A,
{
    let mut removed_elements = vec![];
    for i in (range.start..range.end).rev() {
        removed_elements.push(vec.remove(i));
    }
    removed_elements.reverse();
    vec.insert(range.start, replace_with(removed_elements));
}

//used for the Debug implementation of BackusNaurForm.
fn stringify_expression(expression: &Expression) -> String {
    expression
        .iter()
        .enumerate()
        .fold(String::new(), |expr, (index, choice)| {
            let stringified_choice = stringify_choice(choice, index);
            expr + &stringified_choice
        })
}

//used for the Debug implementation of BackusNaurForm.
//Helper function for stringify_expression.
fn stringify_choice(choice: &Choice, index: usize) -> String {
    choice.iter().fold(
        if index != 0 { "| " } else { "" }.to_string(),
        |ch, symbol| match symbol {
            Symbol::Terminal(inner) => format!("{ch}\"{inner}\" "),
            Symbol::NonTerminal(inner) => format!("{ch}<{inner}> "),
        },
    )
}

//Returns a vector of TerminalTokens where every TerminalToken contains exactly on character of the original string.
//Its only a character each because the algorithm to turn summarize a range of tokens into a higher token needs that.
fn characterize_string(string: &str) -> Vec<Token> {
    string
        .chars()
        .map(|char| Token::from_terminal(&char.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::single_range_in_vec_init)]

    use rule::non_terminal_symbol_from_rule;

    use super::*;

    #[test]
    fn test_backus_naur_form() {
        let bnf = backus_naur_form!(
            priority 0 => r#"<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9""#
            priority 0 => r#"<number> ::= <digit> | <number> <digit>"#
        );
        let mut rhs = BackusNaurForm::default();
        let non_terminal_symbol1 = non_terminal_symbol_from_rule(
            r#"<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9""#,
        );
        let non_terminal_symbol2 =
            non_terminal_symbol_from_rule(r#"<number> ::= <digit> | <number> <digit>"#);
        rhs.add_non_terminal_symbol(non_terminal_symbol1, 0);
        rhs.add_non_terminal_symbol(non_terminal_symbol2, 0);
        assert_eq!(bnf, rhs);
    }

    #[test]
    fn test_range_from_slice() {
        let vec = [1, 2, 3, 4, 5];
        let slice1 = &vec[1..3]; //1, 2, 3
        let slice2 = &vec[4..5]; //5
        assert_eq!(range_from_slice(&vec, slice1), 1..3);
        assert_eq!(range_from_slice(&vec, slice2), 4..5);
    }

    #[test]
    fn test_replace_ranges() {
        let vec = vec![1, 2, 3, 8, 1, 2, 3, 5];
        let mut pattern = vec![3..4];
        let mut pattern1 = vec![0..3, 4..7];
        let mut pattern2 = vec![];
        fn replace_with<T>(_: Vec<T>) -> i32 {
            99
        }

        let mut vec_copy = vec.clone();
        replace_ranges(&mut vec_copy, &mut pattern, replace_with);
        assert_eq!(vec_copy, vec![1, 2, 3, 99, 1, 2, 3, 5]);

        let mut vec_copy = vec.clone();
        replace_ranges(&mut vec_copy, &mut pattern1, replace_with);
        assert_eq!(vec_copy, vec![99, 8, 99, 5]);

        let mut vec_copy = vec.clone();
        replace_ranges(&mut vec_copy, &mut pattern2, replace_with);
        assert_eq!(vec_copy, vec![1, 2, 3, 8, 1, 2, 3, 5]);
    }

    #[test]
    fn test_priority() {
        let bnf = backus_naur_form!(
            priority 0 => r#"<digit> ::= "1" | "2""#
            priority 0 => r#"<sum> ::= <digit> "+" <digit>"#
            priority 1 => r#"<product> ::= <digit> "*" <digit>"#
        );

        let string = "1*2";
        assert_eq!(
            bnf.symbolize_string(string),
            vec![Token::from_non_terminal(
                "product",
                vec![
                    Token::from_non_terminal("digit", vec![Token::from_terminal("1")]),
                    Token::from_terminal("*"),
                    Token::from_non_terminal("digit", vec![Token::from_terminal("2")])
                ]
            )]
        )
    }

    #[test]
    fn test_symbolization() {
        let expression = |vec| Token::from_non_terminal("expression", vec);
        let product = |vec| Token::from_non_terminal("product", vec);
        let sum = |vec| Token::from_non_terminal("sum", vec);
        let number = |vec| Token::from_non_terminal("number", vec);
        let digit = |vec| Token::from_non_terminal("digit", vec);
        let terminal = |str: &str| Token::from_terminal(str);
        //this tests a bunch of recursive stuff
        //really just a simple math language
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
        //test the product creation
        let string = "2*4";
        assert_eq!(
            bnf.symbolize_string(string),
            vec![expression(vec![product(vec![
                number(vec![digit(vec![terminal("2")])]),
                terminal("*"),
                number(vec![digit(vec![terminal("4")])])
            ])])]
        );
        // uses only numbers with 2 digits since up to 3 digits it may be nested really deep (doesnt affect the copmilation though)
        let string = "12+2*45";
        let two_times_fourtyfourty = expression(vec![product(vec![
            number(vec![digit(vec![terminal("2")])]),
            terminal("*"),
            number(vec![
                number(vec![digit(vec![terminal("4")])]),
                number(vec![digit(vec![terminal("5")])]),
            ]),
        ])]);
        assert_eq!(
            bnf.symbolize_string(string),
            vec![Token::from_non_terminal(
                "expression",
                vec![sum(vec![
                    number(vec![
                        number(vec![digit(vec![terminal("1")])]),
                        number(vec![digit(vec![terminal("2")])])
                    ]),
                    terminal("+"),
                    two_times_fourtyfourty
                ])]
            )]
        )
    }

    #[test]
    fn test_compile_string() {
        let mut bnf = backus_naur_form!(
            priority 0 => r#"<digit> ::= "1" | "2" | "3""#
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

                    parsed_and_doubled
                        .first()
                        .expect("failed to parse first digit")
                        .to_owned()
                        + "<here comes the operator>"
                        + parsed_and_doubled
                            .last()
                            .expect("failed to parse last digit")
                }
        );

        //test wether its symboliezd into exactly one symbol
        assert_eq!(
            bnf.symbolize_string("2+3"),
            vec![Token::from_non_terminal(
                "expression",
                vec![
                    Token::from_non_terminal("digit", vec![Token::from_terminal("2")]),
                    Token::from_non_terminal("operator", vec![Token::from_terminal("+")]),
                    Token::from_non_terminal("digit", vec![Token::from_terminal("3")])
                ]
            )]
        );

        bnf.add_compile_function("digit", &|digit_token, _bnf| {
            (digit_token
                .get_terminals()
                .parse::<usize>()
                .expect("failed to parse <digit> to usize")
                * 2)
            .to_string()
        });

        bnf.add_compile_function("expression", &|token, bnf| {
            let digits = token.get_child_tokens_of_type(&Symbol::NonTerminal("digit".to_string()));
            let _operator =
                token.get_child_tokens_of_type(&Symbol::NonTerminal("operator".to_string()));
            let digits = digits
                .into_iter()
                .map(|digit| {
                    bnf.compile_token(
                        digit
                            .to_non_terminal_ref()
                            .expect("<digit> should be a non terminal token"),
                    )
                    .expect("no compile function available for <digit>")
                })
                .collect::<Vec<String>>();
            digits.first().unwrap().to_string()
                + "<here comes the operator>"
                + digits.last().unwrap()
        });

        //check wether the operation gets turned into a expression
        assert!(bnf.compiles_to_root_token("2+3"));

        //test a simple equation
        assert_eq!(
            bnf.compile_string("2+3"),
            "4<here comes the operator>6".to_string()
        );
    }
}
