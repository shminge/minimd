use crate::ast_types::Inline;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Tokens {
    TEXT(String),
    ASTERISK,
    LBRACKET,
    RBRACKET,
    LPAREN,
    RPAREN,
    COLON,
}
pub fn tokenise(content: &str) -> Vec<Tokens> {
    let mut tokens: Vec<Tokens> = Vec::new();

    for c in content.chars() {
        match c {
            '(' => tokens.push(Tokens::LPAREN),
            ')' => tokens.push(Tokens::RPAREN),
            '[' => tokens.push(Tokens::LBRACKET),
            ']' => tokens.push(Tokens::RBRACKET),
            ':' => tokens.push(Tokens::COLON),
            '*' => tokens.push(Tokens::ASTERISK),
            _ => match tokens.last_mut() {
                Some(Tokens::TEXT(s)) => s.push(c),
                _ => tokens.push(Tokens::TEXT(c.to_string())),
            }
        }
    }
    tokens
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenise() {
        assert_eq!(tokenise(""), vec![]);
        assert_eq!(tokenise("*Hi*"), vec![Tokens::ASTERISK, Tokens::TEXT("Hi".to_string()), Tokens::ASTERISK]);
    }
}