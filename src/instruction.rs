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
    AND,
    OR,
    XOR,
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
            CMP => "cmp",
            AND => "and",
            OR => "or",
            XOR => "xor",
        }
    }
    pub fn to_string4linux(&self) -> &str {
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
    R8,
    R9,
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
                R8 => "r8",
                R9 => "r9",
            }
        )?;
        Ok(())
    }
}

pub enum InstOperand {
    Reg(Register),
    Num(usize),
    Label(String),
    Str(&'static str),
    Ptr(Register, usize),
    PtrAdd(Register, String),
    ElseFlag(usize),
    BeginFlag(usize),
    EndFlag(usize),
}

impl InstOperand {
    fn to_string(&self) -> String {
        match self {
            InstOperand::Reg(r) => format!("{}", r),
            InstOperand::Num(i) => format!("{}", i),
            InstOperand::Label(l) => l.clone(),
            InstOperand::Str(s) => String::from(s.clone()),
            InstOperand::Ptr(r, i) => {
                match i {
                    1 => format!("byte ptr[{}]", r),
                    4 => format!("dword ptr[{}]", r),
                    8 => format!("qword ptr[{}]", r),
                    _ => unreachable!()
                }
            },
            InstOperand::ElseFlag(i) =>  format!(".Lelse{}", i),
            InstOperand::BeginFlag(i) =>  format!(".Lbegin{}", i),
            InstOperand::EndFlag(i) =>  format!(".Lend{}", i),
            InstOperand::PtrAdd(s, r) => format!("[{} + {}]", s, r),

        }
    }
}

impl Display for InstOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&self.to_string())
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
    pub fn to_string4linux(&self) -> String {
        let mut res = format!("  {}", self.operator.to_string4linux());
        if let Some(_) = self.operand1 {
            res += " ";
            res += &self.operand1.as_ref().unwrap().to_string()[..]
        }
        if let Some(_) = self.operand2 {
            res += ", ";
            res += &self.operand2.as_ref().unwrap().to_string()[..]
        }
        res
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        let mut res = format!("  {}", self.operator.to_string());
        if let Some(_) = self.operand1 {
            res += " ";
            res += &self.operand1.as_ref().unwrap().to_string()[..]
        }
        if let Some(_) = self.operand2 {
            res += ", ";
            res += &self.operand2.as_ref().unwrap().to_string()[..]
        }
        res
    }
}

pub enum Assembly {
    Inst(Instruction),
    Other(String),
}

impl Assembly {
    pub fn to_string4linux(&self) -> String {
        match self {
            Assembly::Inst(i) => i.to_string4linux(),
            Assembly::Other(o) => o.clone(),
        }
    }
}

impl ToString for Assembly {
    fn to_string(&self) -> String {
        match self {
            Assembly::Inst(i) => i.to_string(),
            Assembly::Other(o) => o.clone(),
        }
    }
}