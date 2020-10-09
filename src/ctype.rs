#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Int,
    Char,
    Ptr(Box<Type>),
    Arr(Box<Type>, usize)
}

impl Type {
    pub fn size_of(&self) -> usize {
        match self {
            Type::Int => 4,
            Type::Char => 1,
            Type::Ptr(_) => 8,
            Type::Arr(t, s) => t.size_of()*s,
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
        Self::Int
    }
}