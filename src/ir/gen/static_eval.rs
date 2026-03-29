use crate::ast;
use crate::ir::module::IrGenerator;
use crate::ir::Error;

impl IrGenerator<'_> {
    pub fn handle_right_val_static(r: &ast::RightVal) -> Result<i32, Error> {
        match &r.inner {
            ast::RightValInner::ArithExpr(expr) => Self::handle_arith_expr_static(expr),
            ast::RightValInner::BoolExpr(expr) => Self::handle_bool_expr_static(expr),
        }
    }

    pub fn handle_cast_expr_static(expr: &ast::expr::CastExpr) -> Result<i32, Error> {
        match &expr.inner {
            ast::expr::CastExprInner::CastOpExpr(expr) => Self::handle_cast_op_expr_static(expr),
            ast::expr::CastExprInner::ExprUnit(unit) => Self::handle_expr_unit_static(unit),
        }
    }

    pub fn handle_arith_expr_static(expr: &ast::ArithExpr) -> Result<i32, Error> {
        match &expr.inner {
            ast::ArithExprInner::ArithBiOpExpr(expr) => Self::handle_arith_biop_expr_static(expr),
            ast::ArithExprInner::CastExpr(cast) => Self::handle_cast_expr_static(cast),
        }
    }

    pub fn handle_bool_expr_static(expr: &ast::BoolExpr) -> Result<i32, Error> {
        match &expr.inner {
            ast::BoolExprInner::BoolBiOpExpr(expr) => Self::handle_bool_biop_expr_static(expr),
            ast::BoolExprInner::BoolUnit(unit) => Self::handle_bool_unit_static(unit),
        }
    }

    pub fn handle_cast_op_expr_static(expr: &ast::expr::CastOpExpr) -> Result<i32, Error> {
        let src = Self::handle_expr_unit_static(&expr.expr)?;
        let target_ts = expr.type_specifier.as_ref().expect("cast missing type");

        match &target_ts.inner {
            ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Int) => {
                Ok(src)
            },

            _ => Err(Error::InvalidCast),
            
        }

    }
    
    pub fn handle_arith_biop_expr_static(expr: &ast::ArithBiOpExpr) -> Result<i32, Error> {
        let left = Self::handle_arith_expr_static(&expr.left)?;
        let right = Self::handle_arith_expr_static(&expr.right)?;
        match &expr.op {
            ast::ArithBiOp::Add => left.checked_add(right).ok_or(Error::IntegerOverflow),
            ast::ArithBiOp::Sub => left.checked_sub(right).ok_or(Error::IntegerOverflow),
            ast::ArithBiOp::Mul => left.checked_mul(right).ok_or(Error::IntegerOverflow),
            ast::ArithBiOp::Div => left.checked_div(right).ok_or(Error::DivisionByZero),
        }
    }

    pub fn handle_expr_unit_static(expr: &ast::ExprUnit) -> Result<i32, Error> {
        match &expr.inner {
            ast::ExprUnitInner::Num(num) => Ok(*num),
            ast::ExprUnitInner::ArithExpr(expr) => Self::handle_arith_expr_static(expr),
            _ => Err(Error::InvalidExprUnit {
                expr_unit: expr.clone(),
            }),
        }
    }

    pub fn handle_bool_biop_expr_static(expr: &ast::BoolBiOpExpr) -> Result<i32, Error> {
        let left = Self::handle_bool_expr_static(&expr.left)? != 0;
        let right = Self::handle_bool_expr_static(&expr.right)? != 0;
        if expr.op == ast::BoolBiOp::And {
            Ok((left && right) as i32)
        } else {
            Ok((left || right) as i32)
        }
    }

    pub fn handle_bool_unit_static(unit: &ast::BoolUnit) -> Result<i32, Error> {
        match &unit.inner {
            ast::BoolUnitInner::ComExpr(expr) => Self::handle_com_op_expr_static(expr),
            ast::BoolUnitInner::BoolExpr(expr) => Self::handle_bool_expr_static(expr),
            ast::BoolUnitInner::BoolUOpExpr(expr) => Self::handle_bool_uop_expr_static(expr),
        }
    }

    pub fn handle_com_op_expr_static(expr: &ast::ComExpr) -> Result<i32, Error> {
        let left = Self::handle_expr_unit_static(&expr.left)?;
        let right = Self::handle_expr_unit_static(&expr.right)?;
        match expr.op {
            ast::ComOp::Lt => Ok((left < right) as i32),
            ast::ComOp::Eq => Ok((left == right) as i32),
            ast::ComOp::Ge => Ok((left >= right) as i32),
            ast::ComOp::Gt => Ok((left > right) as i32),
            ast::ComOp::Le => Ok((left <= right) as i32),
            ast::ComOp::Ne => Ok((left != right) as i32),
        }
    }

    pub fn handle_bool_uop_expr_static(expr: &ast::BoolUOpExpr) -> Result<i32, Error> {
        if expr.op == ast::BoolUOp::Not {
            Ok((Self::handle_bool_unit_static(&expr.cond)? == 0) as i32)
        } else {
            Ok(0)
        }
    }
}
