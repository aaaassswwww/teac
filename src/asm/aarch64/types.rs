use crate::asm::error::Error;
use crate::ir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Virtual(usize),
    Physical(u8),
    StackPointer,
}

/// The hardware register class a virtual or physical register belongs to.
///
/// aarch64 keeps integer (`Gpr` — `w_`/`x_`) and floating-point/SIMD
/// (`Fpr` — `s_`/`d_`/`v_`) register banks completely independent: no
/// instruction can simultaneously source one operand from each bank.
/// The register allocator uses this class to split vregs into two
/// interference graphs that are coloured against disjoint pools.  The
/// enum is consumed only by asmt-4's solution; at the asmt-4 skeleton
/// stage no code constructs the variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum RegClass {
    /// General-purpose integer (`x0`–`x30`, `sp`).
    Gpr,
    /// Floating-point / SIMD (`s0`–`s31`, alternatively `d`/`v`).
    Fpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegSize {
    /// 32-bit general-purpose: `w0`–`w30`, also used for `i1` operands.
    W32,
    /// 64-bit general-purpose: `x0`–`x30`, also used for pointer-typed
    /// operands.
    X64,
    /// 32-bit single-precision floating-point: `s0`–`s31`.
    #[allow(dead_code)]
    S32,
}

impl RegSize {
    /// The register class implied by this width.  `W32`/`X64` live in
    /// the general-purpose bank; `S32` is a floating-point register.
    /// Used by asmt-4's register allocator to bucket vregs.
    #[allow(dead_code)]
    pub fn class(&self) -> RegClass {
        match self {
            RegSize::W32 | RegSize::X64 => RegClass::Gpr,
            RegSize::S32 => RegClass::Fpr,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    SDiv,
}

/// Single-precision floating-point binary operators corresponding 1:1
/// to the aarch64 `fadd`/`fsub`/`fmul`/`fdiv` instructions.  The
/// variants are unused at the asmt-4 skeleton stage; asmt-4's solution
/// produces them from the IR's `FBiOpStmt`.  The `F` prefix mirrors the
/// aarch64 mnemonic family and is preserved deliberately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum FBinOp {
    FAdd,
    FSub,
    FMul,
    FDiv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Addr {
    BaseOff { base: Register, offset: i64 },
    Global(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexOperand {
    Reg(Register),
    Imm(i64),
}

pub fn dtype_to_regsize(dtype: &ir::Dtype) -> Result<RegSize, Error> {
    match dtype {
        ir::Dtype::I1 | ir::Dtype::I32 => Ok(RegSize::W32),
        ir::Dtype::Pointer { .. } => Ok(RegSize::X64),
        _ => Err(Error::UnsupportedDtype {
            dtype: dtype.clone(),
        }),
    }
}
