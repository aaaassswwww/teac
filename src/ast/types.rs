pub type Pos = usize;

#[derive(Debug, Clone)]
pub enum BuiltIn {
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub enum TypeSpecifierInner {
    BuiltIn(BuiltIn),
    Composite(String),
    Reference(Box<TypeSpecifier>),
    Array(Box<TypeSpecifier>, i32),
}

#[derive(Debug, Clone)]
pub struct TypeSpecifier {
    pub pos: Pos,
    pub inner: TypeSpecifierInner,
}
