#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    I8,
    I32,
    Ptr(Box<Type>),
    Arr(Box<Type>, usize),
    Func(Vec<Type>, Box<Type>)
}

impl Type {
    pub fn size_of(&self) -> usize {
        match self {
            Type::I8 => 1,
            Type::I32 => 4,
            Type::Ptr(_) => 8,
            Type::Arr(t, s) => t.size_of()*s,
            Type::Func(..) => 1,
        }
    }
    pub fn dest_type(&self) -> Option<Type> {
        match self {
            Type::Ptr(c) => Some(*c.clone()),
            Type::Arr(c, _) => Some(*c.clone()),
            _ => None
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::I32
    }
}