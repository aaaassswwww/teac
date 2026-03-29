use crate::ast;

use super::function::BlockLabel;
use super::types::Dtype;
use super::value::Operand;
use std::fmt::{self, Display, Formatter, write};

#[derive(Clone)]
pub enum ArithBinOp {
    Add,
    Sub,
    Mul,
    SDiv,
}

impl From<ast::ArithBiOp> for ArithBinOp {
    fn from(value: ast::ArithBiOp) -> Self {
        match value {
            ast::ArithBiOp::Add => ArithBinOp::Add,
            ast::ArithBiOp::Sub => ArithBinOp::Sub,
            ast::ArithBiOp::Mul => ArithBinOp::Mul,
            ast::ArithBiOp::Div => ArithBinOp::SDiv,
        }
    }
}

impl Display for ArithBinOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ArithBinOp::Add => write!(f, "add"),
            ArithBinOp::Sub => write!(f, "sub"),
            ArithBinOp::Mul => write!(f, "mul"),
            ArithBinOp::SDiv => write!(f, "sdiv"),
        }
    }
}

#[derive(Clone)]
pub enum CmpPredicate {
    Eq,
    Ne,
    Sgt,
    Sge,
    Slt,
    Sle,
}

impl From<ast::ComOp> for CmpPredicate {
    fn from(value: ast::ComOp) -> Self {
        match value {
            ast::ComOp::Eq => CmpPredicate::Eq,
            ast::ComOp::Ne => CmpPredicate::Ne,
            ast::ComOp::Gt => CmpPredicate::Sgt,
            ast::ComOp::Ge => CmpPredicate::Sge,
            ast::ComOp::Lt => CmpPredicate::Slt,
            ast::ComOp::Le => CmpPredicate::Sle,
        }
    }
}

impl Display for CmpPredicate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CmpPredicate::Eq => write!(f, "eq"),
            CmpPredicate::Ne => write!(f, "ne"),
            CmpPredicate::Sgt => write!(f, "sgt"),
            CmpPredicate::Sge => write!(f, "sge"),
            CmpPredicate::Slt => write!(f, "slt"),
            CmpPredicate::Sle => write!(f, "sle"),
        }
    }
}

#[derive(Clone)]
pub enum StmtInner {
    Call(CallStmt),
    Load(LoadStmt),
    Phi(PhiStmt),
    Cast(CastStmt),
    BiOp(BiOpStmt),
    Alloca(AllocaStmt),
    Cmp(CmpStmt),
    CJump(CJumpStmt),
    Label(LabelStmt),
    Store(StoreStmt),
    Jump(JumpStmt),
    Gep(GepStmt),
    Return(ReturnStmt),
}

#[derive(Clone)]
pub struct Stmt {
    pub inner: StmtInner,
}

/// Describes how an operand is used within a statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandRole {
    /// Instruction defines (writes) this value.
    Def,
    /// General use (read) of a value.
    Use,
    /// Pointer operand of a `load` instruction.
    LoadPtr,
    /// Pointer operand of a `store` instruction.
    StorePtr,
}

/// Borrowed reference to an operand together with its role.
pub struct OperandRef<'a> {
    pub operand: &'a Operand,
    pub role: OperandRole,
}

impl Stmt {
    pub fn as_call(func_name: String, res: Option<Operand>, args: Vec<Operand>) -> Self {
        Self {
            inner: StmtInner::Call(CallStmt {
                func_name,
                res,
                args,
            }),
        }
    }

    pub fn as_load(dst: Operand, ptr: Operand) -> Self {
        Self {
            inner: StmtInner::Load(LoadStmt { dst, ptr }),
        }
    }

    pub fn as_phi(dst: Operand, incomings: Vec<(BlockLabel, Operand)>) -> Self {
        Self {
            inner: StmtInner::Phi(PhiStmt { dst, incomings }),
        }
    }

    pub fn as_cast(src: Operand, ty: Dtype, dst: Operand) -> Self {
        Self {
            inner: StmtInner::Cast(CastStmt { src, ty, dst }),
        }
    }

    pub fn as_biop(kind: ArithBinOp, left: Operand, right: Operand, dst: Operand) -> Self {
        Self {
            inner: StmtInner::BiOp(BiOpStmt {
                kind,
                left,
                right,
                dst,
            }),
        }
    }

    pub fn as_alloca(dst: Operand) -> Self {
        Self {
            inner: StmtInner::Alloca(AllocaStmt { dst }),
        }
    }

    pub fn as_cmp(kind: CmpPredicate, left: Operand, right: Operand, dst: Operand) -> Self {
        Self {
            inner: StmtInner::Cmp(CmpStmt {
                kind,
                left,
                right,
                dst,
            }),
        }
    }

    pub fn as_cjump(dst: Operand, true_label: BlockLabel, false_label: BlockLabel) -> Self {
        Self {
            inner: StmtInner::CJump(CJumpStmt {
                dst,
                true_label,
                false_label,
            }),
        }
    }

    pub fn as_label(label: BlockLabel) -> Self {
        Self {
            inner: StmtInner::Label(LabelStmt { label }),
        }
    }

    pub fn as_store(src: Operand, ptr: Operand) -> Self {
        Self {
            inner: StmtInner::Store(StoreStmt { src, ptr }),
        }
    }

    pub fn as_jump(target: BlockLabel) -> Self {
        Self {
            inner: StmtInner::Jump(JumpStmt { target }),
        }
    }

    pub fn as_return(val: Option<Operand>) -> Self {
        Self {
            inner: StmtInner::Return(ReturnStmt { val }),
        }
    }

    pub fn as_gep(new_ptr: Operand, base_ptr: Operand, index: Operand) -> Self {
        Self {
            inner: StmtInner::Gep(GepStmt {
                new_ptr,
                base_ptr,
                index,
            }),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            StmtInner::Alloca(s) => write!(f, "\t{}", s),
            StmtInner::BiOp(s) => write!(f, "\t{}", s),
            StmtInner::Cast(s) => write!(f, "\t{}", s),
            StmtInner::CJump(s) => write!(f, "\t{}", s),
            StmtInner::Call(s) => write!(f, "\t{}", s),
            StmtInner::Cmp(s) => write!(f, "\t{}", s),
            StmtInner::Gep(s) => write!(f, "\t{}", s),
            StmtInner::Label(s) => write!(f, "{}", s),
            StmtInner::Load(s) => write!(f, "\t{}", s),
            StmtInner::Phi(s) => write!(f, "\t{}", s),
            StmtInner::Return(s) => write!(f, "\t{}", s),
            StmtInner::Store(s) => write!(f, "\t{}", s),
            StmtInner::Jump(s) => write!(f, "\t{}", s),
            _ => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct CallStmt {
    pub func_name: String,
    pub res: Option<Operand>,
    pub args: Vec<Operand>,
}

#[derive(Clone)]
pub struct LoadStmt {
    pub dst: Operand,
    pub ptr: Operand,
}

#[derive(Clone)]
pub struct PhiStmt {
    pub dst: Operand,
    pub incomings: Vec<(BlockLabel, Operand)>,
}

#[derive(Clone)]
pub struct CastStmt {
    pub src: Operand,   // 源操作数
    pub ty: Dtype,      // 目标类型
    pub dst: Operand,   // 存放转换结果的临时变量
}

#[derive(Clone)]
pub struct BiOpStmt {
    pub kind: ArithBinOp,
    pub left: Operand,
    pub right: Operand,
    pub dst: Operand,
}

#[derive(Clone)]
pub struct AllocaStmt {
    pub dst: Operand,
}

#[derive(Clone)]
pub struct CmpStmt {
    pub kind: CmpPredicate,
    pub left: Operand,
    pub right: Operand,
    pub dst: Operand,
}

#[derive(Clone)]
pub struct CJumpStmt {
    pub dst: Operand,
    pub true_label: BlockLabel,
    pub false_label: BlockLabel,
}

#[derive(Clone)]
pub struct LabelStmt {
    pub label: BlockLabel,
}

#[derive(Clone)]
pub struct StoreStmt {
    pub src: Operand,
    pub ptr: Operand,
}

#[derive(Clone)]
pub struct JumpStmt {
    pub target: BlockLabel,
}

#[derive(Clone)]
pub struct GepStmt {
    pub new_ptr: Operand,
    pub base_ptr: Operand,
    pub index: Operand,
}

#[derive(Clone)]
pub struct ReturnStmt {
    pub val: Option<Operand>,
}

impl Display for CallStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|a| {
                if matches!(a.dtype(), Dtype::Pointer { .. } | Dtype::Array { .. }) {
                    format!("ptr {}", a)
                } else {
                    format!("{} {}", a.dtype(), a)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        if let Some(res) = &self.res {
            write!(
                f,
                "{} = call {} @{}({})",
                res,
                &res.dtype(),
                self.func_name,
                args
            )
        } else {
            write!(f, "call void @{}({})", self.func_name, args)
        }
    }
}

impl Display for LoadStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = load {}, ptr {}, align 4",
            self.dst,
            self.dst.dtype(),
            self.ptr
        )
    }
}

impl Display for PhiStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let dtype = self.dst.dtype();
        let incoming_str = self
            .incomings
            .iter()
            .map(|(label, val)| format!("[ {}, %{} ]", val, label))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{} = phi {} {}", self.dst, dtype, incoming_str)
    }
}

impl Display for StoreStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "store {} {}, ptr {}, align 4",
            self.src.dtype(),
            self.src,
            self.ptr
        )
    }
}

impl Display for AllocaStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.dst.dtype() {
            Dtype::Pointer { pointee } => write!(f, "{} = alloca {}, align 4", self.dst, pointee),
            _ => write!(f, "{} = alloca {}, align 4", self.dst, self.dst.dtype()),
        }
    }
}


impl Display for CastStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = cast({}, {})",
            self.dst,
            self.src,
            self.ty,
        )
    }
}

impl Display for BiOpStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {} {} {}, {}",
            self.dst,
            self.kind,
            self.dst.dtype(),
            self.left,
            self.right
        )
    }
}

impl Display for CmpStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = icmp {} {} {}, {}",
            self.dst,
            self.kind,
            self.left.dtype(),
            self.left,
            self.right
        )
    }
}

impl Display for CJumpStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "br i1 {}, label %{}, label %{}",
            self.dst, &self.true_label, &self.false_label
        )
    }
}

impl Display for JumpStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "br label %{}", &self.target)
    }
}

impl Display for LabelStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.label)
    }
}

impl Display for GepStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.base_ptr.dtype() {
            Dtype::Pointer { pointee } => match pointee.as_ref() {
                Dtype::Array {
                    length: Some(_), ..
                }
                | Dtype::Struct { .. } => write!(
                    f,
                    "{} = getelementptr {}, ptr {}, i32 {}, i32 {}",
                    self.new_ptr, pointee, self.base_ptr, 0, self.index,
                ),
                Dtype::Array {
                    element,
                    length: None,
                } => write!(
                    f,
                    "{} = getelementptr {}, ptr {}, i32 {}",
                    self.new_ptr, element, self.base_ptr, self.index,
                ),
                _ => write!(
                    f,
                    "{} = getelementptr {}, ptr {}, i32 {}",
                    self.new_ptr, pointee, self.base_ptr, self.index,
                ),
            },
            Dtype::Array { .. } => write!(
                f,
                "{} = getelementptr {}, ptr {}, i32 {}, i32 {}",
                self.new_ptr,
                self.base_ptr.dtype(),
                self.base_ptr,
                0,
                self.index,
            ),
            _ => Err(fmt::Error),
        }
    }
}

impl Display for ReturnStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.val {
            Some(v) => write!(f, "ret {} {}", v.dtype(), v),
            None => write!(f, "ret void"),
        }
    }
}

impl Stmt {
    pub fn operands(&self) -> Vec<OperandRef<'_>> {
        use OperandRole::*;
        let mut ops = Vec::new();
        match &self.inner {
            StmtInner::Alloca(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Def,
                });
            }
            StmtInner::Load(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Def,
                });
                ops.push(OperandRef {
                    operand: &s.ptr,
                    role: LoadPtr,
                });
            }
            StmtInner::Store(s) => {
                ops.push(OperandRef {
                    operand: &s.src,
                    role: Use,
                });
                ops.push(OperandRef {
                    operand: &s.ptr,
                    role: StorePtr,
                });
            }
            StmtInner::Cast(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Def,
                });
                ops.push(OperandRef {
                    operand: &s.src,
                    role: Use,
                });
            }
            StmtInner::BiOp(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Def,
                });
                ops.push(OperandRef {
                    operand: &s.left,
                    role: Use,
                });
                ops.push(OperandRef {
                    operand: &s.right,
                    role: Use,
                });
            }
            StmtInner::Cmp(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Def,
                });
                ops.push(OperandRef {
                    operand: &s.left,
                    role: Use,
                });
                ops.push(OperandRef {
                    operand: &s.right,
                    role: Use,
                });
            }
            StmtInner::CJump(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Use,
                });
            }
            StmtInner::Call(s) => {
                if let Some(res) = &s.res {
                    ops.push(OperandRef {
                        operand: res,
                        role: Def,
                    });
                }
                for arg in &s.args {
                    ops.push(OperandRef {
                        operand: arg,
                        role: Use,
                    });
                }
            }
            StmtInner::Gep(s) => {
                ops.push(OperandRef {
                    operand: &s.new_ptr,
                    role: Def,
                });
                ops.push(OperandRef {
                    operand: &s.base_ptr,
                    role: Use,
                });
                ops.push(OperandRef {
                    operand: &s.index,
                    role: Use,
                });
            }
            StmtInner::Return(s) => {
                if let Some(val) = &s.val {
                    ops.push(OperandRef {
                        operand: val,
                        role: Use,
                    });
                }
            }
            StmtInner::Phi(s) => {
                ops.push(OperandRef {
                    operand: &s.dst,
                    role: Def,
                });
                for (_, val) in &s.incomings {
                    ops.push(OperandRef {
                        operand: val,
                        role: Use,
                    });
                }
            }
            StmtInner::Label(_) | StmtInner::Jump(_) => {}
        }
        ops
    }

    pub fn map_use_operands(&self, f: impl Fn(&Operand) -> Operand) -> Stmt {
        match &self.inner {
            StmtInner::Alloca(s) => Stmt::as_alloca(s.dst.clone()),
            StmtInner::Load(s) => Stmt::as_load(s.dst.clone(), f(&s.ptr)),
            StmtInner::Store(s) => Stmt::as_store(f(&s.src), f(&s.ptr)),
            StmtInner::BiOp(s) => {
                Stmt::as_biop(s.kind.clone(), f(&s.left), f(&s.right), s.dst.clone())
            }
            StmtInner::Cmp(s) => {
                Stmt::as_cmp(s.kind.clone(), f(&s.left), f(&s.right), s.dst.clone())
            }
            StmtInner::CJump(s) => {
                Stmt::as_cjump(f(&s.dst), s.true_label.clone(), s.false_label.clone())
            }
            StmtInner::Call(s) => {
                let args = s.args.iter().map(&f).collect();
                Stmt::as_call(s.func_name.clone(), s.res.clone(), args)
            }
            StmtInner::Gep(s) => Stmt::as_gep(s.new_ptr.clone(), f(&s.base_ptr), f(&s.index)),
            StmtInner::Return(s) => Stmt::as_return(s.val.as_ref().map(&f)),
            StmtInner::Phi(s) => {
                let incomings = s
                    .incomings
                    .iter()
                    .map(|(label, val)| (label.clone(), f(val)))
                    .collect();
                Stmt::as_phi(s.dst.clone(), incomings)
            }
            StmtInner::Label(s) => Stmt::as_label(s.label.clone()),
            StmtInner::Jump(s) => Stmt::as_jump(s.target.clone()),
            _ => todo!(),
        }
    }
}
