use crate::token::Token;
use crate::ctype::Type;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Assign,
    LocalVar,
    GlobalVar,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Num,
    Return,
    If,
    While,
    For,
    Break,
    Block,
    CallFunc,
    Addr,
    Deref,
    DefVar,
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Num
    }
}


#[derive(Default)]
pub struct Node {
    pub nt: NodeType,
    pub token: Option<Token>,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    pub els: Option<Box<Node>>,
    pub ini: Option<Box<Node>>,
    pub upd: Option<Box<Node>>,
    pub children: Vec<Node>,
    pub body: Option<Box<Node>>,
    pub value: Option<usize>,
    pub global_name: String,
    pub dest: String,
    pub args: Vec<Node>,
    pub cty: Option<Type>,
    pub offset: Option<usize>,
}

impl Node {
    pub fn new_with_op(token: Option<Token>, nt: NodeType, lhs: Node, rhs: Node) -> Self {
        Self {
            token,
            nt,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            ..Self::default()
        }
    }
    pub fn new_with_op_and_lhs(token: Option<Token>, nt: NodeType, lhs: Node) -> Self {
        Self {
            token,
            nt,
            lhs: Some(Box::new(lhs)),
            ..Self::default()
        }
    }
    pub fn new_with_num(token: Option<Token>, value: usize) -> Self {
        Self {
            token,
            nt: NodeType::Num,
            value: Some(value),
            ..Self::default()
        }
    }
    pub fn new_with_ident(token: Option<Token>, nt: NodeType, offset: usize) -> Self {
        Self {
            token,
            nt,
            offset: Some(offset),
            ..Self::default()
        }
    }
    pub fn new_if_node(token: Option<Token>, cond: Node, then: Node, els: Option<Node>) -> Self {
        Self {
            token,
            nt: NodeType::If,
            cond: Some(Box::new(cond)),
            then: Some(Box::new(then)),
            els: if let Some(els) = els { Some(Box::new(els)) } else { None },
            ..Self::default()
        }
    }
    pub fn new_while_node(token: Option<Token>, cond: Node, then: Node) -> Self {
        Self {
            token,
            nt: NodeType::While,
            cond: Some(Box::new(cond)),
            then: Some(Box::new(then)),
            ..Self::default()
        }
    }
    pub fn new_for_node(token: Option<Token>, ini: Option<Node>, cond: Option<Node>, upd: Option<Node>, then: Node) -> Self {
        Self {
            token,
            nt: NodeType::For,
            ini: if let Some(d) = ini { Some(Box::new(d)) } else { None },
            cond: if let Some(d) = cond { Some(Box::new(d)) } else { None },
            upd: if let Some(d) = upd { Some(Box::new(d)) } else { None },
            then: Some(Box::new(then)),
            ..Self::default()
        }
    }
    pub fn resolve_type(&self) -> Option<Type> {
        match self.nt {
            NodeType::LocalVar | NodeType::Num | NodeType::CallFunc | NodeType::GlobalVar => { self.cty.clone() }
            NodeType::Addr => {
                if let Some(ty) = self.lhs.as_ref().unwrap().resolve_type() {
                    Some(Type::Ptr(Box::new(ty)))
                } else {
                    self.cty.clone()
                }
            }
            NodeType::Deref => {
                self.lhs.as_ref().unwrap().dest_type()
            }
            _ => {
                if let Some(lhs) = self.lhs.as_ref() {
                    if let (None, Some(rhs)) = (lhs.dest_type(), self.rhs.as_ref()) {
                        if let Some(_) = rhs.dest_type() {
                            return rhs.resolve_type()
                        }
                    }
                    lhs.resolve_type()
                } else {
                    unreachable!();
                }
            }
        }
    }
    pub fn dest_type(&self) -> Option<Type> {
        if let Some(t) = self.resolve_type() {
            t.dest_type()
        } else {
            None
        }
    }
}
