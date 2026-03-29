use super::error::Error;
use super::module::Registry;
use super::stmt::{ArithBinOp, CmpPredicate, Stmt};
use super::types::Dtype;
use super::value::{GlobalVariable, LocalVariable, Operand};
use indexmap::IndexMap;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum BlockLabel {
    BasicBlock(usize),
    Function(String),
}

impl Display for BlockLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockLabel::BasicBlock(index) => write!(f, "bb{}", index),
            BlockLabel::Function(identifier) => write!(f, "{}", identifier),
        }
    }
}

impl BlockLabel {
    pub fn key(&self) -> String {
        format!("{}", self)
    }
}

#[derive(Clone)]
pub struct BasicBlock {
    pub label: BlockLabel,
    pub stmts: Vec<Stmt>,
}

pub struct Function {
    pub identifier: String,
    pub local_variables: Option<IndexMap<String, LocalVariable>>,
    pub blocks: Option<Vec<BasicBlock>>,
    pub arguments: Vec<LocalVariable>,
    pub next_vreg: usize,
}

pub struct FunctionGenerator<'ir> {
    pub registry: &'ir Registry,
    pub global_variables: &'ir IndexMap<String, GlobalVariable>,
    pub local_variables: IndexMap<String, LocalVariable>,
    scope_locals: Vec<Vec<String>>,
    pub irs: Vec<Stmt>,
    pub arguments: Vec<LocalVariable>,
    pub next_vreg: usize,
    pub next_basic_block: usize,
}

impl<'ir> FunctionGenerator<'ir> {
    pub fn new(
        registry: &'ir Registry,
        global_variables: &'ir IndexMap<String, GlobalVariable>,
    ) -> Self {
        Self {
            registry,
            global_variables,
            local_variables: IndexMap::new(),
            scope_locals: Vec::new(),
            irs: Vec::new(),
            arguments: Vec::new(),
            next_vreg: 0,
            next_basic_block: 1,
        }
    }

    pub fn alloc_vreg(&mut self) -> usize {
        let idx = self.next_vreg;
        self.next_vreg += 1;
        idx
    }

    pub fn alloc_temporary(&mut self, dtype: Dtype) -> Operand {
        Operand::from(LocalVariable::new(dtype, self.alloc_vreg(), None))
    }

    pub fn alloc_basic_block(&mut self) -> BlockLabel {
        let idx = self.next_basic_block;
        self.next_basic_block += 1;
        BlockLabel::BasicBlock(idx)
    }

    pub fn lookup_variable(&self, id: &str) -> Result<Operand, Error> {
        if let Some(local) = self.local_variables.get(id) {
            Ok(Operand::from(local))
        } else if let Some(global) = self.global_variables.get(id) {
            Ok(Operand::Global(global.clone()))
        } else {
            Err(Error::VariableNotDefined {
                symbol: id.to_string(),
            })
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_locals.push(Vec::new());
    }

    pub fn exit_scope(&mut self) {
        if let Some(locals) = self.scope_locals.pop() {
            for id in locals {
                self.local_variables.shift_remove(&id);
            }
        }
    }

    pub fn record_scoped_local(&mut self, id: String) {
        if let Some(scope) = self.scope_locals.last_mut() {
            scope.push(id);
        }
    }
}

impl FunctionGenerator<'_> {
    pub fn emit_alloca(&mut self, dst: Operand) {
        self.irs.push(Stmt::as_alloca(dst));
    }

    pub fn emit_load(&mut self, dst: Operand, ptr: Operand) {
        self.irs.push(Stmt::as_load(dst, ptr));
    }

    pub fn emit_store(&mut self, src: Operand, ptr: Operand) {
        self.irs.push(Stmt::as_store(src, ptr));
    }

    pub fn emit_gep(&mut self, new_ptr: Operand, base_ptr: Operand, index: Operand) {
        self.irs.push(Stmt::as_gep(new_ptr, base_ptr, index));
    }
    pub fn emit_cast(&mut self, src: Operand, ty: Dtype, dst: Operand) {
        self.irs.push(Stmt::as_cast(src, ty, dst));
    }

    pub fn emit_biop(&mut self, op: ArithBinOp, left: Operand, right: Operand, dst: Operand) {
        self.irs.push(Stmt::as_biop(op, left, right, dst));
    }

    pub fn emit_cmp(&mut self, op: CmpPredicate, left: Operand, right: Operand, dst: Operand) {
        self.irs.push(Stmt::as_cmp(op, left, right, dst));
    }

    pub fn emit_cjump(&mut self, cond: Operand, true_label: BlockLabel, false_label: BlockLabel) {
        self.irs.push(Stmt::as_cjump(cond, true_label, false_label));
    }

    pub fn emit_jump(&mut self, target: BlockLabel) {
        self.irs.push(Stmt::as_jump(target));
    }

    pub fn emit_label(&mut self, label: BlockLabel) {
        self.irs.push(Stmt::as_label(label));
    }

    pub fn emit_call(&mut self, func_name: String, result: Option<Operand>, args: Vec<Operand>) {
        self.irs.push(Stmt::as_call(func_name, result, args));
    }

    pub fn emit_return(&mut self, val: Option<Operand>) {
        self.irs.push(Stmt::as_return(val));
    }
}
