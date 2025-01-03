///Contains everything relevant for the backus naur form, such as the creation of it
///and the tokenization aswell as possible compilation after the tokenization.
pub mod backus_naur_form;

///Used as a "type" (for example `<number>`).
pub use backus_naur_form::symbol::Symbol;
///Represents the nodes of the token tree that is made using a backus naur form.
pub use backus_naur_form::token::non_terminal_token::NonTerminalToken;
///Represents the leaves of the token tree that is made using a backus naur form.
pub use backus_naur_form::token::TerminalToken;
///Enum that contains either a terminal token or a non terminal token.
pub use backus_naur_form::token::Token;
