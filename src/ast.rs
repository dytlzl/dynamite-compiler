use crate::token::{Token, TokenType};
use crate::node::{Node, NodeType};
use crate::error::{error_at, error};
use std::collections::HashMap;

pub struct AstBuilder<'a> {
    code: &'a str,
    tokens: &'a Vec<Token>,
    cur: usize,
    pub offset_size: usize,
    offset_map: HashMap<String, usize>,
}

impl<'a> AstBuilder<'a> {
    pub fn new(code: &'a str, tokens: &'a Vec<Token>) -> Self {
        Self { code, tokens, cur: 0, offset_size: 0, offset_map: HashMap::new() }
    }
    fn consume_reserved(&mut self, s_value: &str) -> Option<Token> {
        if let TokenType::Reserved = self.tokens[self.cur].tt {
            if self.tokens[self.cur].s_value == s_value {
                self.cur += 1;
                return Some(self.tokens[self.cur - 1].clone());
            }
        }
        None
    }
    fn consume_ident(&mut self) -> Option<Token> {
        if let TokenType::Ident = self.tokens[self.cur].tt {
            self.cur += 1;
            return Some(self.tokens[self.cur - 1].clone());
        } else {
            None
        }
    }
    fn expect(&mut self, s_value: &str) -> Token {
        if let TokenType::Reserved = self.tokens[self.cur].tt {
            if self.tokens[self.cur].s_value == s_value {
                self.cur += 1;
                return self.tokens[self.cur - 1].clone();
            }
        }
        error_at(self.code, self.tokens[self.cur].pos, "unexpected token");
        unreachable!()
    }
    fn expect_number(&mut self) -> Token {
        if let TokenType::Num = self.tokens[self.cur].tt {} else {
            error_at(
                self.code, self.tokens[self.cur].pos,
                &format!("expected number, but got {}", &self.tokens[self.cur].s_value))
        }
        self.cur += 1;
        self.tokens[self.cur - 1].clone()
    }

    fn at_eof(&self) -> bool {
        if let TokenType::Eof = self.tokens[self.cur].tt {
            true
        } else {
            false
        }
    }
    pub fn stream(&mut self) -> Vec<Node> {
        let mut v = Vec::new();
        while !self.at_eof() {
            v.push(self.definition())
        }
        v
    }
    fn new_local_variable(&mut self) -> Node {
        if let Some(t) = self.consume_ident() {
            self.offset_size += 8;
            self.offset_map.insert(t.s_value.clone(), self.offset_size);
            return Node::new_with_ident(
                Some(t),
                NodeType::LVar,
                self.offset_size);
        } else {
            error_at(self.code, self.tokens[self.cur].pos, "ident expected");
            unreachable!();
        }
    }
    fn definition(&mut self) -> Node {
        self.offset_size = 0;
        self.offset_map = HashMap::new();
        self.expect("int");
        if let Some(t) = self.consume_ident() {
            if let Some(_) = self.consume_reserved("(") {
                let mut args: Vec<Node> = Vec::new();
                if let None = self.consume_reserved(")") {
                    self.expect("int");
                    args.push(self.new_local_variable());
                    while let None = self.consume_reserved(")") {
                        self.expect(",");
                        self.expect("int");
                        args.push(self.new_local_variable());
                    }
                    if args.len() >= 7 {
                        error_at(self.code, t.pos, "count of args must be less than 7")
                    }
                }
                let s_value = t.s_value.clone();
                if let Some(block) = self.consume_block() {
                    return Node {
                        token: Some(t),
                        nt: NodeType::Df,
                        func_name: String::from(s_value),
                        body: Some(Box::new(block)),
                        offset: self.offset_size,
                        args,
                        ..Node::default()
                    };
                }
            }
        }
        error_at(self.code, self.tokens[self.cur].pos, "unexpected token");
        unreachable!()
    }
    fn stmt(&mut self) -> Node {
        if let Some(t) = self.consume_reserved("if") {
            self.expect("(");
            let cond = self.expr();
            self.expect(")");
            let then = self.stmt();
            let mut els: Option<Node> = None;
            if let Some(_) = self.consume_reserved("else") {
                els = Some(self.stmt());
            }
            return Node::new_if_node(Some(t), cond, then, els);
        }
        if let Some(t) = self.consume_reserved("while") {
            self.expect("(");
            let cond = self.expr();
            self.expect(")");
            return Node::new_while_node(Some(t), cond, self.stmt());
        }
        if let Some(t) = self.consume_reserved("for") {
            self.expect("(");
            let mut ini: Option<Node> = None;
            let mut cond: Option<Node> = None;
            let mut upd: Option<Node> = None;
            if let None = self.consume_reserved(";") {
                ini = Some(self.expr());
                self.expect(";");
            }
            if let None = self.consume_reserved(";") {
                cond = Some(self.expr());
                self.expect(";");
            }
            if let None = self.consume_reserved(")") {
                upd = Some(self.expr());
                self.expect(")");
            }
            return Node::new_for_node(Some(t), ini, cond, upd, self.stmt());
        }
        if let Some(node) = self.consume_block() {
            return node;
        }
        let node = if let Some(_) = self.consume_reserved("int") {
            if let Some(t) = self.consume_ident() {
                self.offset_size += 8;
                self.offset_map.insert(t.s_value.clone(), self.offset_size);
                let mut node = Node::new_with_ident(
                    Some(t),
                    NodeType::LVar,
                    self.offset_size);
                if let Some(t) = self.consume_reserved("=") {
                    node = Node::new_with_op(
                        Some(t),
                        NodeType::Asg,
                        node,
                        self.assign());
                }
                node
            } else {
                error_at(self.code, self.tokens[self.cur].pos, "unexpected ident");
                unreachable!();
            }
        } else if let Some(t) = self.consume_reserved("break") {
            Node {
                token: Some(t),
                nt: NodeType::Brk,
                ..Node::default()
            }
        } else if let Some(t) = self.consume_reserved("return") {
            Node::new_with_op_and_lhs(Some(t), NodeType::Ret, self.expr())
        } else {
            self.expr()
        };
        self.expect(";");
        node
    }
    pub fn consume_block(&mut self) -> Option<Node> {
        if let Some(t) = self.consume_reserved("{") {
            let mut children: Vec<Node> = Vec::new();
            while let None = self.consume_reserved("}") {
                children.push(self.stmt());
            }
            Some(
                Node {
                    token: Some(t),
                    nt: NodeType::Block,
                    children,
                    ..Node::default()
                }
            )
        } else {
            None
        }
    }
    pub fn expr(&mut self) -> Node {
        self.assign()
    }
    fn assign(&mut self) -> Node {
        let mut node = self.equality();
        if let Some(t) = self.consume_reserved("=") {
            if let NodeType::LVar = node.nt {
                node = Node::new_with_op(Some(t), NodeType::Asg, node, self.assign())
                // left-associative => while, right-associative => recursive function
            } else {
                if let Some(t) = &node.token {
                    error_at(
                        self.code,
                        t.pos,
                        &format!("the lhs of assignment is expected variable, but got {:?}", node.nt))
                } else {
                    error(&format!("the lhs of assignment is expected variable, but got {:?}", node.nt))
                }
            }
        }
        node
    }
    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        loop {
            if let Some(t) = self.consume_reserved("==") {
                node = Node::new_with_op(Some(t), NodeType::Eq, node, self.relational())
            } else if let Some(t) = self.consume_reserved("!=") {
                node = Node::new_with_op(Some(t), NodeType::Ne, node, self.relational())
            } else {
                return node;
            }
        }
    }
    fn relational(&mut self) -> Node {
        let mut node = self.add();
        loop {
            if let Some(t) = self.consume_reserved("<") {
                node = Node::new_with_op(Some(t), NodeType::Lt, node, self.add())
            } else if let Some(t) = self.consume_reserved("<=") {
                node = Node::new_with_op(Some(t), NodeType::Le, node, self.add())
            } else if let Some(t) = self.consume_reserved(">") {
                node = Node::new_with_op(Some(t), NodeType::Lt, self.add(), node)
            } else if let Some(t) = self.consume_reserved(">=") {
                node = Node::new_with_op(Some(t), NodeType::Le, self.add(), node)
            } else {
                return node;
            }
        }
    }
    fn add(&mut self) -> Node {
        let mut node = self.mul();
        loop {
            if let Some(t) = self.consume_reserved("+") {
                node = Node::new_with_op(Some(t), NodeType::Add, node, self.mul())
            } else if let Some(t) = self.consume_reserved("-") {
                node = Node::new_with_op(Some(t), NodeType::Sub, node, self.mul())
            } else {
                return node;
            }
        }
    }
    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        loop {
            if let Some(t) = self.consume_reserved("*") {
                node = Node::new_with_op(Some(t), NodeType::Mul, node, self.unary())
            } else if let Some(t) = self.consume_reserved("/") {
                node = Node::new_with_op(Some(t), NodeType::Div, node, self.unary())
            } else if let Some(t) = self.consume_reserved("%") {
                node = Node::new_with_op(Some(t), NodeType::Mod, node, self.unary())
            } else {
                return node;
            }
        }
    }
    fn unary(&mut self) -> Node {
        if let Some(_) = self.consume_reserved("+") {} else if let Some(t) = self.consume_reserved("-") {
            return Node::new_with_op(Some(t), NodeType::Sub, Node::new_with_num(None, 0), self.prim());
        }
        self.prim()
    }
    fn prim(&mut self) -> Node {
        if let Some(_) = self.consume_reserved("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        } else if let Some(t) = self.consume_ident() {
            return if let Some(_) = self.consume_reserved("(") {
                let mut args: Vec<Node> = Vec::new();
                if let None = self.consume_reserved(")") {
                    args.push(self.expr());
                    while let None = self.consume_reserved(")") {
                        self.expect(",");
                        args.push(self.expr());
                    }
                    if args.len() >= 7 {
                        error_at(self.code, t.pos, "count of args must be less than 7")
                    }
                    // conforming to cdecl
                    // arguments are pushed onto the stack, from right to left
                    args.reverse();
                }
                let s_value = t.s_value.clone();
                Node {
                    token: Some(t),
                    nt: NodeType::Cf,
                    func_name: String::from(s_value),
                    args,
                    ..Node::default()
                }
            } else {
                let offset = *self.offset_map.entry(t.s_value.clone()).or_default();
                if offset == 0 {
                    error_at(self.code, t.pos, "undefined variable");
                }
                Node::new_with_ident(
                    Some(t),
                    NodeType::LVar,
                    offset)
            };
        }
        let t = self.expect_number();
        let value = t.i_value;
        Node::new_with_num(Some(t), value)
    }
}