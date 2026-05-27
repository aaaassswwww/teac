use std::collections::{HashMap, HashSet};

use super::types::{Addr, BinOp, Cond, FBinOp, IndexOperand, Operand, RegSize, Register};
use crate::common::graph::CfgNode;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Inst {
    Label(String),

    Mov {
        size: RegSize,
        dst: Register,
        src: Operand,
    },

    BinOp {
        op: BinOp,
        size: RegSize,
        dst: Register,
        lhs: Register,
        rhs: Operand,
    },

    /// Single-precision floating-point binary operation, e.g.
    /// `fadd s_d, s_n, s_m`.  Both operands and the destination live in
    /// the Fpr bank; the result is always 32-bit (`RegSize::S32`).
    FBinOp {
        op: FBinOp,
        dst: Register,
        lhs: Register,
        rhs: Register,
    },

    Ldr {
        size: RegSize,
        dst: Register,
        addr: Addr,
    },

    Str {
        size: RegSize,
        src: Register,
        addr: Addr,
    },

    Lea {
        dst: Register,
        addr: Addr,
    },

    Gep {
        dst: Register,
        base: Register,
        index: IndexOperand,
        scale: i64,
    },

    Cmp {
        size: RegSize,
        lhs: Register,
        rhs: Operand,
    },

    /// Single-precision floating-point comparison `fcmp s_n, s_m`.
    /// Sets NZCV, which is later consumed by a `B { Cond, label }` arm.
    FCmp {
        lhs: Register,
        rhs: Register,
    },

    /// `scvtf s_d, w_n` — convert a signed 32-bit integer to a
    /// single-precision float.
    Scvtf {
        dst: Register,
        src: Register,
    },

    /// `fcvtzs w_d, s_n` — convert a single-precision float to a signed
    /// 32-bit integer, rounding toward zero.
    Fcvtzs {
        dst: Register,
        src: Register,
    },

    /// `fmov s_d, s_n` (Fpr-to-Fpr) or `fmov s_d, w_n` (Gpr-to-Fpr).
    /// Used in phi lowering, in the AAPCS64 entry/exit shims for `f32`
    /// arguments and return values, and to materialise a float constant
    /// from a literal `Operand::Immediate`.
    Fmov {
        dst: Register,
        src: Operand,
    },

    B {
        label: String,
    },
    BCond {
        cond: Cond,
        label: String,
    },
    Bl {
        func: String,
    },

    SaveCallerRegs,
    RestoreCallerRegs,

    SubSp {
        imm: i64,
    },
    AddSp {
        imm: i64,
    },

    Ret,
}

impl CfgNode for Inst {
    fn label(&self) -> Option<String> {
        if let Inst::Label(name) = self {
            Some(name.clone())
        } else {
            None
        }
    }

    fn successors(
        &self,
        idx: usize,
        num_nodes: usize,
        label_map: &HashMap<String, usize>,
    ) -> Vec<usize> {
        match self {
            Inst::Ret => vec![],
            Inst::B { label } => label_map.get(label.as_str()).copied().into_iter().collect(),
            Inst::BCond { label, .. } => {
                let mut succs = Vec::with_capacity(2);
                if let Some(&target) = label_map.get(label.as_str()) {
                    succs.push(target);
                }
                if idx + 1 < num_nodes {
                    succs.push(idx + 1);
                }
                succs
            }
            _ => {
                if idx + 1 < num_nodes {
                    vec![idx + 1]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl Inst {
    pub fn used_vregs(&self) -> HashSet<usize> {
        let mut used = HashSet::new();

        let add_reg = |s: &mut HashSet<usize>, r: &Register| {
            if let Register::Virtual(v) = r {
                s.insert(*v);
            }
        };

        let add_operand = |s: &mut HashSet<usize>, op: &Operand| {
            if let Operand::Register(Register::Virtual(v)) = op {
                s.insert(*v);
            }
        };

        let add_addr = |s: &mut HashSet<usize>, addr: &Addr| {
            if let Addr::BaseOff {
                base: Register::Virtual(v),
                ..
            } = addr
            {
                s.insert(*v);
            }
        };

        match self {
            Inst::Mov { src, .. } => add_operand(&mut used, src),
            Inst::BinOp { lhs, rhs, .. } => {
                add_reg(&mut used, lhs);
                add_operand(&mut used, rhs);
            }
            Inst::FBinOp { lhs, rhs, .. } => {
                add_reg(&mut used, lhs);
                add_reg(&mut used, rhs);
            }
            Inst::Ldr { addr, .. } => add_addr(&mut used, addr),
            Inst::Str { src, addr, .. } => {
                add_reg(&mut used, src);
                add_addr(&mut used, addr);
            }
            Inst::Lea { addr, .. } => add_addr(&mut used, addr),
            Inst::Gep { base, index, .. } => {
                add_reg(&mut used, base);
                if let IndexOperand::Reg(r) = index {
                    add_reg(&mut used, r);
                }
            }
            Inst::Cmp { lhs, rhs, .. } => {
                add_reg(&mut used, lhs);
                add_operand(&mut used, rhs);
            }
            Inst::FCmp { lhs, rhs } => {
                add_reg(&mut used, lhs);
                add_reg(&mut used, rhs);
            }
            Inst::Scvtf { src, .. } | Inst::Fcvtzs { src, .. } => add_reg(&mut used, src),
            Inst::Fmov { src, .. } => add_operand(&mut used, src),
            Inst::Label(_)
            | Inst::B { .. }
            | Inst::BCond { .. }
            | Inst::Bl { .. }
            | Inst::SaveCallerRegs
            | Inst::RestoreCallerRegs
            | Inst::SubSp { .. }
            | Inst::AddSp { .. }
            | Inst::Ret => {}
        }
        used
    }

    pub fn defined_vregs(&self) -> HashSet<usize> {
        let mut defined = HashSet::new();
        match self {
            Inst::Mov { dst, .. }
            | Inst::BinOp { dst, .. }
            | Inst::FBinOp { dst, .. }
            | Inst::Ldr { dst, .. }
            | Inst::Lea { dst, .. }
            | Inst::Gep { dst, .. }
            | Inst::Scvtf { dst, .. }
            | Inst::Fcvtzs { dst, .. }
            | Inst::Fmov { dst, .. } => {
                if let Register::Virtual(v) = dst {
                    defined.insert(*v);
                }
            }
            _ => {}
        }
        defined
    }
}
