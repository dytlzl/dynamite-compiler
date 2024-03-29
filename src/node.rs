use crate::ctype::Type;
use crate::token::Token;
use std::mem::swap;

#[derive(Debug, PartialEq, Clone)]
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
    BitLeft,
    BitRight,
    BitAnd,
    BitXor,
    BitOr,
    BitNot,
    LogicalAnd,
    LogicalOr,
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
    SuffixIncr,
    SuffixDecr,
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Num
    }
}

#[derive(Default, Clone)]
pub struct Node {
    pub nt: NodeType,
    pub cty: Option<Type>,
    pub token: Option<Token>,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    // for number
    pub value: Option<usize>,
    // for "if", "for", "while" statement
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    // for "if" statement
    pub els: Option<Box<Node>>,
    // for "for" statement
    pub ini: Option<Box<Node>>,
    pub upd: Option<Box<Node>>,
    // for block, definition of local variables
    pub children: Vec<Node>,
    // flag of global variables
    pub global_name: String,
    // flag of string literal
    pub dest: String,
    // for calling function
    pub args: Vec<Node>,
    // for declaration of local variables
    pub offset: Option<usize>,
}

impl Node {
    pub fn new_with_op(token: Option<Token>, nt: NodeType, lhs: Node, rhs: Node) -> Self {
        let (mut lhs, mut rhs) = (lhs, rhs);
        if let NodeType::Add = nt {
            if lhs.dest_type().is_none() && rhs.dest_type().is_some() {
                swap(&mut lhs, &mut rhs);
            }
        }
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
    pub fn new_if_node(token: Option<Token>, cond: Node, then: Node, els: Option<Node>) -> Self {
        Self {
            token,
            nt: NodeType::If,
            cond: Some(Box::new(cond)),
            then: Some(Box::new(then)),
            els: els.map(Box::new),
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
    pub fn new_for_node(
        token: Option<Token>,
        ini: Option<Node>,
        cond: Option<Node>,
        upd: Option<Node>,
        then: Node,
    ) -> Self {
        Self {
            token,
            nt: NodeType::For,
            ini: ini.map(Box::new),
            cond: cond.map(Box::new),
            upd: upd.map(Box::new),
            then: Some(Box::new(then)),
            ..Self::default()
        }
    }
    pub fn resolve_type(&self) -> Option<Type> {
        match self.nt {
            NodeType::LocalVar | NodeType::Num | NodeType::CallFunc | NodeType::GlobalVar => {
                self.cty.clone()
            }
            NodeType::Addr => {
                if let Some(ty) = self.lhs.as_ref().unwrap().resolve_type() {
                    Some(Type::Ptr(Box::new(ty)))
                } else {
                    self.cty.clone()
                }
            }
            NodeType::Deref => self.lhs.as_ref().unwrap().dest_type(),
            _ => {
                if let Some(node) = self.lhs.as_ref() {
                    if let Some(ty) = node.resolve_type() {
                        return Some(ty);
                    }
                }
                if let Some(node) = self.rhs.as_ref() {
                    if let Some(ty) = node.resolve_type() {
                        return Some(ty);
                    }
                }
                if let Some(node) = self.then.as_ref() {
                    if let Some(ty) = node.resolve_type() {
                        return Some(ty);
                    }
                }
                None
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
    pub fn to_debug_string(&self, indent: usize) -> String {
        match self.nt {
            NodeType::LocalVar => {
                format!(
                    "LocalVar: {{ type: {:?}, offset: {} }}",
                    self.cty.as_ref().unwrap(),
                    self.offset.unwrap()
                )
            }
            NodeType::GlobalVar => {
                format!("GlobalVar: {{ name: {}{} }}", &self.global_name, &self.dest)
            }
            NodeType::Num => {
                format!("Num: {}", self.value.unwrap())
            }
            _ => [
                vec![format!("{:?}:", self.nt)],
                self.ini
                    .iter()
                    .map(|c| {
                        format!(
                            "{}ini->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.cond
                    .iter()
                    .map(|c| {
                        format!(
                            "{}cond->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.upd
                    .iter()
                    .map(|c| {
                        format!(
                            "{}upd->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.then
                    .iter()
                    .map(|c| {
                        format!(
                            "{}then->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.els
                    .iter()
                    .map(|c| {
                        format!(
                            "{}els->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.lhs
                    .iter()
                    .map(|c| {
                        format!(
                            "{}lhs->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.rhs
                    .iter()
                    .map(|c| {
                        format!(
                            "{}rhs->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.children
                    .iter()
                    .map(|c| {
                        format!(
                            "{}children->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
                self.args
                    .iter()
                    .map(|c| {
                        format!(
                            "{}args->{}",
                            " ".repeat(indent),
                            c.to_debug_string(indent + 2)
                        )
                    })
                    .collect::<Vec<String>>(),
            ]
            .concat()
            .join("\n"),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_debug_string(2))
    }
}
