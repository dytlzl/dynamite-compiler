use crate::error;
use crate::token::{Token, TokenType};
use crate::trie::Trie;
use std::collections::HashSet;

pub const TYPES: [&str; 2] = ["int", "char"];
const RESERVED_WORDS: [&str; 7] = ["return", "if", "else", "while", "for", "break", "sizeof"];
const RESERVED_SYMBOLS: [&str; 48] = [
    "=", "+", "-", "*", "/", "%", "<", ">", "==", "!=", "+=", "-=", "*=", "/=", "%=", "<=", ">=",
    "&", "^", "|", "&&", "||", "<<", ">>", "{", "}", "(", ")", "[", "]", ",", ";", "/*", "//",
    "\"", "!", "~", "?", ":", "<<=", ">>=", "&=", "^=", "|=", "++", "--", "#", "'",
];

fn close_symbol(s: &str) -> Option<&str> {
    match s {
        "\"" => Some("\""),
        "\'" => Some("\'"),
        "//" => Some("\n"),
        "/*" => Some("*/"),
        "#" => Some("\n"),
        _ => None,
    }
}

pub struct Tokenizer {}

impl Tokenizer {
    fn reserved_token(pos: usize, s_value: String) -> Token {
        Token {
            tt: TokenType::Reserved,
            pos,
            s_value,
            ..Token::default()
        }
    }
    fn num_token(pos: usize, i_value: usize) -> Token {
        Token {
            tt: TokenType::Num,
            pos,
            i_value,
            ..Token::default()
        }
    }
    fn ident_token(pos: usize, s_value: String) -> Token {
        Token {
            tt: TokenType::Ident,
            pos,
            s_value,
            ..Token::default()
        }
    }
    pub fn tokenize(code: &str, is_debug: bool) -> Result<Vec<Token>, error::SyntaxError> {
        let mut tokens = Vec::<Token>::new();
        let reserved_words = RESERVED_WORDS.into_iter().collect::<HashSet<&str>>();
        let reserved_symbols = Trie::new(&RESERVED_SYMBOLS);
        let chars: Vec<(usize, char)> = code.char_indices().collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i].1 {
                ' ' | '\t' | '\n' => {
                    i += 1;
                }
                '0'..='9' => {
                    let pos = chars[i].0;
                    let mut temp = chars[i].1 as usize - '0' as usize;
                    i += 1;
                    while i < chars.len() {
                        match chars[i].1 {
                            '0'..='9' => {
                                temp = temp * 10 + chars[i].1 as usize - '0' as usize;
                                i += 1;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    tokens.push(Self::num_token(pos, temp));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let pos = chars[i].0;
                    i += 1;
                    while i < chars.len() {
                        match chars[i].1 {
                            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                                i += 1;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    if reserved_words
                        .contains(&&code[pos..chars[i - 1].0 + chars[i - 1].1.len_utf8()])
                        || TYPES.contains(&&code[pos..chars[i - 1].0 + chars[i - 1].1.len_utf8()])
                    {
                        tokens.push(Self::reserved_token(
                            pos,
                            String::from(&code[pos..chars[i - 1].0 + chars[i - 1].1.len_utf8()]),
                        ));
                    } else {
                        tokens.push(Self::ident_token(
                            pos,
                            String::from(&code[pos..chars[i - 1].0 + chars[i - 1].1.len_utf8()]),
                        ));
                    }
                }
                _ => {
                    let pos = chars[i].0;
                    let match_size = reserved_symbols.matched_length(&code[pos..]);
                    if match_size != 0 {
                        i += match_size;
                        match close_symbol(&code[pos..pos + match_size]) {
                            Some(close_sym) => {
                                loop {
                                    if i >= chars.len() {
                                        match &code[pos..pos + match_size] {
                                            "//" | "#" => break,
                                            _ => {
                                                return Err(error::SyntaxError::new(
                                                    i,
                                                    "unexpected EOF",
                                                ));
                                            }
                                        }
                                    }
                                    if code[chars[i].0..].starts_with(close_sym) {
                                        i += close_sym.len();
                                        break;
                                    }
                                    if code[chars[i].0..].starts_with('\\') {
                                        i += 2;
                                    } else {
                                        i += 1;
                                    }
                                }
                                if &code[pos..pos + match_size] == "\"" {
                                    tokens.push(Token {
                                        tt: TokenType::Str,
                                        pos,
                                        s_value: String::from(&code[pos + 1..i - 1]), // symbol is ascii
                                        ..Token::default()
                                    })
                                }
                                if &code[pos..pos + match_size] == "'" {
                                    if i - pos == 2 {
                                        return Err(error::SyntaxError::new(
                                            pos + 1,
                                            "unexpected character",
                                        ));
                                    }
                                    if i - pos > 3 {
                                        let escaped_character = match &code[pos + 1..pos + 3] {
                                            "\\\\" => '\\',
                                            "\\'" => '\'',
                                            "\\t" => '\t',
                                            "\\n" => '\n',
                                            _ => {
                                                return Err(error::SyntaxError::new(
                                                    pos,
                                                    "multi-character character constant",
                                                ));
                                            }
                                        };
                                        tokens.push(Token {
                                            tt: TokenType::Num,
                                            pos,
                                            i_value: escaped_character as usize,
                                            ..Token::default()
                                        })
                                    } else {
                                        tokens.push(Token {
                                            tt: TokenType::Num,
                                            pos,
                                            i_value: code.as_bytes()[pos + 1..pos + 2][0] as usize,
                                            ..Token::default()
                                        })
                                    }
                                }
                            }
                            None => {
                                tokens.push(Self::reserved_token(
                                    pos,
                                    String::from(&code[pos..pos + match_size]),
                                ));
                            }
                        }
                    } else if i < chars.len() {
                        return Err(error::SyntaxError::new(pos, "unexpected character"));
                    }
                }
            }
        }
        if is_debug {
            Self::print_tokens(&tokens);
        }
        Ok(tokens)
    }
    pub fn print_tokens(tokens: &[Token]) {
        tokens.iter().for_each(|t| t.print())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let code = "int main() { return 0; }";
        let expected = vec![
            Token {
                tt: TokenType::Reserved,
                pos: 0,
                s_value: String::from("int"),
                i_value: 0,
            },
            Token {
                tt: TokenType::Ident,
                pos: 4,
                s_value: String::from("main"),
                i_value: 0,
            },
            Token {
                tt: TokenType::Reserved,
                pos: 8,
                s_value: String::from("("),
                i_value: 0,
            },
            Token {
                tt: TokenType::Reserved,
                pos: 9,
                s_value: String::from(")"),
                i_value: 0,
            },
            Token {
                tt: TokenType::Reserved,
                pos: 11,
                s_value: String::from("{"),
                i_value: 0,
            },
            Token {
                tt: TokenType::Reserved,
                pos: 13,
                s_value: String::from("return"),
                i_value: 0,
            },
            Token {
                tt: TokenType::Num,
                pos: 20,
                s_value: String::new(),
                i_value: 0,
            },
            Token {
                tt: TokenType::Reserved,
                pos: 21,
                s_value: String::from(";"),
                i_value: 0,
            },
            Token {
                tt: TokenType::Reserved,
                pos: 23,
                s_value: String::from("}"),
                i_value: 0,
            },
        ];
        assert_eq!(Tokenizer::tokenize(code, false).unwrap(), expected);
    }

    #[test]
    fn test_tokenize_error() {
        let code = "int main() { char *s = \"hello; return 0; }";
        assert!(Tokenizer::tokenize(code, false).is_err());
    }
}
