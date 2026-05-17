use super::expr::*;
use super::ops::*;
use super::program::Program;
use super::tree::DisplayAsTree;
use super::types::*;
use std::fmt::{Display, Error, Formatter};

impl Display for BuiltIn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            BuiltIn::Int => write!(f, "int"),
            BuiltIn::Float => write!(f, "int"),
        }
    }
}

impl Display for TypeSpecifierInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            TypeSpecifierInner::BuiltIn(b) => write!(f, "{}", b),
            TypeSpecifierInner::Composite(name) => write!(f, "{}", name),
            TypeSpecifierInner::Reference(inner) => write!(f, "&[{}]", inner.inner),
            TypeSpecifierInner::Array(inner, a) => write!(f, "[{}; {}]", inner.inner, a),
        }
    }
}

impl Display for TypeSpecifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}@{}", self.inner, self.pos)
    }
}

impl Display for ArithBiOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ArithBiOp::Add => write!(f, "add"),
            ArithBiOp::Sub => write!(f, "sub"),
            ArithBiOp::Mul => write!(f, "mul"),
            ArithBiOp::Div => write!(f, "sdiv"),
        }
    }
}

impl Display for BoolUOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            BoolUOp::Not => write!(f, "!"),
        }
    }
}

impl Display for BoolBiOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let op = match self {
            BoolBiOp::And => "&&",
            BoolBiOp::Or => "||",
        };
        write!(f, "{}", op)
    }
}

impl Display for ComOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ComOp::Eq => write!(f, "eq"),
            ComOp::Ne => write!(f, "ne"),
            ComOp::Gt => write!(f, "sgt"),
            ComOp::Ge => write!(f, "sge"),
            ComOp::Lt => write!(f, "slt"),
            ComOp::Le => write!(f, "sle"),
        }
    }
}

// 为 CastOpExpr 实现 Display
impl Display for CastOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // 格式: (expr as target_type)
        let type_str = self
            .type_specifier
            .as_ref()
            .map_or("unknown".to_string(), |ts| ts.to_string());
        write!(f, "({} as {})", self.expr, type_str)
    }
}

// 为 CastExprInner 实现 Display
impl Display for CastExprInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            CastExprInner::CastOpExpr(op) => write!(f, "{}", op),
            CastExprInner::ExprUnit(unit) => write!(f, "{}", unit),
        }
    }
}

// 为 CastExpr 实现 Display
impl Display for CastExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for ArithBiOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

impl Display for ArithExprInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ArithExprInner::ArithBiOpExpr(expr) => write!(f, "{}", expr),
            ArithExprInner::CastExpr(cast) => write!(f, "{}", cast),
        }
    }
}

impl Display for ArithExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for ComExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

impl Display for BoolUOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({}{})", self.op, self.cond)
    }
}

impl Display for BoolBiOpExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

impl Display for BoolExprInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            BoolExprInner::BoolUnit(b) => write!(f, "{}", b),
            BoolExprInner::BoolBiOpExpr(b) => write!(f, "{}", b),
        }
    }
}

impl Display for BoolExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for BoolUnitInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            BoolUnitInner::ComExpr(c) => write!(f, "{}", c),
            BoolUnitInner::BoolExpr(b) => write!(f, "{}", b),
            BoolUnitInner::BoolUOpExpr(u) => write!(f, "{}", u),
        }
    }
}

impl Display for BoolUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for RightValInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            RightValInner::ArithExpr(a) => write!(f, "{}", a),
            RightValInner::BoolExpr(b) => write!(f, "{}", b),
        }
    }
}

impl Display for RightVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for LeftValInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            LeftValInner::Id(id) => write!(f, "{}", id),
            LeftValInner::ArrayExpr(ae) => write!(f, "{}", ae),
            LeftValInner::MemberExpr(me) => write!(f, "{}", me),
        }
    }
}

impl Display for LeftVal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for IndexExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.inner {
            IndexExprInner::Num(n) => write!(f, "{}", n),
            IndexExprInner::Id(id) => write!(f, "{}", id),
        }
    }
}

impl Display for ArrayExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}[{}]", self.arr, self.idx)
    }
}

impl Display for MemberExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}.{}", self.struct_id, self.member_id)
    }
}

impl Display for FnCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let args: Vec<String> = self.vals.iter().map(|v| format!("{}", v)).collect();
        if let Some(module) = &self.module_prefix {
            write!(f, "{}::{}({})", module, self.name, args.join(", "))
        } else {
            write!(f, "{}({})", self.name, args.join(", "))
        }
    }
}

impl Display for ExprUnitInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            ExprUnitInner::Num(n) => write!(f, "{}", n),
            ExprUnitInner::Float(fl) => write!(f, "{}", fl),
            ExprUnitInner::Id(id) => write!(f, "{}", id),
            ExprUnitInner::ArithExpr(a) => write!(f, "{}", a),
            ExprUnitInner::FnCall(fc) => write!(f, "{}", fc),
            ExprUnitInner::ArrayExpr(ae) => write!(f, "{}", ae),
            ExprUnitInner::MemberExpr(me) => write!(f, "{}", me),
            ExprUnitInner::Reference(id) => write!(f, "&{}", id),
        }
    }
}

impl Display for ExprUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.fmt_tree_root(f)
    }
}
