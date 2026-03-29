use crate::ast;
use crate::ir::types::Dtype;
use crate::ir::value::Named;

fn base_dtype(type_specifier: &Option<ast::TypeSpecifier>) -> Dtype {
    match type_specifier.as_ref().map(|t| &t.inner) {
        Some(ast::TypeSpecifierInner::Composite(name)) => Dtype::Struct {
            type_name: name.to_string(),
        },
        Some(ast::TypeSpecifierInner::Reference(inner)) => Dtype::ptr_to(Dtype::Array {
            element: Box::new(base_dtype(&Some(inner.as_ref().clone()))),
            length: None,
        }),
        Some(ast::TypeSpecifierInner::BuiltIn(_)) | None => Dtype::I32,
        _ => todo!()
    }
}

impl Named for ast::VarDecl {
    fn identifier(&self) -> Option<String> {
        Some(self.identifier.clone())
    }
}

impl Named for ast::VarDef {
    fn identifier(&self) -> Option<String> {
        Some(self.identifier.clone())
    }
}

impl Named for ast::VarDeclStmt {
    fn identifier(&self) -> Option<String> {
        match &self.inner {
            ast::VarDeclStmtInner::Decl(d) => Some(d.identifier.clone()),
            ast::VarDeclStmtInner::Def(d) => Some(d.identifier.clone()),
        }
    }
}

impl From<ast::TypeSpecifier> for Dtype {
    fn from(a: ast::TypeSpecifier) -> Self {
        Self::from(&a)
    }
}

impl From<&ast::TypeSpecifier> for Dtype {
    fn from(a: &ast::TypeSpecifier) -> Self {
        match &a.inner {
            ast::TypeSpecifierInner::BuiltIn(_) => Self::I32,
            ast::TypeSpecifierInner::Composite(name) => Self::Struct {
                type_name: name.to_string(),
            },
            ast::TypeSpecifierInner::Reference(inner) => Self::ptr_to(Dtype::Array {
                element: Box::new(Self::from(inner.as_ref())),
                length: None,
            }),
            _ => todo!()
        }
    }
}

impl TryFrom<&ast::VarDecl> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(decl: &ast::VarDecl) -> Result<Self, Self::Error> {
        let base_dtype = base_dtype(&decl.type_specifier);
        match &decl.inner {
            ast::VarDeclInner::Array(decl) => Ok(Dtype::array_of(base_dtype, decl.len)),
            ast::VarDeclInner::Scalar => Ok(base_dtype),
        }
    }
}

impl TryFrom<&ast::VarDef> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(def: &ast::VarDef) -> Result<Self, Self::Error> {
        if let Dtype::Struct { .. } = &base_dtype(&def.type_specifier) {
            return Err(crate::ir::Error::StructInitialization);
        }
        let base_dtype = base_dtype(&def.type_specifier);
        match &def.inner {
            ast::VarDefInner::Array(def) => Ok(Dtype::array_of(base_dtype, def.len)),
            ast::VarDefInner::Scalar(_) => Ok(base_dtype),
        }
    }
}

impl TryFrom<&ast::VarDeclStmt> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(value: &ast::VarDeclStmt) -> Result<Self, Self::Error> {
        match &value.inner {
            ast::VarDeclStmtInner::Decl(d) => Dtype::try_from(d.as_ref()),
            ast::VarDeclStmtInner::Def(d) => Dtype::try_from(d.as_ref()),
        }
    }
}
