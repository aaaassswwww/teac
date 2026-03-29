use crate::ast;

use super::common::{get_pos, grammar_error, parse_num, Pair, ParseResult, Rule};
use super::ParseContext;

impl<'a> ParseContext<'a> {
    pub(crate) fn parse_use_stmt(&self, pair: Pair) -> ParseResult<ast::UseStmt> {
        let parts: Vec<&str> = pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::identifier)
            .map(|p| p.as_str())
            .collect();
        Ok(ast::UseStmt {
            module_name: parts.join("::"),
        })
    }

    pub(crate) fn parse_program_element(
        &self,
        pair: Pair,
    ) -> ParseResult<Option<Box<ast::ProgramElement>>> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::var_decl_stmt => {
                    return Ok(Some(Box::new(ast::ProgramElement {
                        inner: ast::ProgramElementInner::VarDeclStmt(
                            self.parse_var_decl_stmt(inner)?,
                        ),
                    })));
                }
                Rule::struct_def => {
                    return Ok(Some(Box::new(ast::ProgramElement {
                        inner: ast::ProgramElementInner::StructDef(self.parse_struct_def(inner)?),
                    })));
                }
                Rule::fn_decl_stmt => {
                    return Ok(Some(Box::new(ast::ProgramElement {
                        inner: ast::ProgramElementInner::FnDeclStmt(
                            self.parse_fn_decl_stmt(inner)?,
                        ),
                    })));
                }
                Rule::fn_def => {
                    return Ok(Some(Box::new(ast::ProgramElement {
                        inner: ast::ProgramElementInner::FnDef(self.parse_fn_def(inner)?),
                    })));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    pub(crate) fn parse_struct_def(&self, pair: Pair) -> ParseResult<Box<ast::StructDef>> {
        let mut identifier = String::new();
        let mut decls = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => identifier = inner.as_str().to_string(),
                Rule::typed_var_decl_list => decls = self.parse_typed_var_decl_list(inner)?,
                _ => {}
            }
        }

        Ok(Box::new(ast::StructDef { identifier, decls }))
    }

    pub(crate) fn parse_typed_var_decl_list(&self, pair: Pair) -> ParseResult<Vec<ast::VarDecl>> {
        let mut decls = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::typed_var_decl {
                decls.push(*self.parse_var_decl(inner)?);
            }
        }
        Ok(decls)
    }

    pub(crate) fn parse_var_decl(&self, pair: Pair) -> ParseResult<Box<ast::VarDecl>> {
        let pair_for_error = pair.clone();
        let mut identifier: Option<String> = None;
        let mut type_specifier: Option<ast::TypeSpecifier> = None;
        let mut array_len: Option<usize> = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier if identifier.is_none() => {
                    identifier = Some(inner.as_str().to_string());
                }
                Rule::type_spec => {
                    type_specifier = self.parse_type_spec(inner)?;
                }
                Rule::num => {
                    array_len = Some(parse_num(inner)? as usize);
                }
                _ => {}
            }
        }

        let identifier =
            identifier.ok_or_else(|| grammar_error("var_decl.identifier", &pair_for_error))?;
        let inner = if let Some(len) = array_len {
            ast::VarDeclInner::Array(Box::new(ast::VarDeclArray { len }))
        } else {
            ast::VarDeclInner::Scalar
        };

        Ok(Box::new(ast::VarDecl {
            identifier,
            type_specifier,
            inner,
        }))
    }

    pub(crate) fn parse_type_spec(&self, pair: Pair) -> ParseResult<Option<ast::TypeSpecifier>> {
        let pos = get_pos(&pair);

        let children: Vec<_> = pair.into_inner().collect();

        for child in &children {
            match child.as_rule() {
                Rule::ref_type => {
                    let ref_children: Vec<_> = child.clone().into_inner().collect();
                    let inner_type_spec = ref_children
                        .iter()
                        .find(|c| c.as_rule() == Rule::type_spec)
                        .expect("Ref type_spec must have inner type_spec");
                    let inner_ts = self
                        .parse_type_spec(inner_type_spec.clone())?
                        .expect("Ref inner type_spec must not be empty");
                    return Ok(Some(ast::TypeSpecifier {
                        pos,
                        inner: ast::TypeSpecifierInner::Reference(Box::new(inner_ts)),
                    }));
                }
                Rule::kw_i32 => {
                    return Ok(Some(ast::TypeSpecifier {
                        pos,
                        inner: ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Int),
                    }));
                }
                Rule::kw_f32 => {
                    return Ok(Some(ast::TypeSpecifier {
                        pos,
                        inner: ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Float),
                    }));
                }
                Rule::array_type => {
                    let array_children: Vec<_> = child.clone().into_inner().collect();
                    // 数组类型子节点顺序: [ '[', type_spec, ';', num, ']' ]
                    let inner_type_spec_pair = array_children[1].clone();
                    let inner_ts = self
                        .parse_type_spec(inner_type_spec_pair)?
                        .expect("Array element type_spec must not be empty");
                    let length_pair = array_children[3].clone();
                    let length = parse_num(length_pair)?; 
                    return Ok(Some(ast::TypeSpecifier {
                        pos,
                        inner: ast::TypeSpecifierInner::Array(Box::new(inner_ts), length),
                    }))
                }
                Rule::identifier => {
                    return Ok(Some(ast::TypeSpecifier {
                        pos,
                        inner: ast::TypeSpecifierInner::Composite(child.as_str().to_string()),
                    }));
                }
                _ => {}
            }
        }

        Ok(None)
    }

    pub(crate) fn parse_var_decl_stmt(&self, pair: Pair) -> ParseResult<Box<ast::VarDeclStmt>> {
        let pair_for_error = pair.clone();
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::var_def => {
                    return Ok(Box::new(ast::VarDeclStmt {
                        inner: ast::VarDeclStmtInner::Def(self.parse_var_def(inner)?),
                    }));
                }
                Rule::var_decl => {
                    return Ok(Box::new(ast::VarDeclStmt {
                        inner: ast::VarDeclStmtInner::Decl(self.parse_var_decl(inner)?),
                    }));
                }
                _ => {}
            }
        }

        Err(grammar_error("var_decl_stmt", &pair_for_error))
    }

    pub(crate) fn parse_var_def(&self, pair: Pair) -> ParseResult<Box<ast::VarDef>> {
        let pair_for_error = pair.clone();
        let inner_pairs: Vec<_> = pair.into_inner().collect();

        let identifier = inner_pairs[0].as_str().to_string();

        let has_initializer = inner_pairs
            .iter()
            .any(|p| p.as_rule() == Rule::array_initializer);
        let has_colon = inner_pairs.iter().any(|p| p.as_rule() == Rule::colon);

        if has_initializer {
            let len = parse_num(
                inner_pairs
                    .iter()
                    .find(|p| p.as_rule() == Rule::num)
                    .ok_or_else(|| grammar_error("var_def.array_len", &pair_for_error))?
                    .clone(),
            )? as usize;

            let type_specifier = if has_colon {
                self.parse_type_spec(
                    inner_pairs
                        .iter()
                        .find(|p| p.as_rule() == Rule::type_spec)
                        .ok_or_else(|| grammar_error("var_def.type_spec", &pair_for_error))?
                        .clone(),
                )?
            } else {
                None
            };

            let initializer = self.parse_array_initializer(
                inner_pairs
                    .iter()
                    .find(|p| p.as_rule() == Rule::array_initializer)
                    .ok_or_else(|| grammar_error("var_def.array_init", &pair_for_error))?
                    .clone(),
            )?;

            Ok(Box::new(ast::VarDef {
                identifier,
                type_specifier,
                inner: ast::VarDefInner::Array(Box::new(ast::VarDefArray { len, initializer })),
            }))
        } else {
            let type_specifier = if has_colon {
                self.parse_type_spec(
                    inner_pairs
                        .iter()
                        .find(|p| p.as_rule() == Rule::type_spec)
                        .ok_or_else(|| grammar_error("var_def.type_spec", &pair_for_error))?
                        .clone(),
                )?
            } else {
                None
            };

            let val = self.parse_right_val(
                inner_pairs
                    .iter()
                    .find(|p| p.as_rule() == Rule::right_val)
                    .ok_or_else(|| grammar_error("var_def.val", &pair_for_error))?
                    .clone(),
            )?;

            Ok(Box::new(ast::VarDef {
                identifier,
                type_specifier,
                inner: ast::VarDefInner::Scalar(Box::new(ast::VarDefScalar { val })),
            }))
        }
    }

    fn parse_array_initializer(&self, pair: Pair) -> ParseResult<ast::ArrayInitializer> {
        let pair_for_error = pair.clone();
        let children: Vec<_> = pair.into_inner().collect();

        if let Some(list_pair) = children
            .iter()
            .find(|p| p.as_rule() == Rule::right_val_list)
        {
            let vals = self.parse_right_val_list(list_pair.clone())?;
            return Ok(ast::ArrayInitializer::ExplicitList(vals));
        }

        let val_pair = children
            .iter()
            .find(|p| p.as_rule() == Rule::right_val)
            .ok_or_else(|| grammar_error("array_initializer.val", &pair_for_error))?;
        let count_pair = children
            .iter()
            .find(|p| p.as_rule() == Rule::num)
            .ok_or_else(|| grammar_error("array_initializer.count", &pair_for_error))?;

        let val = self.parse_right_val(val_pair.clone())?;
        let count = parse_num(count_pair.clone())? as usize;

        Ok(ast::ArrayInitializer::Fill { val, count })
    }

    pub(crate) fn parse_fn_decl_stmt(&self, pair: Pair) -> ParseResult<Box<ast::FnDeclStmt>> {
        let pair_for_error = pair.clone();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::fn_decl {
                return Ok(Box::new(ast::FnDeclStmt {
                    fn_decl: self.parse_fn_decl(inner)?,
                }));
            }
        }

        Err(grammar_error("fn_decl_stmt", &pair_for_error))
    }

    fn parse_fn_decl(&self, pair: Pair) -> ParseResult<Box<ast::FnDecl>> {
        let mut identifier = String::new();
        let mut param_decl = None;
        let mut return_dtype = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => identifier = inner.as_str().to_string(),
                Rule::param_decl => param_decl = Some(self.parse_param_decl(inner)?),
                Rule::type_spec => return_dtype = self.parse_type_spec(inner)?,
                _ => {}
            }
        }

        Ok(Box::new(ast::FnDecl {
            identifier,
            param_decl,
            return_dtype,
        }))
    }

    fn parse_param_decl(&self, pair: Pair) -> ParseResult<Box<ast::ParamDecl>> {
        let pair_for_error = pair.clone();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::typed_var_decl_list {
                return Ok(Box::new(ast::ParamDecl {
                    decls: self.parse_typed_var_decl_list(inner)?,
                }));
            }
        }
        Err(grammar_error("param_decl", &pair_for_error))
    }

    pub(crate) fn parse_fn_def(&self, pair: Pair) -> ParseResult<Box<ast::FnDef>> {
        let pair_for_error = pair.clone();
        let mut fn_decl = None;
        let mut stmts = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::fn_decl => fn_decl = Some(self.parse_fn_decl(inner)?),
                Rule::code_block_stmt => stmts.push(*self.parse_code_block_stmt(inner)?),
                _ => {}
            }
        }

        Ok(Box::new(ast::FnDef {
            fn_decl: fn_decl.ok_or_else(|| grammar_error("fn_def.fn_decl", &pair_for_error))?,
            stmts,
        }))
    }
}
