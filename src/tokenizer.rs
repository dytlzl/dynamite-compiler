use crate::token::{Token, TokenType};
use crate::error::error_at;
use std::collections::{HashSet, HashMap};

pub const RESERVED_WORDS: [&str; 7] = ["return", "if", "else", "while", "for", "break", "sizeof"];
pub const TYPES: [&str; 2] = ["int", "char"];
pub const RESERVED_SYMBOLS: [&str; 46] = [
    "=", "+", "-", "*", "/", "%", "<", ">", "==", "!=", "+=", "-=", "*=", "/=", "%=", "<=", ">=",
    "&", "^", "|", "&&", "||", "<<", ">>", "{", "}", "(", ")", "[", "]", ",", ";", "/*", "//", "\"",
    "!", "~", "?", ":", "<<=", ">>=", "&=", "^=", "|=", "++", "--",
];

pub fn close_symbol(s: &str) -> Option<&str> {
    match s {
        "\"" => Some("\""),
        "//" => Some("\n"),
        "/*" => Some("*/"),
        _ => None,
    }
}

pub struct Tokenizer {
    pub tokens: Vec<Token>,
    pub reserved_symbols: Trie,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { tokens: Vec::new(), reserved_symbols: Trie::new() }
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
        let mut reserved_words = HashSet::new();
        for &word in &RESERVED_WORDS {
            reserved_words.insert(word);
        }
        let chars: Vec<(usize, char)> = code.char_indices().map(
            |(pos, ch)| { (pos, ch) }).collect();
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
                    self.push_num_token(pos, temp);
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
                    if reserved_words.contains(&&code[pos..chars[i - 1].0+chars[i - 1].1.len_utf8()]) ||
                        TYPES.contains(&&code[pos..chars[i - 1].0+chars[i - 1].1.len_utf8()]) {
                        self.push_reserved_token(pos, String::from(&code[pos..chars[i - 1].0+chars[i - 1].1.len_utf8()]));
                    } else {
                        self.push_ident_token(pos, String::from(&code[pos..chars[i - 1].0+chars[i - 1].1.len_utf8()]));
                    }
                }
                _ => {
                    let pos = chars[i].0;
                    let match_size = self.reserved_symbols.matched_length(&code[pos..]);
                    if match_size != 0 {
                        i += match_size;
                        match close_symbol(&code[pos..pos+match_size]) {
                            Some(close_sym) => {
                                loop {
                                    if i >= chars.len() {
                                        if &code[pos..pos+match_size] == "//" {
                                            break;
                                        }
                                        error_at(code, i, "unexpected EOF")
                                    }
                                    if code[chars[i].0..].starts_with(close_sym) {
                                        i += close_sym.len();
                                        break;
                                    } else {
                                        i += 1;
                                    }
                                }
                                if &code[pos..pos+match_size] == "\"" {
                                    self.tokens.push(
                                        Token {
                                            tt: TokenType::Str,
                                            pos,
                                            s_value: String::from(&code[pos + 1..chars[i].0 - 1]), // symbol is ascii
                                            ..Token::default()
                                        }
                                    )
                                }
                            }
                            None => {
                                self.push_reserved_token(pos, String::from(&code[pos..pos+match_size]));
                            }
                        }
                    } else if i < chars.len() {
                        error_at(code, pos, "unexpected character")
                    }
                }
            }
        }
    }
    pub fn print_tokens(&self) {
        self.tokens.iter().for_each(|t| {
            t.print()
        })
    }
}

#[derive(Default)]
struct Node {
    children: HashMap<usize, Node>
}

pub struct Trie {
    double_array: Vec<(usize, usize)>,
}

const END_SYMBOL: usize = 127;

impl Trie {
    pub fn new() -> Self {
        // Make a double array
        let mut double_array = Vec::with_capacity(1000);
        double_array.extend(std::iter::repeat((0, 0)).take(END_SYMBOL*2+1));
        let mut trie = Self {
            double_array
        };
        // Make a tree of reserved symbols
        let mut root = Node::default();
        for s in RESERVED_SYMBOLS.iter() {
            let mut current_node = &mut root;
            for c in s.chars() {
                let c_num = c as usize;
                current_node = current_node.children.entry(c_num).or_default();
            }
            current_node.children.entry(END_SYMBOL).or_default();
        }
        // Insert a tree into trie
        trie.insert(1, &root);
        trie
    }
    fn insert(&mut self, index: usize, dict: &Node) {
        loop {
            let mut is_matching = true;
            let offset = self.double_array[index].1;
            if index + offset + END_SYMBOL >= self.double_array.len() {
                self.double_array.extend(std::iter::repeat((0, 0)).take(index+offset+END_SYMBOL*3-self.double_array.len()))
            }
            for (&c_index, _) in &dict.children {
                if self.double_array[index + offset + c_index].0 != 0 {
                    self.double_array[index].1 += 1;
                    is_matching = false;
                    break;
                }
            }
            if is_matching {
                break;
            }
        }
        let offset = self.double_array[index].1;
        for (&c_index, _) in &dict.children {
            self.double_array[index + offset + c_index].0 = index;
        }
        for (&c_index, d) in &dict.children {
            self.insert(index + offset + c_index, d);
        }
    }
    pub fn matched_length(&self, s: &str) -> usize {
        let mut max_len = 0;
        let mut current_index = 1;
        let mut offset = self.double_array[current_index].1;
        for (i, c) in s.char_indices() {
            let c_index = c as usize;
            if c_index >= END_SYMBOL || self.double_array[current_index + offset + c_index].0 != current_index {
                break;
            }
            current_index = current_index + offset + c_index;
            offset = self.double_array[current_index].1;
            if self.double_array[current_index + offset + END_SYMBOL].0 == current_index {
                max_len = i+1;
            }
        }
        max_len
    }
}