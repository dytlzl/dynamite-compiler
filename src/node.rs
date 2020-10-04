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
    Ret,
    If,
    Whl,
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
            value,
            ..Self::default()
        }
    }
    pub fn new_with_ident(token: Option<Token>, nt: NodeType, offset: usize) -> Self {
        Self {
            token,
            nt,
            offset,
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
            nt: NodeType::Whl,
            cond: Some(Box::new(cond)),
            then: Some(Box::new(then)),
            ..Self::default()
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
            NodeType::If => {
                format!("{:?}({}, {}, {})",
                        self.nt,
                        self.cond.as_ref().map(|n| n.format()).unwrap(),
                        self.then.as_ref().map(|n| n.format()).unwrap(),
                        if let Some(_) = self.els {
                            self.els.as_ref().map(|n| n.format()).unwrap()
                        } else {
                            String::from("None")
                        })
            }
            NodeType::Whl => {
                format!("{:?}({}, {})",
                        self.nt,
                        self.cond.as_ref().map(|n| n.format()).unwrap(),
                        self.then.as_ref().map(|n| n.format()).unwrap())
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