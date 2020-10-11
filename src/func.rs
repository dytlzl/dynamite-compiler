use crate::ctype::Type;
use crate::node::Node;
use crate::token::Token;

#[derive(Default)]
pub struct Func {
    pub body: Option<Node>,
    pub arg_types: Vec<Type>,
    pub return_type: Type,
    pub offset_size: usize,
    pub token: Option<Token>,
    pub args: Vec<Node>,
}

impl Func {
    pub fn print(&self) {
        eprintln!("args:");
        for arg in &self.args {
            eprintln!("{:?}", arg);
        }
        eprintln!("body:");
        if let Some(n) = &self.body {
            n.print(0);
        }
    }
}