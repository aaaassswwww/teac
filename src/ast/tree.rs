use super::decl::*;
use super::expr::*;
use super::program::*;
use super::stmt::*;
use std::fmt::{Error, Formatter};

pub trait DisplayAsTree {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error>;

    fn fmt_tree_root(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.fmt_tree(f, &[], true)
    }
}

fn tree_indent(indent_levels: &[bool], is_last: bool) -> String {
    let mut s = String::new();
    for &last in indent_levels {
        if last {
            s.push_str("   ");
        } else {
            s.push_str("|  ");
        }
    }
    if is_last {
        s.push_str("`--");
    } else {
        s.push_str("|--");
    }
    s
}

impl<T: DisplayAsTree + ?Sized> DisplayAsTree for Box<T> {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        (**self).fmt_tree(f, indent_levels, is_last)
    }
}

impl DisplayAsTree for Program {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}Program", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        for (i, elem) in self.elements.iter().enumerate() {
            elem.fmt_tree(f, &next, i + 1 == self.elements.len())?;
        }
        Ok(())
    }
}

impl DisplayAsTree for ProgramElement {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        match &self.inner {
            ProgramElementInner::VarDeclStmt(v) => v.fmt_tree(f, indent_levels, is_last),
            ProgramElementInner::StructDef(s) => s.fmt_tree(f, indent_levels, is_last),
            ProgramElementInner::FnDeclStmt(d) => d.fmt_tree(f, indent_levels, is_last),
            ProgramElementInner::FnDef(def) => def.fmt_tree(f, indent_levels, is_last),
        }
    }
}

impl DisplayAsTree for VarDeclStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}VarDeclStmt", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.inner.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for VarDeclStmtInner {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        match self {
            VarDeclStmtInner::Decl(v) => v.fmt_tree(f, indent_levels, is_last),
            VarDeclStmtInner::Def(d) => d.fmt_tree(f, indent_levels, is_last),
        }
    }
}

impl DisplayAsTree for VarDecl {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        let ty = self
            .type_specifier
            .as_ref()
            .map_or("unknown".to_string(), |ts| ts.to_string());
        writeln!(f, "{}{}: {}", tree_indent(indent_levels, is_last), self.identifier, ty)
    }
}

impl DisplayAsTree for VarDef {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}VarDef {}", tree_indent(indent_levels, is_last), self.identifier)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            VarDefInner::Scalar(s) => s.val.fmt_tree(f, &next, true),
            VarDefInner::Array(a) => match &a.initializer {
                ArrayInitializer::ExplicitList(vals) => {
                    for (i, v) in vals.iter().enumerate() {
                        v.fmt_tree(f, &next, i + 1 == vals.len())?;
                    }
                    Ok(())
                }
                ArrayInitializer::Fill { val, count } => {
                    writeln!(f, "{}fill x{}", tree_indent(&next, false), count)?;
                    val.fmt_tree(f, &next, true)
                }
            },
        }
    }
}

impl DisplayAsTree for Vec<VarDecl> {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}VarDeclList", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        for (i, decl) in self.iter().enumerate() {
            decl.fmt_tree(f, &next, i + 1 == self.len())?;
        }
        Ok(())
    }
}

impl DisplayAsTree for FnDeclStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        self.fn_decl.fmt_tree(f, indent_levels, is_last)
    }
}

impl DisplayAsTree for FnDecl {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}FnDecl {}", tree_indent(indent_levels, is_last), self.identifier)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        if let Some(params) = &self.param_decl {
            for (i, decl) in params.decls.iter().enumerate() {
                decl.fmt_tree(f, &next, i + 1 == params.decls.len() && self.return_dtype.is_none())?;
            }
        }
        if let Some(ret) = &self.return_dtype {
            writeln!(f, "{}return: {}", tree_indent(&next, true), ret)?;
        }
        Ok(())
    }
}

impl DisplayAsTree for FnDef {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}FnDef {}", tree_indent(indent_levels, is_last), self.fn_decl.identifier)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.fn_decl.fmt_tree(f, &next, self.stmts.is_empty())?;
        for (i, stmt) in self.stmts.iter().enumerate() {
            stmt.fmt_tree(f, &next, i + 1 == self.stmts.len())?;
        }
        Ok(())
    }
}

impl DisplayAsTree for StructDef {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}StructDef {}", tree_indent(indent_levels, is_last), self.identifier)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.decls.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for CodeBlockStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        self.inner.fmt_tree(f, indent_levels, is_last)
    }
}

impl DisplayAsTree for CodeBlockStmtInner {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        match self {
            CodeBlockStmtInner::VarDecl(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::Assignment(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::Call(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::If(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::While(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::Return(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::Continue(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::Break(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
            CodeBlockStmtInner::Null(stmt) => stmt.fmt_tree(f, indent_levels, is_last),
        }
    }
}

impl DisplayAsTree for AssignmentStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}AssignmentStmt", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.left_val.fmt_tree(f, &next, false)?;
        self.right_val.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for CallStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}CallStmt {}", tree_indent(indent_levels, is_last), self.fn_call.qualified_name())?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        for (i, val) in self.fn_call.vals.iter().enumerate() {
            val.fmt_tree(f, &next, i + 1 == self.fn_call.vals.len())?;
        }
        Ok(())
    }
}

impl DisplayAsTree for IfStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}IfStmt", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.bool_unit.fmt_tree(f, &next, false)?;
        for stmt in &self.if_stmts {
            stmt.fmt_tree(f, &next, false)?;
        }
        if let Some(else_stmts) = &self.else_stmts {
            for (i, stmt) in else_stmts.iter().enumerate() {
                stmt.fmt_tree(f, &next, i + 1 == else_stmts.len())?;
            }
        }
        Ok(())
    }
}

impl DisplayAsTree for WhileStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}WhileStmt", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.bool_unit.fmt_tree(f, &next, false)?;
        for (i, stmt) in self.stmts.iter().enumerate() {
            stmt.fmt_tree(f, &next, i + 1 == self.stmts.len())?;
        }
        Ok(())
    }
}

impl DisplayAsTree for ReturnStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ReturnStmt", tree_indent(indent_levels, is_last))?;
        if let Some(v) = &self.val {
            let mut next = indent_levels.to_vec();
            next.push(is_last);
            v.fmt_tree(f, &next, true)?;
        }
        Ok(())
    }
}

impl DisplayAsTree for ContinueStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ContinueStmt", tree_indent(indent_levels, is_last))
    }
}

impl DisplayAsTree for BreakStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}BreakStmt", tree_indent(indent_levels, is_last))
    }
}

impl DisplayAsTree for NullStmt {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}NullStmt", tree_indent(indent_levels, is_last))
    }
}

impl DisplayAsTree for LeftVal {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}LeftVal", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            LeftValInner::Id(id) => writeln!(f, "{}Id({})", tree_indent(&next, true), id),
            LeftValInner::ArrayExpr(expr) => expr.fmt_tree(f, &next, true),
            LeftValInner::MemberExpr(expr) => expr.fmt_tree(f, &next, true),
        }
    }
}

impl DisplayAsTree for RightVal {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}RightVal", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            RightValInner::ArithExpr(expr) => expr.fmt_tree(f, &next, true),
            RightValInner::BoolExpr(expr) => expr.fmt_tree(f, &next, true),
        }
    }
}

impl DisplayAsTree for ArrayExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ArrayExpr", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.arr.fmt_tree(f, &next, false)?;
        self.idx.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for MemberExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}MemberExpr {}", tree_indent(indent_levels, is_last), self.member_id)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.struct_id.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for IndexExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        match &self.inner {
            IndexExprInner::Num(n) => writeln!(f, "{}IndexExpr Num({})", tree_indent(indent_levels, is_last), n),
            IndexExprInner::Id(id) => writeln!(f, "{}IndexExpr Id({})", tree_indent(indent_levels, is_last), id),
        }
    }
}

impl DisplayAsTree for ArithExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ArithExpr", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            ArithExprInner::ArithBiOpExpr(expr) => expr.fmt_tree(f, &next, true),
            ArithExprInner::ExprUnit(unit) => unit.fmt_tree(f, &next, true),
        }
    }
}

impl DisplayAsTree for ArithBiOpExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ArithBiOpExpr {:?}", tree_indent(indent_levels, is_last), self.op)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.left.fmt_tree(f, &next, false)?;
        self.right.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for CastExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}CastExpr", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.expr.fmt_tree(f, &next, false)?;
        writeln!(f, "{}target: {}", tree_indent(&next, true), self.target)
    }
}

impl DisplayAsTree for ExprUnit {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ExprUnit", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            ExprUnitInner::Num(n) => writeln!(f, "{}Num({})", tree_indent(&next, true), n),
            ExprUnitInner::Float(v) => writeln!(f, "{}Float({})", tree_indent(&next, true), v),
            ExprUnitInner::Id(id) => writeln!(f, "{}Id({})", tree_indent(&next, true), id),
            ExprUnitInner::ArithExpr(expr) => expr.fmt_tree(f, &next, true),
            ExprUnitInner::Cast(expr) => expr.fmt_tree(f, &next, true),
            ExprUnitInner::FnCall(call) => call.fmt_tree(f, &next, true),
            ExprUnitInner::ArrayExpr(expr) => expr.fmt_tree(f, &next, true),
            ExprUnitInner::MemberExpr(expr) => expr.fmt_tree(f, &next, true),
            ExprUnitInner::Reference(id) => writeln!(f, "{}Ref({})", tree_indent(&next, true), id),
        }
    }
}

impl DisplayAsTree for BoolExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}BoolExpr", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            BoolExprInner::BoolBiOpExpr(expr) => expr.fmt_tree(f, &next, true),
            BoolExprInner::BoolUnit(unit) => unit.fmt_tree(f, &next, true),
        }
    }
}

impl DisplayAsTree for BoolBiOpExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}BoolBiOpExpr {:?}", tree_indent(indent_levels, is_last), self.op)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.left.fmt_tree(f, &next, false)?;
        self.right.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for BoolUnit {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}BoolUnit", tree_indent(indent_levels, is_last))?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        match &self.inner {
            BoolUnitInner::ComExpr(expr) => expr.fmt_tree(f, &next, true),
            BoolUnitInner::BoolExpr(expr) => expr.fmt_tree(f, &next, true),
            BoolUnitInner::BoolUOpExpr(expr) => expr.fmt_tree(f, &next, true),
        }
    }
}

impl DisplayAsTree for ComExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}ComExpr {:?}", tree_indent(indent_levels, is_last), self.op)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.left.fmt_tree(f, &next, false)?;
        self.right.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for BoolUOpExpr {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}BoolUOpExpr {:?}", tree_indent(indent_levels, is_last), self.op)?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        self.cond.fmt_tree(f, &next, true)
    }
}

impl DisplayAsTree for FnCall {
    fn fmt_tree(
        &self,
        f: &mut Formatter<'_>,
        indent_levels: &[bool],
        is_last: bool,
    ) -> Result<(), Error> {
        writeln!(f, "{}FnCall {}", tree_indent(indent_levels, is_last), self.qualified_name())?;
        let mut next = indent_levels.to_vec();
        next.push(is_last);
        for (i, val) in self.vals.iter().enumerate() {
            val.fmt_tree(f, &next, i + 1 == self.vals.len())?;
        }
        Ok(())
    }
}
