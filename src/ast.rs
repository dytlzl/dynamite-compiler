use crate::token::{Token, TokenType};
use crate::node::{Node, NodeType, Type};
use crate::error::{error_at};
use std::collections::HashMap;

pub struct AstBuilder<'a> {
    code: &'a str,
    tokens: &'a Vec<Token>,
    cur: usize,
    pub offset_size: usize,
    offset_map: HashMap<String, (Type, usize)>,
    function_types: HashMap<String, (Vec<Type>, Type, bool)>,
    global_variables: HashMap<String, Type>,
    pub string_literals: Vec<String>,
}

impl<'a> AstBuilder<'a> {
    pub fn new(code: &'a str, tokens: &'a Vec<Token>) -> Self {
        let mut builder = Self {
            code,
            tokens,
            cur: 0,
            offset_size: 0,
            offset_map: HashMap::new(),
            function_types: HashMap::new(),
            global_variables: HashMap::new(),
            string_literals: Vec::new(),
        };
        builder.function_types.insert(
            String::from("printf"),
            (vec![Type::Ptr(Box::new(Type::Char))], Type::Int, false)
        );
        builder.function_types.insert(
            String::from("exit"),
            (vec![Type::Int], Type::Int, false)
        );
        builder
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
        error_at(self.code, self.tokens[self.cur].pos, &format!("`{}` expected", s_value));
        unreachable!()
    }
    fn expect_number(&mut self) -> Token {
        if let TokenType::Num = self.tokens[self.cur].tt {} else {
            error_at(
                self.code, self.tokens[self.cur].pos,
                &format!("number expected, but got {}", &self.tokens[self.cur].s_value))
        }
        self.cur += 1;
        self.tokens[self.cur - 1].clone()
    }
    fn consume(&mut self, tt: TokenType) -> Option<Token> {
        if tt == self.tokens[self.cur].tt {
            self.cur += 1;
            Some(self.tokens[self.cur - 1].clone())
        } else {
            None
        }
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
        let mut ty = self.expect_type();
        while let Some(_) = self.consume_reserved("*") {
            ty = Type::Ptr(Box::new(ty));
        }
        if let Some(t) = self.consume_ident() {
            self.offset_size += 8;
            self.offset_map.insert(t.s_value.clone(), (ty.clone(), self.offset_size));
            return Node {
                token: Some(t),
                nt: NodeType::LVar,
                cty: Some(ty),
                offset: Some(self.offset_size),
                ..Node::default()
            };
        } else {
            error_at(self.code, self.tokens[self.cur].pos, "ident expected");
            unreachable!();
        }
    }
    fn consume_type(&mut self) -> Option<Type> {
        if let Some(_) = self.consume_reserved("int") {
            Some(Type::Int)
        } else if let Some(_) = self.consume_reserved("char") {
            Some(Type::Char)
        } else {
            None
        }
    }
    fn expect_type(&mut self) -> Type {
        if let Some(ty) = self.consume_type() {
            ty
        } else {
            error_at(self.code, self.tokens[self.cur].pos, "type expected");
            unreachable!()
        }
    }
    fn definition(&mut self) -> Node {
        self.offset_size = 0;
        self.offset_map = HashMap::new();
        let mut ty = self.expect_type();
        while let Some(_) = self.consume_reserved("*") {
            ty = Type::Ptr(Box::new(ty));
        }
        if let Some(t) = self.consume_ident() {
            if let Some(_) = self.consume_reserved("(") {
                let mut args: Vec<Node> = Vec::new();
                if let None = self.consume_reserved(")") {
                    args.push(self.new_local_variable());
                    while let None = self.consume_reserved(")") {
                        self.expect(",");
                        args.push(self.new_local_variable());
                    }
                    if args.len() >= 7 {
                        error_at(self.code, t.pos, "count of args must be less than 7")
                    }
                }
                let arg_types: Vec<Type> = args.iter().map(
                    |arg| { arg.resolve_type().clone().unwrap() }
                ).collect();
                self.function_types.insert(t.s_value.clone(), (arg_types, ty, true));
                let s_value = t.s_value.clone();
                if let Some(block) = self.consume_block() {
                    return Node {
                        token: Some(t),
                        nt: NodeType::Df,
                        global_name: String::from(s_value),
                        body: Some(Box::new(block)),
                        offset: Some(self.offset_size),
                        args,
                        ..Node::default()
                    };
                }
            } else {
                if let Some(_) = self.consume_reserved("[") {
                    let n = self.expect_number();
                    ty = Type::Arr(Box::new(ty), n.i_value);
                    self.expect("]");
                }
                self.expect(";");
                self.global_variables.insert(t.s_value.clone(), ty.clone());
                return Node {
                    token: Some(t.clone()),
                    nt: NodeType::GVar,
                    cty: Some(ty),
                    global_name: t.s_value,
                    ..Node::default()
                };
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
        let node = if let Some(mut ty) = self.consume_type() {
            while let Some(_) = self.consume_reserved("*") {
                ty = Type::Ptr(Box::new(ty));
            }
            if let Some(t) = self.consume_ident() {
                if let Some(_) = self.consume_reserved("[") {
                    let n = self.expect_number();
                    let size = n.i_value * ty.size_of();
                    ty = Type::Arr(Box::new(ty), n.i_value);
                    self.offset_size += size;
                    self.offset_map.insert(t.s_value.clone(), (ty.clone(), self.offset_size));
                    self.expect("]");
                    Node {
                        token: Some(t),
                        nt: NodeType::LVar,
                        cty: Some(ty),
                        offset: Some(self.offset_size),
                        ..Node::default()
                    }
                } else {
                    self.offset_size += 8;
                    self.offset_map.insert(t.s_value.clone(), (ty.clone(), self.offset_size));
                    let mut node = Node {
                        token: Some(t),
                        nt: NodeType::LVar,
                        cty: Some(ty),
                        offset: Some(self.offset_size),
                        ..Node::default()
                    };
                    if let Some(t) = self.consume_reserved("=") {
                        node = Node::new_with_op(
                            Some(t),
                            NodeType::Asg,
                            node,
                            self.assign());
                    }
                    node
                }
            } else {
                error_at(self.code, self.tokens[self.cur].pos, "ident expected");
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
            // left-associative => while, right-associative => recursive function
            node = Node::new_with_op(Some(t), NodeType::Asg, node, self.assign())
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
        if let Some(t) = self.consume_reserved("sizeof") {
            return Node {
                token: Some(t),
                value: Some(self.unary().resolve_type().unwrap().size_of()),
                cty: Some(Type::Int),
                ..Node::default()
            };
        }
        if let Some(_) = self.consume_reserved("+") {} else if let Some(t) = self.consume_reserved("-") {
            return Node::new_with_op(Some(t), NodeType::Sub, Node::new_with_num(None, 0), self.prim());
        }
        if let Some(t) = self.consume_reserved("&") {
            return Node {
                token: Some(t),
                nt: NodeType::Addr,
                lhs: Some(Box::new(self.unary())),
                ..Node::default()
            };
        }
        if let Some(t) = self.consume_reserved("*") {
            return Node {
                token: Some(t),
                nt: NodeType::Deref,
                lhs: Some(Box::new(self.unary())),
                ..Node::default()
            };
        }
        self.prim()
    }
    fn prim(&mut self) -> Node {
        let mut node = if let Some(_) = self.consume_reserved("(") {
            let node = self.expr();
            self.expect(")");
            node
        } else if let Some(t) = self.consume_ident() {
            if let Some(_) = self.consume_reserved("(") {
                if !self.function_types.contains_key(&t.s_value) {
                    error_at(self.code, t.pos, "undefined function");
                }
                let (arg_types, ret_ty, is_enabled_arg_types_validation) = self.function_types.get(&t.s_value).unwrap().clone();
                let mut args: Vec<Node> = Vec::new();
                if let None = self.consume_reserved(")") {
                    args.push(self.expr());
                    while let None = self.consume_reserved(")") {
                        self.expect(",");
                        args.push(self.expr());
                    }
                    if is_enabled_arg_types_validation {
                        if args.len() != arg_types.len() {
                            error_at(self.code, t.pos, "invalid count of arguments");
                        }
                        /*
                        for (n, arg_type) in args.iter().zip(arg_types) {
                            if n.resolve_type() != Some(arg_type.clone()) {
                                error_at(self.code, n.token.as_ref().unwrap().pos, "invalid type")
                            }
                        }
                         */
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
                    global_name: String::from(s_value),
                    cty: Some(ret_ty.clone()),
                    args,
                    ..Node::default()
                }
            } else {
                if self.offset_map.contains_key(&t.s_value) {
                    let (ty, offset) = self.offset_map.get(&t.s_value).unwrap();
                    Node {
                        token: Some(t.clone()),
                        nt: NodeType::LVar,
                        cty: Some(ty.clone()),
                        offset: Some(offset.clone()),
                        ..Node::default()
                    }
                } else if self.global_variables.contains_key(&t.s_value) {
                    let ty = self.global_variables.get(&t.s_value).unwrap();
                    Node {
                        token: Some(t.clone()),
                        nt: NodeType::GVar,
                        cty: Some(ty.clone()),
                        global_name: t.s_value.clone(),
                        ..Node::default()
                    }
                } else {
                    error_at(self.code, t.pos, "undefined variable");
                    unreachable!();
                }
            }
        } else if let Some(t) = self.consume(TokenType::Str) {
            self.string_literals.push(t.s_value.clone());
            let node = Node {
                token: Some(t.clone()),
                nt: NodeType::GVar,
                dest: format!("L_.str.{}", self.string_literals.len()-1),
                ..Node::default()
            };
            Node {
                token: Some(t.clone()),
                nt: NodeType::Addr,
                cty: Some(Type::Ptr(Box::new(Type::Char))),
                lhs: Some(Box::new(node)),
                ..Node::default()
            }
        } else {
            let t = self.expect_number();
            Node {
                token: Some(t.clone()),
                nt: NodeType::Num,
                value: Some(t.i_value),
                cty: Some(Type::Int),
                ..Node::default()
            }
        };
        if let Some(b_token) = self.consume_reserved("[") {
            let rhs = self.expr();
            self.expect("]");
            let addition = Node {
                token: Some(b_token.clone()),
                nt: NodeType::Add,
                lhs: Some(Box::new(node)),
                rhs: Some(Box::new(rhs)),
                ..Node::default()
            };
            node = Node {
                token: Some(b_token.clone()),
                nt: NodeType::Deref,
                lhs: Some(Box::new(addition)),
                ..Node::default()
            }
        }
        node
    }
}