use crate::Os;

pub enum InstOperator {
    PUSH,
    POP,
    MOV,
    ADD,
    SUB,
    RET,
    BL,
    SVC,
    CQO,
    JMP,
    JE,
    JNE,
    IMUL,
    IDIV,
    SHL,
    SAR,
    NOT,
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

use std::fmt::{Debug, Display, Error, Formatter};
use InstOperator::*;

impl InstOperator {
    pub fn to_string(&self, target_os: Os) -> &str {
        if matches!(target_os, Os::Linux) && matches!(self, MOVZX) {
            return "movzb";
        }
        match self {
            PUSH => "push",
            POP => "pop",
            MOV => "mov",
            ADD => "add",
            SUB => "sub",
            RET => "ret",
            BL => "bl",
            CQO => "cqo",
            SVC => "svc",
            JMP => "jmp",
            JE => "je",
            JNE => "jne",
            IMUL => "imul",
            IDIV => "idiv",
            SHL => "shl",
            SAR => "sar",
            NOT => "not",
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
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    AL,
    CL,
    RIP,
    R8,
    R9,
}
use Register::*;

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(match self {
            X8 => "x8",
            X9 => "x9",
            X10 => "x10",
            X11 => "x11",
            X12 => "x12",
            X13 => "x13",
            X14 => "x14",
            X15 => "x15",
            X16 => "x16",
            X17 => "dil",
            AL => "al",
            CL => "cl",
            RIP => "rip",
            R8 => "r8",
            R9 => "r9",
        })?;
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

impl Display for InstOperand {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&match self {
            InstOperand::Reg(r) => format!("{}", r),
            InstOperand::Num(i) => format!("#{}", i),
            InstOperand::Label(l) => l.clone(),
            InstOperand::Str(s) => String::from(*s),
            InstOperand::Ptr(r, _) => format!("[{}]", r),
            InstOperand::ElseFlag(i) => format!(".Lelse{}", i),
            InstOperand::BeginFlag(i) => format!(".Lbegin{}", i),
            InstOperand::EndFlag(i) => format!(".Lend{}", i),
            InstOperand::PtrAdd(s, r) => format!("[{} + {}]", s, r),
        })
    }
}

impl From<String> for InstOperand {
    fn from(val: String) -> Self {
        InstOperand::Label(val)
    }
}

impl From<&'static str> for InstOperand {
    fn from(val: &'static str) -> Self {
        InstOperand::Str(val)
    }
}

impl From<usize> for InstOperand {
    fn from(val: usize) -> Self {
        InstOperand::Num(val)
    }
}

impl From<Register> for InstOperand {
    fn from(val: Register) -> Self {
        InstOperand::Reg(val)
    }
}

pub struct Instruction {
    pub operator: InstOperator,
    pub operand1: Option<InstOperand>,
    pub operand2: Option<InstOperand>,
    pub operand3: Option<InstOperand>,
}
impl Instruction {
    pub fn to_string(&self, target_os: Os) -> String {
        if self.operand3.is_some() {
            return format!(
                "  {} {}, {}, {}",
                self.operator.to_string(target_os),
                &self.operand1.as_ref().unwrap().to_string()[..],
                &self.operand2.as_ref().unwrap().to_string()[..],
                &self.operand3.as_ref().unwrap().to_string()[..]
            );
        }
        if self.operand2.is_some() {
            return format!(
                "  {} {}, {}",
                self.operator.to_string(target_os),
                &self.operand1.as_ref().unwrap().to_string()[..],
                &self.operand2.as_ref().unwrap().to_string()[..]
            );
        }
        if self.operand1.is_some() {
            return format!(
                "  {} {}",
                self.operator.to_string(target_os),
                &self.operand1.as_ref().unwrap().to_string()[..],
            );
        }
        format!("  {}", self.operator.to_string(target_os))
    }
}
