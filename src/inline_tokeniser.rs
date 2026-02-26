

use logos::Logos;
use std::fmt;


#[derive(Logos, Debug, PartialEq)]
pub enum Tokens {
    #[regex(r"[^\*\[\]\(\)\|\\\!]+", |lex| lex.slice().to_string())]
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

    #[token("\\")]
    BACKSLASH,

    #[token("!")]
    EXCLAMATION,

    #[token("|")]
    PIPE
}

pub fn tokenise(content: &str) -> Vec<Tokens> {
    Tokens::lexer(content)
        .filter_map(|result| result.ok())
        .collect()
}

impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tokens::TEXT(s) => write!(f, "{}", s),
            Tokens::ASTERISK => write!(f, "*"),
            Tokens::LBRACKET => write!(f, "["),
            Tokens::RBRACKET => write!(f, "]"),
            Tokens::LPAREN => write!(f, "("),
            Tokens::RPAREN => write!(f, ")"),
            Tokens::PIPE => write!(f, "|"),
            Tokens::BACKSLASH => write!(f, "\\"),
            Tokens::EXCLAMATION => write!(f, "!"),
        }
    }
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