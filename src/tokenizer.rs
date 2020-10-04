use crate::token::{Token, TokenType};
use crate::error::error_at;

pub struct Tokenizer {
    pub tokens: Vec<Token>,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
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
                    self.tokens.push(Token {
                        tt: TokenType::Reserved,
                        pos,
                        i_value: 0,
                        s_value: String::from(chars[i].1),
                    });
                    i += 1;
                }
                '<' | '>' => {
                    let pos = chars[i].0;
                    let mut temp = String::from(chars[i].1);
                    i += 1;
                    match chars[i].1 {
                        '=' => {
                            temp.push(chars[i].1);
                            self.tokens.push(Token {
                                tt: TokenType::Reserved,
                                pos,
                                i_value: 0,
                                s_value: temp,
                            });
                            i += 1;
                        }
                        _ => {
                            self.tokens.push(Token {
                                tt: TokenType::Reserved,
                                pos,
                                i_value: 0,
                                s_value: temp,
                            })
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
                    self.tokens.push(Token {
                        tt: TokenType::Reserved,
                        pos,
                        i_value: 0,
                        s_value: temp,
                    })
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
                    self.tokens.push(Token {
                        tt: TokenType::Reserved,
                        pos,
                        i_value: 0,
                        s_value: temp,
                    })
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
                    self.tokens.push(Token {
                        tt: TokenType::Num,
                        pos,
                        i_value: temp,
                        s_value: String::new(),
                    })
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
                    if &temp[..] == "return" {
                        self.tokens.push(Token {
                            tt: TokenType::Reserved,
                            pos,
                            i_value: 0,
                            s_value: temp,
                        })
                    } else {
                        self.tokens.push(Token {
                            tt: TokenType::Ident,
                            pos,
                            i_value: 0,
                            s_value: temp,
                        })
                    }
                }
                _ => {
                    error_at(code, i, "unexpected_character")
                }
            }
        }
        self.tokens.push(Token {
            tt: TokenType::Eof,
            pos: chars.len(),
            i_value: 0,
            s_value: String::new(),
        })
    }
    pub fn print_tokens(&self) {
        self.tokens.iter().for_each(|t| {
            t.print()
        })
    }
}