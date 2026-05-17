//! Conversions between AST types and IR types.
//!
//! This module provides trait implementations and helper functions to convert
//! AST-level type representations (`ast::TypeSpecifier`, `ast::VarDecl`,
//! `ast::VarDef`, `ast::VarDeclStmt`) into their corresponding IR-level
//! data types (`Dtype`).

use crate::ast;
use crate::ir::types::Dtype;

/// Converts an optional AST type specifier into the corresponding base IR data type (`Dtype`).
///
/// Delegates to `Dtype::from(&TypeSpecifier)` when a specifier is present;
/// defaults to `Dtype::I32` when absent.
///
/// This function is used for **global variables** and **function parameters** where
/// an absent type annotation defaults to `i32`. Local variables are handled by the
/// separate type inference pass (`type_infer::infer_function`), which resolves their
/// types before IR generation.
fn base_dtype(type_specifier: Option<&ast::TypeSpecifier>) -> Dtype {
    type_specifier.map_or(Dtype::I32, Dtype::from)
}

/// Combines a scalar element type with an AST declaration shape (scalar vs. array)
/// to produce the storage [`Dtype`] for that declaration.
pub(crate) fn compose_var_decl_dtype(base: Dtype, inner: &ast::VarDeclInner) -> Dtype {
    match inner {
        ast::VarDeclInner::Scalar => base,
        ast::VarDeclInner::Array(arr) => Dtype::array_of(base, arr.len),
    }
}

/// Analogue of [`compose_var_decl_dtype`] for [`ast::VarDef`]s.
pub(crate) fn compose_var_def_dtype(base: Dtype, inner: &ast::VarDefInner) -> Dtype {
    match inner {
        ast::VarDefInner::Scalar(_) => base,
        ast::VarDefInner::Array(arr) => Dtype::array_of(base, arr.len),
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
            ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Int) => Self::I32,
            ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Float) => Self::F32,
            ast::TypeSpecifierInner::Composite(name) => Self::Struct {
                type_name: name.clone(),
            },
            ast::TypeSpecifierInner::Reference(inner) => Self::ptr_to(Dtype::Array {
                element: Box::new(Self::from(inner.as_ref())),
                length: None,
            }),
            ast::TypeSpecifierInner::Array(inner, len) => {
                Self::array_of(Self::from(inner.as_ref()), *len as usize)
            }
        }
    }
}

impl TryFrom<&ast::VarDecl> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(decl: &ast::VarDecl) -> Result<Self, Self::Error> {
        let base = base_dtype(decl.type_specifier.as_ref());
        Ok(compose_var_decl_dtype(base, &decl.inner))
    }
}

impl TryFrom<&ast::VarDef> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(def: &ast::VarDef) -> Result<Self, Self::Error> {
        let base = base_dtype(def.type_specifier.as_ref());
        if matches!(&base, Dtype::Struct { .. }) {
            return Err(crate::ir::Error::StructInitialization);
        }
        Ok(compose_var_def_dtype(base, &def.inner))
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
