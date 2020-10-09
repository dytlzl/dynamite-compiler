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
    pub global_functions: HashMap<String, Func>,
    pub global_variables: HashMap<String, Type>,
    pub string_literals: Vec<String>,
}

#[derive(Default)]
pub struct Func {
    pub body: Option<Node>,
    pub arg_types: Vec<Type>,
    pub return_type: Type,
    pub offset_size: usize,
    pub token: Option<Token>,
    pub args: Vec<Node>,
}

impl<'a> AstBuilder<'a> {
    pub fn new(code: &'a str, tokens: &'a Vec<Token>) -> Self {
        let mut builder = Self {
            code,
            tokens,
            cur: 0,
            offset_size: 0,
            offset_map: HashMap::new(),
            global_functions: HashMap::new(),
            global_variables: HashMap::new(),
            string_literals: Vec::new(),
        };
        builder.global_functions.insert(
            String::from("printf"),
            Func {
                arg_types: vec![Type::Ptr(Box::new(Type::Char))],
                return_type: Type::Int,
                ..Func::default()
            },
        );
        builder.global_functions.insert(
            String::from("exit"),
            Func {
                body: None,
                arg_types: vec![Type::Int],
                return_type: Type::Int,
                ..Func::default()
            },
        );
        builder
    }
    fn consume_str(&mut self, s_value: &str) -> Option<Token> {
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
    pub fn build(&mut self) {
        while !self.at_eof() {
            self.global_definition()
        }
    }
    fn expect_ident_with_type(&mut self, ty: Type) -> (Token, Type) {
        let mut ty = ty.clone();
        while let Some(_) = self.consume_str("*") {
            ty = Type::Ptr(Box::new(ty));
        }
        if let Some(t) = self.consume_ident() {
            if let Some(_) = self.consume_str("[") {
                let n = self.expect_number();
                ty = Type::Arr(Box::new(ty), n.i_value);
                self.expect("]");
            }
            (t, ty)
        } else {
            error_at(self.code, self.tokens[self.cur].pos, "ident expected");
            unreachable!();
        }
    }
    fn new_local_variable(&mut self, ty: Type) -> Node {
        let (t, ty) = self.expect_ident_with_type(ty);
        self.offset_size += ty.size_of();
        let segment_size =
            if let Some(dest) = ty.dest_type() { dest.size_of() } else { ty.size_of() };
        self.offset_size += segment_size - self.offset_size % segment_size;

        self.offset_map.insert(t.s_value.clone(), (ty.clone(), self.offset_size));
        let mut node = Node {
            token: Some(t),
            nt: NodeType::LVar,
            cty: Some(ty),
            offset: Some(self.offset_size),
            ..Node::default()
        };
        if let Some(t) = self.consume_str("=") {
            node = Node::new_with_op(
                Some(t),
                NodeType::Asg,
                node,
                self.assign());
        }
        node
    }
    fn consume_type(&mut self) -> Option<Type> {
        if let Some(_) = self.consume_str("int") {
            Some(Type::Int)
        } else if let Some(_) = self.consume_str("char") {
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
    fn global_definition(&mut self) {
        self.offset_size = 0;
        self.offset_map = HashMap::new();
        let ty = self.expect_type();
        let cur_to_back = self.cur;
        let (t, return_type) = self.expect_ident_with_type(ty.clone());
        if let Some(_) = self.consume_str("(") {
            let mut args: Vec<Node> = Vec::new();
            if let None = self.consume_str(")") {
                let ty = self.expect_type();
                args.push(self.new_local_variable(ty));
                while let None = self.consume_str(")") {
                    self.expect(",");
                    let ty = self.expect_type();
                    args.push(self.new_local_variable(ty));
                }
                if args.len() >= 7 {
                    error_at(self.code, t.pos, "count of args must be less than 7")
                }
            }
            let arg_types: Vec<Type> = args.iter().map(
                |arg| { arg.resolve_type().clone().unwrap() }
            ).collect();
            self.global_functions.insert(
                t.s_value.clone(),
                Func {
                    arg_types: arg_types.iter().map(|ty| ty.clone() ).collect(),
                    return_type: return_type.clone(),
                    token: Some(t.clone()),
                    ..Func::default()
                });
            let body = self.consume_block();
            self.global_functions.insert(
                t.s_value.clone(),
                Func {
                    body,
                    arg_types,
                    return_type,
                    offset_size: self.offset_size,
                    token: Some(t.clone()),
                    args,
                });
        } else {
            self.cur = cur_to_back; // back the cursor
            loop {
                let (t, ty) = self.expect_ident_with_type(ty.clone());
                self.global_variables.insert(t.s_value.clone(), ty.clone());
                if let None = self.consume_str(",") {
                    break;
                }
            }
            self.expect(";");
        }
    }
    fn stmt(&mut self) -> Node {
        if let Some(t) = self.consume_str("if") {
            self.expect("(");
            let cond = self.expr();
            self.expect(")");
            let then = self.stmt();
            let mut els: Option<Node> = None;
            if let Some(_) = self.consume_str("else") {
                els = Some(self.stmt());
            }
            return Node::new_if_node(Some(t), cond, then, els);
        }
        if let Some(t) = self.consume_str("while") {
            self.expect("(");
            let cond = self.expr();
            self.expect(")");
            return Node::new_while_node(Some(t), cond, self.stmt());
        }
        if let Some(t) = self.consume_str("for") {
            self.expect("(");
            let mut ini: Option<Node> = None;
            let mut cond: Option<Node> = None;
            let mut upd: Option<Node> = None;
            if let None = self.consume_str(";") {
                ini = Some(
                    if let Some(ty) = self.consume_type() {
                        self.define_local_variable(ty)
                    } else {
                        self.expr()
                    }
                );
                self.expect(";");
            }
            if let None = self.consume_str(";") {
                cond = Some(self.expr());
                self.expect(";");
            }
            if let None = self.consume_str(")") {
                upd = Some(self.expr());
                self.expect(")");
            }
            return Node::new_for_node(Some(t), ini, cond, upd, self.stmt());
        }
        if let Some(node) = self.consume_block() {
            return node;
        }
        let node = if let Some(ty) = self.consume_type() {
            self.define_local_variable(ty)
        } else if let Some(t) = self.consume_str("break") {
            Node {
                token: Some(t),
                nt: NodeType::Brk,
                ..Node::default()
            }
        } else if let Some(t) = self.consume_str("return") {
            Node::new_with_op_and_lhs(Some(t), NodeType::Ret, self.expr())
        } else {
            self.expr()
        };
        self.expect(";");
        node
    }

    pub fn define_local_variable(&mut self, ty: Type) -> Node {
        let mut vec = Vec::new();
        loop {
            vec.push(self.new_local_variable(ty.clone()));
            if let None = self.consume_str(",") {
                break;
            }
        }
        Node {
            nt: NodeType::DefVar,
            children: vec,
            ..Node::default()
        }
    }

    pub fn consume_block(&mut self) -> Option<Node> {
        if let Some(t) = self.consume_str("{") {
            let mut children: Vec<Node> = Vec::new();
            while let None = self.consume_str("}") {
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
        if let Some(t) = self.consume_str("=") {
            // left-associative => while, right-associative => recursive function
            node = Node::new_with_op(Some(t), NodeType::Asg, node, self.assign())
        }
        node
    }
    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        loop {
            if let Some(t) = self.consume_str("==") {
                node = Node::new_with_op(Some(t), NodeType::Eq, node, self.relational())
            } else if let Some(t) = self.consume_str("!=") {
                node = Node::new_with_op(Some(t), NodeType::Ne, node, self.relational())
            } else {
                return node;
            }
        }
    }
    fn relational(&mut self) -> Node {
        let mut node = self.add();
        loop {
            if let Some(t) = self.consume_str("<") {
                node = Node::new_with_op(Some(t), NodeType::Lt, node, self.add())
            } else if let Some(t) = self.consume_str("<=") {
                node = Node::new_with_op(Some(t), NodeType::Le, node, self.add())
            } else if let Some(t) = self.consume_str(">") {
                node = Node::new_with_op(Some(t), NodeType::Lt, self.add(), node)
            } else if let Some(t) = self.consume_str(">=") {
                node = Node::new_with_op(Some(t), NodeType::Le, self.add(), node)
            } else {
                return node;
            }
        }
    }
    fn add(&mut self) -> Node {
        let mut node = self.mul();
        loop {
            if let Some(t) = self.consume_str("+") {
                node = Node::new_with_op(Some(t), NodeType::Add, node, self.mul())
            } else if let Some(t) = self.consume_str("-") {
                node = Node::new_with_op(Some(t), NodeType::Sub, node, self.mul())
            } else {
                return node;
            }
        }
    }
    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        loop {
            if let Some(t) = self.consume_str("*") {
                node = Node::new_with_op(Some(t), NodeType::Mul, node, self.unary())
            } else if let Some(t) = self.consume_str("/") {
                node = Node::new_with_op(Some(t), NodeType::Div, node, self.unary())
            } else if let Some(t) = self.consume_str("%") {
                node = Node::new_with_op(Some(t), NodeType::Mod, node, self.unary())
            } else {
                return node;
            }
        }
    }
    fn unary(&mut self) -> Node {
        if let Some(t) = self.consume_str("sizeof") {
            return Node {
                token: Some(t),
                value: Some(self.unary().resolve_type().unwrap().size_of()),
                cty: Some(Type::Int),
                ..Node::default()
            };
        }
        if let Some(_) = self.consume_str("+") {} else if let Some(t) = self.consume_str("-") {
            return Node::new_with_op(Some(t), NodeType::Sub, Node::new_with_num(None, 0), self.prim());
        }
        if let Some(t) = self.consume_str("&") {
            return Node {
                token: Some(t),
                nt: NodeType::Addr,
                lhs: Some(Box::new(self.unary())),
                ..Node::default()
            };
        }
        if let Some(t) = self.consume_str("*") {
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
        let mut node = if let Some(_) = self.consume_str("(") {
            let node = self.expr();
            self.expect(")");
            node
        } else if let Some(t) = self.consume_ident() {
            if let Some(_) = self.consume_str("(") {
                if !self.global_functions.contains_key(&t.s_value) {
                    error_at(self.code, t.pos, "undefined function");
                }
                let return_type =
                    self.global_functions.get(&t.s_value).unwrap().return_type.clone();
                let mut args: Vec<Node> = Vec::new();
                if let None = self.consume_str(")") {
                    args.push(self.expr());
                    while let None = self.consume_str(")") {
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
                    global_name: String::from(s_value),
                    cty: Some(return_type),
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
                dest: format!("L_.str.{}", self.string_literals.len() - 1),
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
        if let Some(b_token) = self.consume_str("[") {
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

