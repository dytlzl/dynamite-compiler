#[derive(Debug, Clone)]
pub enum TokenType {
    Reserved,
    Ident,
    Num,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tt: TokenType,
    pub pos: usize,
    pub i_value: usize,
    pub s_value: String,
}

impl Token {
    pub fn print(&self) {
        match self.tt {
            TokenType::Num => {
                eprintln!("num: {}", self.i_value)
            }
            TokenType::Reserved => {
                eprintln!("rsv: {}", &self.s_value)
            }
            TokenType::Eof => {}
            TokenType::Ident => {
                eprintln!("idt: {}", &self.s_value)
            }
        }
    }
}