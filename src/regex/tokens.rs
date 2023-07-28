use std::str::Chars;

#[derive(Debug)]
enum Token {
    Char(char),
    Star,
    Pipe,
    OpenParens,
    CloseParens,
    Backslash,
}

struct RegexTokenizer<'a> {
    char_iter: Chars<'a>,
}

impl<'a> RegexTokenizer<'a> {
    pub fn from_string(string: &'a str) -> RegexTokenizer<'a> {
        return RegexTokenizer {
            char_iter: string.chars(),
        };
    }
}

impl<'a> Iterator for RegexTokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if let Some(c) = self.char_iter.next() {
            match c {
                '*' => Some(Token::Star),
                '|' => Some(Token::Pipe),
                '(' => Some(Token::OpenParens),
                ')' => Some(Token::CloseParens),
                '\\' => Some(Token::Backslash),
                _ => Some(Token::Char(c)),
            }
        } else {
            None
        }
    }
}
