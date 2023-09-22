use crate::aarch64::instruction::{
    InstOperand,
    InstOperator::{self, *},
    Instruction,
    Register::*,
};
use crate::Os;

pub enum Assembly {
    Inst(Instruction),
    Other(String),
    Group(Vec<Assembly>),
}

impl From<Instruction> for Assembly {
    fn from(val: Instruction) -> Self {
        Assembly::Inst(val)
    }
}

impl From<&'static str> for Assembly {
    fn from(val: &'static str) -> Self {
        Assembly::Other(String::from(val))
    }
}

impl From<String> for Assembly {
    fn from(val: String) -> Self {
        Assembly::Other(val)
    }
}

impl From<Vec<Assembly>> for Assembly {
    fn from(val: Vec<Assembly>) -> Self {
        Assembly::Group(val)
    }
}

impl Assembly {
    fn is_empty(&self) -> bool {
        match self {
            Assembly::Group(v) => v.is_empty(),
            _ => false,
        }
    }
    pub fn inst0(operator: InstOperator) -> Assembly {
        Assembly::Inst(Instruction {
            operator,
            operand1: None,
            operand2: None,
            operand3: None,
        })
    }
    pub fn inst1<T1>(operator: InstOperator, operand1: T1) -> Assembly
    where
        T1: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            operand2: None,
            operand3: None,
        })
    }
    pub fn inst2<T1, T2>(operator: InstOperator, operand1: T1, operand2: T2) -> Assembly
    where
        T1: Into<InstOperand>,
        T2: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            operand2: Some(operand2.into()),
            operand3: None,
        })
    }
    pub fn inst3<T1, T2, T3>(
        operator: InstOperator,
        operand1: T1,
        operand2: T2,
        operand3: T3,
    ) -> Assembly
    where
        T1: Into<InstOperand>,
        T2: Into<InstOperand>,
        T3: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            operand2: Some(operand2.into()),
            operand3: Some(operand3.into()),
        })
    }
    pub fn reset_stack(stack_size: usize) -> Assembly {
        vec![
            Assembly::inst2(MOV, X15, X14),
            Assembly::inst2(SUB, X15, stack_size),
        ]
        .into()
    }
    pub fn epilogue() -> Assembly {
        Assembly::Group(vec![
            Assembly::inst2(MOV, X15, X14),
            Assembly::inst1(POP, X14),
            Assembly::inst0(RET),
        ])
    }
}

impl crate::generator::Assembly for Assembly {
    fn to_string(&self, target_os: Os) -> String {
        match self {
            Assembly::Inst(i) => i.to_string(target_os),
            Assembly::Other(o) => o.clone(),
            Assembly::Group(b) => b
                .iter()
                .filter(|a| !a.is_empty())
                .map(|a| a.to_string(target_os))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}
