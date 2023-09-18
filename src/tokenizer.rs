use crate::error;
use crate::token::{Token, TokenType};
use crate::trie::Trie;
use std::collections::HashSet;

pub const TYPES: [&str; 2] = ["int", "char"];
const RESERVED_WORDS: [&str; 7] = ["return", "if", "else", "while", "for", "break", "sizeof"];
const RESERVED_SYMBOLS: [&str; 46] = [
    "=", "+", "-", "*", "/", "%", "<", ">", "==", "!=", "+=", "-=", "*=", "/=", "%=", "<=", ">=",
    "&", "^", "|", "&&", "||", "<<", ">>", "{", "}", "(", ")", "[", "]", ",", ";", "/*", "//",
    "\"", "!", "~", "?", ":", "<<=", ">>=", "&=", "^=", "|=", "++", "--",
];

fn close_symbol(s: &str) -> Option<&str> {
    match s {
        "\"" => Some("\""),
        "//" => Some("\n"),
        "/*" => Some("*/"),
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
    pub fn tokenize(code: &str) -> Result<Vec<Token>, error::SyntaxError> {
        let mut tokens = Vec::<Token>::new();
        let mut reserved_words = HashSet::new();
        for &word in &RESERVED_WORDS {
            reserved_words.insert(word);
        }
        let reserved_symbols = Trie::new(&RESERVED_SYMBOLS);
        let chars: Vec<(usize, char)> = code.char_indices().map(|(pos, ch)| (pos, ch)).collect();
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
                                        if &code[pos..pos + match_size] == "//" {
                                            break;
                                        }
                                        return Err(error::SyntaxError::new(i, "unexpected EOF"));
                                    }
                                    if code[chars[i].0..].starts_with(close_sym) {
                                        i += close_sym.len();
                                        break;
                                    } else {
                                        i += 1;
                                    }
                                }
                                if &code[pos..pos + match_size] == "\"" {
                                    tokens.push(Token {
                                        tt: TokenType::Str,
                                        pos,
                                        s_value: String::from(&code[pos + 1..chars[i].0 - 1]), // symbol is ascii
                                        ..Token::default()
                                    })
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
        Ok(tokens)
    }
    pub fn print_tokens(tokens: &[Token]) {
        tokens.iter().for_each(|t| t.print())
    }
}
