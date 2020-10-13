use crate::token::{Token, TokenType};
use crate::node::{Node, NodeType};
use crate::error::{error_at};
use std::collections::HashMap;
use crate::ctype::Type;
use crate::func::Func;
use crate::global::{GlobalVariable, GlobalVariableData};
use std::mem::swap;

pub struct ASTBuilder<'a> {
    code: &'a str,
    tokens: &'a Vec<Token>,
    cur: usize,
    pub offset_size: usize,
    offset_map: HashMap<String, (Type, usize)>,
    pub functions: HashMap<String, Func>,
    pub global_variables: HashMap<String, GlobalVariable>,
    pub string_literals: Vec<String>,
}

impl<'a> ASTBuilder<'a> {
    pub fn new(code: &'a str, tokens: &'a Vec<Token>) -> Self {
        let mut builder = Self {
            code,
            tokens,
            cur: 0,
            offset_size: 0,
            offset_map: HashMap::new(),
            functions: HashMap::new(),
            global_variables: HashMap::new(),
            string_literals: Vec::new(),
        };
        builder.functions.insert(
            String::from("printf"),
            Func {
                arg_types: vec![Type::Ptr(Box::new(Type::Char))],
                return_type: Type::Int,
                ..Func::default()
            },
        );
        builder.functions.insert(
            String::from("puts"),
            Func {
                arg_types: vec![Type::Ptr(Box::new(Type::Char))],
                return_type: Type::Int,
                ..Func::default()
            },
        );
        builder.functions.insert(
            String::from("putchar"),
            Func {
                arg_types: vec![Type::Char],
                return_type: Type::Int,
                ..Func::default()
            },
        );
        builder.functions.insert(
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
        self.cur >= self.tokens.len()
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
            while let Some(_) = self.consume_str("[") {
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
        Node {
            token: Some(t),
            nt: NodeType::LocalVar,
            cty: Some(ty),
            offset: Some(self.offset_size),
            ..Node::default()
        }
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
        if let Some(_) = self.consume_str("(") { // function
            let mut args: Vec<Node> = Vec::new();
            if let None = self.consume_str(")") {
                loop {
                    let ty = self.expect_type();
                    args.push(self.new_local_variable(ty));
                    if let None = self.consume_str(",") { break; }
                }
                self.expect(")");
            }
            if args.len() >= 7 {
                error_at(self.code, t.pos, "count of args must be less than 7")
            }
            let arg_types: Vec<Type> = args.iter().map(
                |arg| { arg.resolve_type().clone().unwrap() }
            ).collect();
            self.functions.insert(
                t.s_value.clone(),
                Func {
                    arg_types: arg_types.iter().map(|ty| ty.clone()).collect(),
                    return_type: return_type.clone(),
                    token: Some(t.clone()),
                    ..Func::default()
                });
            let body = self.consume_block();
            self.functions.insert(
                t.s_value.clone(),
                Func {
                    body,
                    arg_types,
                    return_type,
                    offset_size: self.offset_size,
                    token: Some(t.clone()),
                    args,
                });
        } else { // global variable
            self.cur = cur_to_back; // back the cursor
            loop {
                let (t, ty) = self.expect_ident_with_type(ty.clone());
                let data = if let Some(_) = self.consume_str("=") {
                    Some(self.global_data())
                } else {
                    None
                };
                self.global_variables.insert(
                    t.s_value.clone(),
                    GlobalVariable { ty: ty.clone(), data });
                if let None = self.consume_str(",") {
                    break;
                }
            }
            self.expect(";");
        }
    }
    fn global_data(&mut self) -> GlobalVariableData {
        if let Some(_) = self.consume_str("{") {
            let mut vec = Vec::new();
            if let None = self.consume_str("}") {
                loop {
                    vec.push(self.global_data());
                    if let None = self.consume_str(",") { break; }
                }
                self.expect("}");
            }
            GlobalVariableData::Arr(vec)
        } else if let Some(t) = self.consume(TokenType::Str) {
            GlobalVariableData::Elem(self.new_string_literal(&t.s_value))
        } else {
            let equality = self.equality();
            GlobalVariableData::Elem(format!("{}", self.eval(&equality)))
        }
    }
    fn eval(&mut self, node: &Node) -> i64 {
        match node.nt {
            NodeType::Num => node.value.unwrap() as i64,
            NodeType::Eq => {
                (
                    self.eval(node.lhs.as_ref().unwrap()) == self.eval(node.rhs.as_ref().unwrap())
                ) as i64
            }
            NodeType::Ne => {
                (
                    self.eval(node.lhs.as_ref().unwrap()) != self.eval(node.rhs.as_ref().unwrap())
                ) as i64
            }
            NodeType::Le => {
                (
                    self.eval(node.lhs.as_ref().unwrap()) <= self.eval(node.rhs.as_ref().unwrap())
                ) as i64
            }
            NodeType::Lt => {
                (
                    self.eval(node.lhs.as_ref().unwrap()) < self.eval(node.rhs.as_ref().unwrap())
                ) as i64
            }
            NodeType::Add => {
                self.eval(node.lhs.as_ref().unwrap()) + self.eval(node.rhs.as_ref().unwrap())
            }
            NodeType::Sub => {
                self.eval(node.lhs.as_ref().unwrap()) - self.eval(node.rhs.as_ref().unwrap())
            }
            NodeType::Mul => {
                self.eval(node.lhs.as_ref().unwrap()) * self.eval(node.rhs.as_ref().unwrap())
            }
            NodeType::Div => {
                self.eval(node.lhs.as_ref().unwrap()) / self.eval(node.rhs.as_ref().unwrap())
            }
            NodeType::Mod => {
                self.eval(node.lhs.as_ref().unwrap()) % self.eval(node.rhs.as_ref().unwrap())
            }
            _ => {
                error_at(self.code, node.token.as_ref().unwrap().pos,
                         "initializer element is not a compile-time constant");
                unreachable!()
            }
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
                        self.local_variable_definition(ty)
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
            self.local_variable_definition(ty)
        } else if let Some(t) = self.consume_str("break") {
            Node {
                token: Some(t),
                nt: NodeType::Break,
                ..Node::default()
            }
        } else if let Some(t) = self.consume_str("return") {
            Node::new_with_op_and_lhs(Some(t), NodeType::Return, self.expr())
        } else {
            self.expr()
        };
        self.expect(";");
        node
    }

    pub fn local_variable_definition(&mut self, ty: Type) -> Node {
        let mut vec = Vec::new();
        loop {
            let node = self.new_local_variable(ty.clone());
            if let Some(token) = self.consume_str("=") {
                // if initializer element exists, push into AST
                vec.push(self.local_variable_initialization(&node, &token));
            }
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

    fn local_variable_initialization(&mut self, node: &Node, assign_token: &Token) -> Node {
        if let Some(b_token) = self.consume_str("{") {
            let mut vec = Vec::new();
            if let None = self.consume_str("}") {
                let mut index = 0;
                loop {
                    let rhs = Node::new_with_num(None, index);
                    let mut node = Node {
                        token: Some(b_token.clone()),
                        nt: NodeType::Add,
                        lhs: Some(Box::new(node.clone())),
                        rhs: Some(Box::new(rhs)),
                        ..Node::default()
                    };
                    node = Node {
                        token: Some(b_token.clone()),
                        nt: NodeType::Deref,
                        lhs: Some(Box::new(node)),
                        ..Node::default()
                    };
                    vec.push(self.local_variable_initialization(&node, &assign_token));
                    if let None = self.consume_str(",") { break; }
                    index += 1;
                }
                self.expect("}");
            }
            Node {
                token: Some(assign_token.clone()),
                nt: NodeType::DefVar,
                children: vec,
                ..Node::default()
            }
        } else {
            Node {
                token: Some(assign_token.clone()),
                nt: NodeType::Assign,
                lhs: Some(Box::new(node.clone())),
                rhs: Some(Box::new(self.expr())),
                ..Node::default()
            }
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
            node = Node::new_with_op(Some(t), NodeType::Assign, node, self.assign())
        } else if let Some(t) = self.consume_str("+=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Add, node, self.assign()))
        } else if let Some(t) = self.consume_str("-=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Sub, node, self.assign()))
        } else if let Some(t) = self.consume_str("*=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Mul, node, self.assign()))
        } else if let Some(t) = self.consume_str("/=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Div, node, self.assign()))
        } else if let Some(t) = self.consume_str("%=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Mod, node, self.assign()))
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
                if !self.functions.contains_key(&t.s_value) {
                    error_at(self.code, t.pos, "undefined function");
                }
                let return_type =
                    self.functions.get(&t.s_value).unwrap().return_type.clone();
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
                    nt: NodeType::CallFunc,
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
                        nt: NodeType::LocalVar,
                        cty: Some(ty.clone()),
                        offset: Some(offset.clone()),
                        ..Node::default()
                    }
                } else if self.global_variables.contains_key(&t.s_value) {
                    let ty = self.global_variables.get(&t.s_value).unwrap().ty.clone();
                    Node {
                        token: Some(t.clone()),
                        nt: NodeType::GlobalVar,
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
            // String literal
            let node = Node {
                token: Some(t.clone()),
                nt: NodeType::GlobalVar,
                dest: self.new_string_literal(&t.s_value),
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
        while let Some(b_token) = self.consume_str("[") {
            // Subscript array
            let mut rhs = self.expr();
            self.expect("]");
            if let Some(Type::Arr(..)) = node.resolve_type() {} else {
                if let Some(Type::Arr(..)) = rhs.resolve_type() {
                    swap(&mut node, &mut rhs);
                }
            }
            node = Node {
                token: Some(b_token.clone()),
                nt: NodeType::Add,
                lhs: Some(Box::new(node)),
                rhs: Some(Box::new(rhs)),
                ..Node::default()
            };
            node = Node {
                token: Some(b_token.clone()),
                nt: NodeType::Deref,
                lhs: Some(Box::new(node)),
                ..Node::default()
            }
        }
        node
    }

    fn new_string_literal(&mut self, s: &str) -> String {
        self.string_literals.push(s.to_string());
        format!("L_.str.{}", self.string_literals.len() - 1)
    }

    pub fn print_functions(&self) {
        self.functions.iter().for_each(|(s, f)| {
            if let Some(..) = f.body {
                eprintln!("[FUNC: {}]", s);
                f.print();
            }
        })
    }
}

