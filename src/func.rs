use crate::ctype::Type;
use crate::node::Node;
use crate::token::Token;

#[derive(Default)]
pub struct Func {
    pub body: Option<Node>,
    pub cty: Type,
    pub offset_size: usize,
    pub token: Option<Token>,
    pub args: Vec<Node>,
}

impl Func {
    pub fn print(&self) {
        eprintln!("args:");
        self.args.iter().for_each(|arg| arg.print(0));
        eprintln!("body:");
        self.body.iter().for_each(|n| n.print(0));
    }
}
