mod function_generator;
mod inst;
mod phi_lowering;
mod printer;
mod register_allocator;
mod types;

pub use inst::Inst;
pub use types::{Addr, BinOp, Cond, Operand, Register};

use crate::asm::common::{StackFrame, StructLayouts};
use crate::asm::error::Error;
use crate::common::{Generator, Target};
use crate::ir;
use function_generator::FunctionGenerator;
use printer::{AsmPrint, AsmPrinter};
use register_allocator::rewrite_insts;
use std::collections::HashMap;
use std::io::Write;
use types::dtype_to_regsize;

fn debug_longcode_enabled() -> bool {
    std::env::var_os("TEAC_DEBUG_LONGCODE2").is_some()
}

struct GeneratedGlobal {
    symbol: String,
    data: GlobalData,
}

enum GlobalData {
    Word { value: i64 },
    Array { words: Vec<i64>, zero_bytes: i64 },
}

struct GeneratedFunction {
    symbol: String,
    frame_size: i64,
    insts: Vec<Inst>,
}

pub struct AArch64AsmGenerator<'a> {
    module: &'a ir::Module,
    registry: &'a ir::Registry,
    target: Target,
    globals: Vec<GeneratedGlobal>,
    functions: Vec<GeneratedFunction>,
}

impl<'a> AArch64AsmGenerator<'a> {
    pub fn new(module: &'a ir::Module, registry: &'a ir::Registry, target: Target) -> Self {
        Self {
            module,
            registry,
            target,
            globals: Vec::new(),
            functions: Vec::new(),
        }
    }
}

impl<'a> Generator for AArch64AsmGenerator<'a> {
    type Error = Error;

    fn generate(&mut self) -> Result<(), Error> {
        if debug_longcode_enabled() {
            eprintln!(
                "[dbg] asm::generate structs={} globals={} funcs={}",
                self.registry.struct_types.len(),
                self.module.global_list.len(),
                self.module.function_list.len()
            );
        }
        let layouts = StructLayouts::from_struct_types(&self.registry.struct_types)?;
        if debug_longcode_enabled() {
            eprintln!("[dbg] asm::layouts ready");
        }

        self.globals.clear();
        for (name, def) in &self.module.global_list {
            if debug_longcode_enabled() {
                eprintln!("[dbg] asm::handle_global {} dtype={}", name, def.dtype);
            }
            self.globals
                .push(Self::handle_global(&layouts, name, def, self.target)?);
        }

        self.functions.clear();
        for func in self.module.function_list.values() {
            // Skip external declarations; they are provided by the linked
            // object file (e.g. std.o) and must not be emitted as assembly
            // symbols, otherwise the linker will report duplicate definitions.
            let Some(body) = func.body.as_ref() else {
                continue;
            };
            if debug_longcode_enabled() {
                eprintln!(
                    "[dbg] asm::handle_function {} blocks={} next_vreg={}",
                    func.link_name,
                    body.blocks.len(),
                    body.next_vreg
                );
            }
            self.functions.push(Self::handle_function(
                &layouts,
                &func.link_name,
                body,
                self.target,
            )?);
        }

        Ok(())
    }

    fn output<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        let mut printer = AsmPrinter::new(w, self.target);

        if !self.globals.is_empty() {
            printer.emit_section("data")?;
            for g in &self.globals {
                printer.emit_global(&g.symbol)?;
                printer.emit_align(2)?;
                printer.emit_label(&g.symbol)?;
                match &g.data {
                    GlobalData::Word { value } => printer.emit_word(*value)?,
                    GlobalData::Array { words, zero_bytes } => {
                        for v in words {
                            printer.emit_word(*v)?;
                        }
                        if *zero_bytes > 0 {
                            printer.emit_zero(*zero_bytes)?;
                        }
                    }
                }
            }
            printer.emit_newline()?;
        }

        printer.emit_section("text")?;
        for func in &self.functions {
            printer.emit_global(&func.symbol)?;
            printer.emit_align(2)?;
            printer.emit_label(&func.symbol)?;
            printer.emit_prologue(func.frame_size)?;
            printer.emit_insts(&func.insts)?;
            printer.emit_newline()?;
        }

        Ok(())
    }
}

impl<'a> AArch64AsmGenerator<'a> {
    fn handle_arguments(body: &ir::FunctionBody) -> Result<Vec<Inst>, Error> {
        let mut insts = Vec::new();

        for (i, arg) in body.arguments.iter().enumerate() {
            let v = arg.id.0;
            let size = dtype_to_regsize(&arg.dtype)?;

            if i < 8 {
                insts.push(Inst::Mov {
                    size,
                    dst: Register::Virtual(v),
                    src: Operand::Register(Register::Physical(i as u8)),
                });
            } else {
                // Stack arguments (9th onward) are above the saved fp/lr pair.
                // Stack layout after prologue:
                //   [fp+16]: arg 8, [fp+24]: arg 9, ...
                //   [fp+8]:  saved lr
                //   [fp]:    saved fp (frame pointer points here)
                let offset = 16 + ((i - 8) as i64) * 8;
                insts.push(Inst::Ldr {
                    size,
                    dst: Register::Virtual(v),
                    addr: Addr::BaseOff {
                        base: Register::Physical(29),
                        offset,
                    },
                });
            }
        }

        Ok(insts)
    }

    fn handle_global(
        layouts: &StructLayouts,
        name: &str,
        def: &ir::GlobalDef,
        target: Target,
    ) -> Result<GeneratedGlobal, Error> {
        let symbol = target.mangle_symbol(name);

        let data = match &def.dtype {
            ir::Dtype::I32 => {
                let value = def
                    .initializers
                    .as_ref()
                    .and_then(|v| v.first())
                    .copied()
                    .map(|v| v as i64)
                    .unwrap_or(0);
                GlobalData::Word { value }
            }
            ir::Dtype::Array { element, length } => {
                let len = length.expect("unsized array in global data");
                let (elem_size, _) = layouts.size_align_of(element.as_ref())?;

                if let Some(inits) = &def.initializers {
                    let words: Vec<i64> = inits.iter().take(len).map(|&v| v as i64).collect();
                    let remaining = len.saturating_sub(inits.len());
                    let zero_bytes = (remaining as i64) * elem_size;
                    GlobalData::Array { words, zero_bytes }
                } else {
                    let zero_bytes = (len as i64) * elem_size;
                    GlobalData::Array {
                        words: Vec::new(),
                        zero_bytes,
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedDtype {
                    dtype: def.dtype.clone(),
                })
            }
        };

        Ok(GeneratedGlobal { symbol, data })
    }

    fn handle_function(
        layouts: &StructLayouts,
        link_name: &str,
        body: &ir::FunctionBody,
        target: Target,
    ) -> Result<GeneratedFunction, Error> {
        if debug_longcode_enabled() {
            eprintln!("[dbg] asm::handle_function enter {}", link_name);
        }
        let symbol = target.mangle_symbol(link_name);
        let mut frame = StackFrame::from_blocks(&body.blocks, layouts)?;
        if debug_longcode_enabled() {
            eprintln!("[dbg] asm::stack frame built {}", link_name);
        }
        let mut next_vreg = body.next_vreg;
        let mut cond_map: HashMap<usize, Cond> = HashMap::new();
        let mut insts: Vec<Inst> = Vec::new();
        insts.extend(Self::handle_arguments(body)?);

        {
            let mut ctx = FunctionGenerator {
                func_id: &symbol,
                frame: &frame,
                layouts,
                target,
                insts: &mut insts,
                next_vreg: &mut next_vreg,
                cond_map: &mut cond_map,
            };
            if debug_longcode_enabled() {
                eprintln!("[dbg] asm::phi lowering {}", link_name);
            }
            phi_lowering::lower_function_blocks(&mut ctx, &body.blocks)?;
        }

        let alloc = register_allocator::allocate(&insts);
        if debug_longcode_enabled() {
            eprintln!(
                "[dbg] asm::register allocation {} spilled={}",
                link_name,
                alloc.spilled.len()
            );
        }
        for v in alloc.spilled.iter().copied() {
            frame.alloc_spill(v, 8, 8);
        }
        let insts = rewrite_insts(&insts, &alloc, &frame)?;
        if debug_longcode_enabled() {
            eprintln!("[dbg] asm::handle_function done {}", link_name);
        }

        Ok(GeneratedFunction {
            symbol,
            frame_size: frame.frame_size_aligned(),
            insts,
        })
    }
}
