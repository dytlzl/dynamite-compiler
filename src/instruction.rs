pub enum InstOperator {
    PUSH,
    POP,
    MOV,
    ADD,
    SUB,
    RET,
    CALL,
    CQO,
    JMP,
    JE,
    IMUL,
    IDIV,
    SETE,
    SETNE,
    SETL,
    SETLE,
    LEA,
    MOVSX,
    MOVSXD,
    MOVZX,
    CMP,
}

use InstOperator::*;
use std::fmt::{Display, Formatter, Debug, Error};

impl InstOperator {
    pub fn to_string(&self) -> &str {
        match self {
            PUSH => "push",
            POP => "pop",
            MOV => "mov",
            ADD => "add",
            SUB => "sub",
            RET => "ret",
            CALL => "call",
            CQO => "cqo",
            JMP => "jmp",
            JE => "je",
            IMUL => "imul",
            IDIV => "idiv",
            SETE => "sete",
            SETNE => "setne",
            SETL => "setl",
            SETLE => "setle",
            LEA => "lea",
            MOVSX => "movsx",
            MOVSXD => "movsxd",
            MOVZX => "movzx",
            CMP => "cmp"
        }
    }
    pub fn to_string_for_linux(&self) -> &str {
        match self {
            MOVZX => "movzb",
            _ => self.to_string()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RBP,
    RSP,
    EDI,
    DIL,
    AL,
    RIP,
}
use Register::*;

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(
            match self {
                RAX => "rax",
                RBX => "rbx",
                RCX => "rcx",
                RDX => "rdx",
                RSI => "rsi",
                RDI => "rdi",
                RBP => "rbp",
                RSP => "rsp",
                EDI => "edi",
                DIL => "dil",
                AL => "al",
                RIP => "rip",
            }
        )?;
        Ok(())
    }
}

pub enum InstOperand {
    Reg(Register),
    Num(usize),
    Label(String),
    Str(&'static str)
}

impl InstOperand {
    pub fn to_string(&self) -> String {
        match self {
            InstOperand::Str(s) => String::from(s.clone()),
            InstOperand::Num(i) => format!("{}", i),
            _ => String::new()
        }
    }
}

impl Into<InstOperand> for String {
    fn into(self) -> InstOperand {
        InstOperand::Label(self)
    }
}

impl Into<InstOperand> for &'static str {
    fn into(self) -> InstOperand {
        InstOperand::Str(self)
    }
}

impl Into<InstOperand> for usize {
    fn into(self) -> InstOperand {
        InstOperand::Num(self)
    }
}

impl Into<InstOperand> for Register {
    fn into(self) -> InstOperand {
        InstOperand::Reg(self)
    }
}

pub struct Instruction {
    pub operator: InstOperator,
    pub operand1: Option<InstOperand>,
    pub operand2: Option<InstOperand>,
}

impl Instruction {
    pub fn to_string(&self) -> String {
        let mut res = format!("  {}", self.operator.to_string());
        if let Some(_) = self.operand1 {
            res += " ";
            res += &self.operand1.as_ref().unwrap().to_string()[..]
        }
        if let Some(_) = self.operand2 {
            res += " ";
            res += &self.operand2.as_ref().unwrap().to_string()[..]
        }
        res
    }
}