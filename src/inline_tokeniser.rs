use crate::ast_types::Inline;

use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Tokens {
    #[regex(r"[^\*\[\]\(\):]+", |lex| lex.slice().to_string())]
    TEXT(String),

    #[token("*")]
    ASTERISK,

    #[token("[")]
    LBRACKET,

    #[token("]")]
    RBRACKET,

    #[token("(")]
    LPAREN,

    #[token(")")]
    RPAREN,

    #[token(":")]
    COLON,
}

pub fn tokenise(content: &str) -> Vec<Tokens> {
    Tokens::lexer(content)
        .filter_map(|result| result.ok())
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenise() {
        assert_eq!(tokenise(""), vec![]);
        assert_eq!(tokenise("*Hi12*"), vec![Tokens::ASTERISK, Tokens::TEXT("Hi12".to_string()), Tokens::ASTERISK]);
    }
}