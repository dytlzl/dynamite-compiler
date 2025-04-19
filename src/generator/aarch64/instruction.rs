use crate::generator::Os;

#[derive(Default)]
pub enum InstOperator {
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
    MUL,
    SDIV,
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
    LDR,
    STR,
    ADRP,
    STP,
    LDP,
    MSUB,
    #[default]
    NOP,
}

use InstOperator::*;
use std::fmt::{Debug, Display, Error, Formatter};

impl InstOperator {
    pub fn to_string(&self, target_os: Os) -> &str {
        if matches!(target_os, Os::Linux) && matches!(self, MOVZX) {
            return "movzb";
        }
        match self {
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
            MUL => "mul",
            SDIV => "sdiv",
            MSUB => "msub",
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
            LDR => "ldr",
            STR => "str",
            ADRP => "adrp",
            STP => "stp",
            LDP => "ldp",
            NOP => "nop",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
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
    X29,
    X30,
    AL,
    CL,
    RIP,
    R8,
    R9,
    SP,
}
use Register::*;

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(match self {
            X0 => "x0",
            X1 => "x1",
            X2 => "x2",
            X3 => "x3",
            X4 => "x4",
            X5 => "x5",
            X6 => "x6",
            X7 => "x7",
            X8 => "x8",
            X9 => "x9",
            X10 => "x10",
            X11 => "x11",
            X12 => "x12",
            X13 => "x13",
            X14 => "x14",
            X15 => "x15",
            X16 => "x16",
            X17 => "x17",
            X29 => "x29",
            X30 => "x30",
            AL => "al",
            CL => "cl",
            RIP => "rip",
            R8 => "r8",
            R9 => "r9",
            SP => "sp",
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
            InstOperand::PtrAdd(s, r) => format!("[{}, {}]", s, r),
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
#[derive(Default)]
pub struct Instruction {
    pub operator: InstOperator,
    pub operand1: Option<InstOperand>,
    pub operand2: Option<InstOperand>,
    pub operand3: Option<InstOperand>,
    pub operand4: Option<InstOperand>,
}
impl Instruction {
    pub fn to_string(&self, target_os: Os) -> String {
        if self.operand4.is_some() {
            return format!(
                "  {} {}, {}, {}, {}",
                self.operator.to_string(target_os),
                &self.operand1.as_ref().unwrap().to_string()[..],
                &self.operand2.as_ref().unwrap().to_string()[..],
                &self.operand3.as_ref().unwrap().to_string()[..],
                &self.operand4.as_ref().unwrap().to_string()[..],
            );
        }
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
