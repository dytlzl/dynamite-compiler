use crate::token::{Token, TokenType};
use crate::node::{Node, NodeType};
use crate::error::{error_at};
use std::collections::HashMap;
use crate::ctype::Type;
use crate::func::Func;
use crate::global::{GlobalVariable, GlobalVariableData};

pub struct ASTBuilder<'a> {
    code: &'a str,
    tokens: &'a Vec<Token>,
    cur: usize,
    offset_size: usize,
    scope_stack: Vec<HashMap<String, Identifier>>,
    pub functions: HashMap<String, Func>,
    pub global_variables: HashMap<String, GlobalVariable>,
    pub string_literals: Vec<String>,
}

pub enum Identifier {
    TypeDef(Type),
    Local(Type, usize),
    Global(Type),
}

impl<'a> ASTBuilder<'a> {
    pub fn new(code: &'a str, tokens: &'a Vec<Token>) -> Self {
        let mut builder = Self {
            code,
            tokens,
            cur: 0,
            offset_size: 0,
            scope_stack: Vec::new(),
            functions: HashMap::new(),
            global_variables: HashMap::new(),
            string_literals: Vec::new(),
        };
        let mut map = HashMap::new();
        map.insert(
            String::from("printf"),
            Identifier::Global(Type::Func(vec![], Box::new(Type::Int))),
        );
        map.insert(
            String::from("puts"),
            Identifier::Global(Type::Func(vec![], Box::new(Type::Int))),
        );
        map.insert(
            String::from("putchar"),
            Identifier::Global(Type::Func(vec![], Box::new(Type::Int))),
        );
        map.insert(
            String::from("exit"),
            Identifier::Global(Type::Func(vec![], Box::new(Type::Int))),
        );
        builder.scope_stack.push(map);
        builder
    }
    fn attempt_reserved(&mut self, s_value: &str) -> Option<Token> {
        if let TokenType::Reserved = self.tokens[self.cur].tt {
            if self.tokens[self.cur].s_value == s_value {
                self.cur += 1;
                return Some(self.tokens[self.cur - 1].clone());
            }
        }
        None
    }
    fn attempt_ident(&mut self) -> Option<Token> {
        if let TokenType::Ident = self.tokens[self.cur].tt {
            self.cur += 1;
            return Some(self.tokens[self.cur - 1].clone());
        } else {
            None
        }
    }
    fn expect_reserved(&mut self, s_value: &str) -> Token {
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
    fn attempt(&mut self, tt: TokenType) -> Option<Token> {
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
        while let Some(_) = self.attempt_reserved("*") {
            ty = Type::Ptr(Box::new(ty));
        }
        if let Some(t) = self.attempt_ident() {
            while let Some(_) = self.attempt_reserved("[") {
                let n = self.expect_number();
                ty = Type::Arr(Box::new(ty), n.i_value);
                self.expect_reserved("]");
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
        if self.scope_stack.last().unwrap().contains_key(&t.s_value) {
            error_at(self.code, t.pos, "invalid redeclaration");
        }
        self.scope_stack.last_mut().unwrap().insert(t.s_value.clone(), Identifier::Local(ty.clone(), self.offset_size));
        Node {
            token: Some(t),
            nt: NodeType::LocalVar,
            cty: Some(ty),
            offset: Some(self.offset_size),
            ..Node::default()
        }
    }
    fn attempt_type(&mut self) -> Option<Type> {
        if let Some(_) = self.attempt_reserved("int") {
            Some(Type::Int)
        } else if let Some(_) = self.attempt_reserved("char") {
            Some(Type::Char)
        } else {
            None
        }
    }
    fn expect_type(&mut self) -> Type {
        if let Some(ty) = self.attempt_type() {
            ty
        } else {
            error_at(self.code, self.tokens[self.cur].pos, "type expected");
            unreachable!()
        }
    }
    fn global_definition(&mut self) {
        let ty = self.expect_type();
        let cur_to_back = self.cur;
        let (t, return_type) = self.expect_ident_with_type(ty.clone());
        if let Some(_) = self.attempt_reserved("(") { // function
            self.offset_size = 0;
            self.scope_stack.push(HashMap::new());
            let mut args: Vec<Node> = Vec::new();
            if let None = self.attempt_reserved(")") {
                loop {
                    let ty = self.expect_type();
                    args.push(self.new_local_variable(ty));
                    if let None = self.attempt_reserved(",") { break; }
                }
                self.expect_reserved(")");
            }
            if args.len() >= 7 {
                error_at(self.code, t.pos, "count of args must be less than 7")
            }
            let arg_types: Vec<Type> = args.iter().map(
                |arg| { arg.resolve_type().clone().unwrap() }
            ).collect();
            self.scope_stack.first_mut().unwrap().insert(
                t.s_value.clone(),
                Identifier::Global(
                    Type::Func(
                        arg_types.clone(),
                        Box::new(return_type.clone()))));
            self.functions.insert(
                t.s_value.clone(),
                Func {
                    cty: Type::Func(
                        arg_types.clone(),
                        Box::new(return_type.clone())),
                    token: Some(t.clone()),
                    ..Func::default()
                });
            let body = self.consume_block();
            self.scope_stack.pop();
            self.functions.insert(
                t.s_value.clone(),
                Func {
                    body,
                    cty: Type::Func(
                        arg_types.clone(),
                        Box::new(return_type.clone())),
                    offset_size: self.offset_size,
                    token: Some(t.clone()),
                    args,
                });
        } else { // global variable
            self.cur = cur_to_back; // back the cursor
            loop {
                let (t, ty) = self.expect_ident_with_type(ty.clone());
                let data = if let Some(_) = self.attempt_reserved("=") {
                    Some(self.global_data())
                } else {
                    None
                };
                self.scope_stack.last_mut().unwrap().insert(
                    t.s_value.clone(),
                    Identifier::Global(ty.clone()));
                self.global_variables.insert(
                    t.s_value.clone(),
                    GlobalVariable { ty: ty.clone(), data });
                if let None = self.attempt_reserved(",") {
                    break;
                }
            }
            self.expect_reserved(";");
        }
    }
    fn resolve_name(&mut self, s: &str) -> Option<&Identifier> {
        for map in self.scope_stack.iter().rev() {
            if map.contains_key(s) {
                return map.get(s);
            }
        }
        return None;
    }
    fn global_data(&mut self) -> GlobalVariableData {
        if let Some(_) = self.attempt_reserved("{") {
            let mut vec = Vec::new();
            if let None = self.attempt_reserved("}") {
                loop {
                    vec.push(self.global_data());
                    if let None = self.attempt_reserved(",") { break; }
                }
                self.expect_reserved("}");
            }
            GlobalVariableData::Arr(vec)
        } else if let Some(t) = self.attempt(TokenType::Str) {
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
        if let Some(t) = self.attempt_reserved("if") {
            self.expect_reserved("(");
            let cond = self.expr();
            self.expect_reserved(")");
            let then = self.stmt();
            let mut els: Option<Node> = None;
            if let Some(_) = self.attempt_reserved("else") {
                els = Some(self.stmt());
            }
            return Node::new_if_node(Some(t), cond, then, els);
        }
        if let Some(t) = self.attempt_reserved("while") {
            self.expect_reserved("(");
            let cond = self.expr();
            self.expect_reserved(")");
            return Node::new_while_node(Some(t), cond, self.stmt());
        }
        if let Some(t) = self.attempt_reserved("for") {
            self.scope_stack.push(HashMap::new());
            self.expect_reserved("(");
            let mut ini: Option<Node> = None;
            let mut cond: Option<Node> = None;
            let mut upd: Option<Node> = None;
            if let None = self.attempt_reserved(";") {
                ini = Some(
                    if let Some(ty) = self.attempt_type() {
                        self.local_variable_definition(ty)
                    } else {
                        self.expr()
                    }
                );
                self.expect_reserved(";");
            }
            if let None = self.attempt_reserved(";") {
                cond = Some(self.expr());
                self.expect_reserved(";");
            }
            if let None = self.attempt_reserved(")") {
                upd = Some(self.expr());
                self.expect_reserved(")");
            }
            self.scope_stack.pop();
            return Node::new_for_node(Some(t), ini, cond, upd, self.stmt());
        }
        if let Some(node) = self.consume_block() {
            return node;
        }
        let node = if let Some(ty) = self.attempt_type() {
            self.local_variable_definition(ty)
        } else if let Some(t) = self.attempt_reserved("break") {
            Node {
                token: Some(t),
                nt: NodeType::Break,
                ..Node::default()
            }
        } else if let Some(t) = self.attempt_reserved("return") {
            Node::new_with_op_and_lhs(Some(t), NodeType::Return, self.expr())
        } else {
            self.expr()
        };
        self.expect_reserved(";");
        node
    }

    pub fn local_variable_definition(&mut self, ty: Type) -> Node {
        let mut vec = Vec::new();
        loop {
            let node = self.new_local_variable(ty.clone());
            if let Some(token) = self.attempt_reserved("=") {
                // if initializer element exists, push into AST
                vec.push(self.local_variable_initialization(&node, &token));
            }
            if let None = self.attempt_reserved(",") {
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
        if let Some(b_token) = self.attempt_reserved("{") {
            let mut vec = Vec::new();
            if let None = self.attempt_reserved("}") {
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
                    if let None = self.attempt_reserved(",") { break; }
                    index += 1;
                }
                self.expect_reserved("}");
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
        if let Some(t) = self.attempt_reserved("{") {
            self.scope_stack.push(HashMap::new());
            let mut children: Vec<Node> = Vec::new();
            while let None = self.attempt_reserved("}") {
                children.push(self.stmt());
            }
            self.scope_stack.pop();
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
        let mut node = self.ternary();
        if let Some(t) = self.attempt_reserved("=") {
            // left-associative => while, right-associative => recursive function
            node = Node::new_with_op(Some(t), NodeType::Assign, node, self.assign())
        } else if let Some(t) = self.attempt_reserved("+=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Add, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("-=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Sub, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("*=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Mul, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("/=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Div, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("%=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Mod, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("<<=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::BitLeft, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved(">>=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::BitRight, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("&=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::BitAnd, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("^=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::BitXor, node, self.assign()))
        } else if let Some(t) = self.attempt_reserved("|=") {
            node = Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::BitOr, node, self.assign()))
        }
        node
    }
    fn ternary(&mut self) -> Node {
        let node = self.logical_or();
        if let Some(t) = self.attempt_reserved("?") {
            let then = self.logical_or();
            self.expect_reserved(":");
            let els = self.logical_or();
            return Node::new_if_node(Some(t), node, then, Some(els));
        }
        return node;
    }
    fn logical_or(&mut self) -> Node {
        let mut node = self.logical_and();
        loop {
            if let Some(t) = self.attempt_reserved("||") {
                node = Node::new_with_op(Some(t), NodeType::LogicalOr, node, self.logical_and());
            } else {
                return node;
            }
        }
    }
    fn logical_and(&mut self) -> Node {
        let mut node = self.bitwise_or();
        loop {
            if let Some(t) = self.attempt_reserved("&&") {
                node = Node::new_with_op(Some(t), NodeType::LogicalAnd, node, self.bitwise_or());
            } else {
                return node;
            }
        }
    }
    fn bitwise_or(&mut self) -> Node {
        let mut node = self.bitwise_xor();
        loop {
            if let Some(t) = self.attempt_reserved("|") {
                node = Node::new_with_op(Some(t), NodeType::BitOr, node, self.bitwise_xor())
            } else {
                return node;
            }
        }
    }
    fn bitwise_xor(&mut self) -> Node {
        let mut node = self.bitwise_and();
        loop {
            if let Some(t) = self.attempt_reserved("^") {
                node = Node::new_with_op(Some(t), NodeType::BitXor, node, self.bitwise_and())
            } else {
                return node;
            }
        }
    }
    fn bitwise_and(&mut self) -> Node {
        let mut node = self.equality();
        loop {
            if let Some(t) = self.attempt_reserved("&") {
                node = Node::new_with_op(Some(t), NodeType::BitAnd, node, self.equality())
            } else {
                return node;
            }
        }
    }
    fn equality(&mut self) -> Node {
        let mut node = self.relational();
        loop {
            if let Some(t) = self.attempt_reserved("==") {
                node = Node::new_with_op(Some(t), NodeType::Eq, node, self.relational())
            } else if let Some(t) = self.attempt_reserved("!=") {
                node = Node::new_with_op(Some(t), NodeType::Ne, node, self.relational())
            } else {
                return node;
            }
        }
    }
    fn relational(&mut self) -> Node {
        let mut node = self.bit_shift();
        loop {
            if let Some(t) = self.attempt_reserved("<") {
                node = Node::new_with_op(Some(t), NodeType::Lt, node, self.bit_shift())
            } else if let Some(t) = self.attempt_reserved("<=") {
                node = Node::new_with_op(Some(t), NodeType::Le, node, self.bit_shift())
            } else if let Some(t) = self.attempt_reserved(">") {
                node = Node::new_with_op(Some(t), NodeType::Lt, self.bit_shift(), node)
            } else if let Some(t) = self.attempt_reserved(">=") {
                node = Node::new_with_op(Some(t), NodeType::Le, self.bit_shift(), node)
            } else {
                return node;
            }
        }
    }
    fn bit_shift(&mut self) -> Node {
        let mut node = self.add();
        loop {
            if let Some(t) = self.attempt_reserved("<<") {
                node = Node::new_with_op(Some(t), NodeType::BitLeft, node, self.add())
            } else if let Some(t) = self.attempt_reserved(">>") {
                node = Node::new_with_op(Some(t), NodeType::BitRight, node, self.add())
            } else {
                return node;
            }
        }
    }
    fn add(&mut self) -> Node {
        let mut node = self.mul();
        loop {
            if let Some(t) = self.attempt_reserved("+") {
                node = Node::new_with_op(Some(t), NodeType::Add, node, self.mul())
            } else if let Some(t) = self.attempt_reserved("-") {
                node = Node::new_with_op(Some(t), NodeType::Sub, node, self.mul())
            } else {
                return node;
            }
        }
    }
    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        loop {
            if let Some(t) = self.attempt_reserved("*") {
                node = Node::new_with_op(Some(t), NodeType::Mul, node, self.unary())
            } else if let Some(t) = self.attempt_reserved("/") {
                node = Node::new_with_op(Some(t), NodeType::Div, node, self.unary())
            } else if let Some(t) = self.attempt_reserved("%") {
                node = Node::new_with_op(Some(t), NodeType::Mod, node, self.unary())
            } else {
                return node;
            }
        }
    }
    fn unary(&mut self) -> Node {
        if let Some(t) = self.attempt_reserved("sizeof") {
            return Node {
                token: Some(t),
                value: Some(self.unary().resolve_type().unwrap().size_of()),
                cty: Some(Type::Int),
                ..Node::default()
            };
        }
        if let Some(_) = self.attempt_reserved("+") {} else if let Some(t) = self.attempt_reserved("-") {
            return Node::new_with_op(Some(t), NodeType::Sub, Node::new_with_num(None, 0), self.prim());
        }
        if let Some(t) = self.attempt_reserved("&") {
            return Node {
                token: Some(t),
                nt: NodeType::Addr,
                lhs: Some(Box::new(self.unary())),
                ..Node::default()
            };
        }
        if let Some(t) = self.attempt_reserved("*") {
            return Node {
                token: Some(t),
                nt: NodeType::Deref,
                lhs: Some(Box::new(self.unary())),
                ..Node::default()
            };
        }
        if let Some(t) = self.attempt_reserved("~") {
            return Node {
                token: Some(t),
                nt: NodeType::BitNot,
                lhs: Some(Box::new(self.unary())),
                ..Node::default()
            };
        }
        if let Some(t) = self.attempt_reserved("!") {
            return Node {
                token: Some(t),
                nt: NodeType::Eq,
                lhs: Some(Box::new(self.unary())),
                rhs: Some(Box::new(Node::new_with_num(None, 0))),
                ..Node::default()
            };
        }
        if let Some(t) = self.attempt_reserved("++") {
            let node = self.unary();
            return Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Add, node, Node::new_with_num(None, 1)));
        }
        if let Some(t) = self.attempt_reserved("--") {
            let node = self.unary();
            return Node::new_with_op(
                Some(t.clone()), NodeType::Assign, node.clone(),
                Node::new_with_op(Some(t), NodeType::Sub, node, Node::new_with_num(None, 1)));
        }
        self.prim()
    }
    fn prim(&mut self) -> Node {
        let mut node = if let Some(_) = self.attempt_reserved("(") {
            let node = self.expr();
            self.expect_reserved(")");
            node
        } else if let Some(t) = self.attempt(TokenType::Str) {
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
        } else if let Some(t) = self.attempt(TokenType::Num) {
            // Number literal
            Node {
                token: Some(t.clone()),
                nt: NodeType::Num,
                value: Some(t.i_value),
                cty: Some(Type::Int),
                ..Node::default()
            }
        } else if let Some(t) = self.attempt_ident() {
            if let Some(ident) = self.resolve_name(&t.s_value) {
                match ident {
                    Identifier::Local(ty, offset) => {
                        Node {
                            token: Some(t.clone()),
                            nt: NodeType::LocalVar,
                            cty: Some(ty.clone()),
                            offset: Some(offset.clone()),
                            ..Node::default()
                        }
                    }
                    Identifier::Global(ty) => {
                        Node {
                            token: Some(t.clone()),
                            nt: NodeType::GlobalVar,
                            cty: Some(ty.clone()),
                            global_name: t.s_value.clone(),
                            ..Node::default()
                        }
                    }
                    Identifier::TypeDef(..) => {
                        unimplemented!()
                    }
                }
            } else {
                error_at(self.code, t.pos, "undefined variable");
                unreachable!();
            }
        } else {
            error_at(self.code, self.tokens[self.cur].pos, "unexpected token");
            unreachable!();
        };
        loop {
            if let Some(_) = self.attempt_reserved("(") {
                // Call function
                let t = node.token.clone().unwrap();
                let return_type = if let Some(Type::Func(_, return_type)) =
                node.resolve_type() {
                    *return_type.clone()
                } else { unreachable!() };
                let mut args: Vec<Node> = Vec::new();
                if let None = self.attempt_reserved(")") {
                    args.push(self.expr());
                    while let None = self.attempt_reserved(")") {
                        self.expect_reserved(",");
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
                node = Node {
                    token: Some(t),
                    nt: NodeType::CallFunc,
                    global_name: String::from(s_value),
                    cty: Some(return_type),
                    args,
                    ..Node::default()
                }
            } else if let Some(b_token) = self.attempt_reserved("[") {
                // Subscript array
                node = Node::new_with_op(Some(b_token.clone()), NodeType::Add, node, self.expr());
                self.expect_reserved("]");
                node = Node {
                    token: Some(b_token.clone()),
                    nt: NodeType::Deref,
                    lhs: Some(Box::new(node)),
                    ..Node::default()
                }
            } else if let Some(token) = self.attempt_reserved("++") {
                // Suffix increment
                node = Node::new_with_op_and_lhs(Some(token), NodeType::SuffixIncr, node);
            } else if let Some(token) = self.attempt_reserved("--") {
                // Suffix decrement
                node = Node::new_with_op_and_lhs(Some(token), NodeType::SuffixDecr, node);
            } else {
                return node;
            }
        }
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

