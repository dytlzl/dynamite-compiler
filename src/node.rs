use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Asg,
    LVar,
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
    Ret
}

pub struct Node {
    pub nt: NodeType,
    pub token: Option<Token>,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub value: usize,
    pub offset: usize,
}

impl Node {
    pub fn new_with_op(token: Option<Token>, nt: NodeType, lhs: Node, rhs: Node) -> Self {
        Self {
            token,
            nt,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            value: 0,
            offset: 0,
        }
    }
    pub fn new_with_op_and_lhs(token: Option<Token>, nt: NodeType, lhs: Node) -> Self {
        Self {
            token,
            nt,
            lhs: Some(Box::new(lhs)),
            rhs: None,
            value: 0,
            offset: 0,
        }
    }
    pub fn new_with_num(token: Option<Token>, value: usize) -> Self {
        Self {
            token,
            nt: NodeType::Num,
            lhs: None,
            rhs: None,
            value,
            offset: 0,
        }
    }
    pub fn new_with_ident(token: Option<Token>, nt: NodeType, offset: usize) -> Self {
        Self {
            token,
            nt,
            lhs: None,
            rhs: None,
            value: 0,
            offset,
        }
    }
    pub fn format(&self) -> String {
        match self.nt {
            NodeType::Num => {
                format!("{}", self.value)
            }
            NodeType::LVar => {
                format!("{:?}", self.nt)
            }
            NodeType::Ret => {
                format!("{:?}({})",
                        self.nt,
                        self.lhs.as_ref().map(|n| n.format()).unwrap())
            }
            _ => {
                format!("{:?}({}, {})",
                        self.nt,
                        self.lhs.as_ref().map(|n| n.format()).unwrap(),
                        self.rhs.as_ref().map(|n| n.format()).unwrap())
            }
        }
    }
}