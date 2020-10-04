use crate::token::{Token, TokenType};
use crate::error::error_at;

const RESERVED_WORDS: [&str; 5] = ["return", "if", "else", "while", "for"];

pub struct Tokenizer {
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }
    pub fn push_reserved_token(&mut self, pos: usize, s_value: String) {
        self.tokens.push(
            Token {
                tt: TokenType::Reserved,
                pos,
                s_value,
                ..Token::default()
            }
        );
    }
    pub fn push_num_token(&mut self, pos: usize, i_value: usize) {
        self.tokens.push(
            Token {
                tt: TokenType::Num,
                pos,
                i_value,
                ..Token::default()
            }
        );
    }
    pub fn push_ident_token(&mut self, pos: usize, s_value: String) {
        self.tokens.push(
            Token {
                tt: TokenType::Ident,
                pos,
                s_value,
                ..Token::default()
            }
        );
    }
    pub fn tokenize(&mut self, code: &str) {	
        let chars: Vec<(usize, char)> = code.char_indices().map(
            |(pos, ch)| { (pos, ch) }).collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i].1 {
                ' ' | '\t' | '\n' => {
                    i += 1;
                }
                '+' | '-' | '*' | '/' | '%' | '(' | ')' | ';' => {
                    let pos = chars[i].0;
                    self.push_reserved_token(pos, String::from(chars[i].1));
                    i += 1;
                }
                '<' | '>' => {
                    let pos = chars[i].0;
                    let mut temp = String::from(chars[i].1);
                    i += 1;
                    match chars[i].1 {
                        '=' => {
                            temp.push(chars[i].1);
                            self.push_reserved_token(pos, temp);
                            i += 1;
                        }
                        _ => {
                            self.push_reserved_token(pos, temp);
                        }
                    }
                }
                '=' => {
                    let pos = chars[i].0;
                    let mut temp = String::from(chars[i].1);
                    i += 1;
                    match chars[i].1 {
                        '=' => {
                            temp.push(chars[i].1);
                            i += 1;
                        }
                        _ => {}
                    }
                    self.push_reserved_token(pos, temp);
                }
                '!' => {
                    let pos = chars[i].0;
                    let mut temp = String::from(chars[i].1);
                    i += 1;
                    match chars[i].1 {
                        '=' => {
                            temp.push(chars[i].1);
                            i += 1;
                        }
                        _ => {
                            error_at(code, i, "unexpected_character")
                        }
                    }
                    self.push_reserved_token(pos, temp);
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
                    self.push_num_token(pos, temp);
                }
                'a'..='z' => {
                    let pos = chars[i].0;
                    let mut temp = String::from(chars[i].1);
                    i += 1;
                    while i < chars.len() {
                        match chars[i].1 {
                            'a'..='z' => {
                                temp.push(chars[i].1);
                                i += 1;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    if RESERVED_WORDS.contains(&&temp[..]) {
                        self.push_reserved_token(pos, temp);
                    } else {
                        self.push_ident_token(pos, temp);
                    }
                }
                _ => {
                    error_at(code, i, "unexpected_character")
                }
            }
        }
        self.tokens.push(
            Token {
                tt: TokenType::Eof,
                pos: chars.len(),
                i_value: 0,
                s_value: String::new(),
            }
        )
    }
    pub fn print_tokens(&self) {
        self.tokens.iter().for_each(|t| {
            t.print()
        })
    }
}