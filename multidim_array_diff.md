**`src/ir/gen/conversions.rs`**

多维数组真正相关的变化是把 `TypeSpecifierInner::Array` 递归转换成 `Dtype::Array`，以及 `Reference` 继续指向“元素类型的 unsized array”。

```diff
diff --git a/src/ir/gen/conversions.rs b/src/ir/gen/conversions.rs
--- a/src/ir/gen/conversions.rs
+++ b/src/ir/gen/conversions.rs
@@
 impl From<&ast::TypeSpecifier> for Dtype {
     fn from(a: &ast::TypeSpecifier) -> Self {
         match &a.inner {
-            ast::TypeSpecifierInner::BuiltIn(_) => Self::I32,
+            ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Int) => Self::I32,
             ast::TypeSpecifierInner::Composite(name) => Self::Struct {
                 type_name: name.clone(),
             },
             ast::TypeSpecifierInner::Reference(inner) => Self::ptr_to(Dtype::Array {
                 element: Box::new(Self::from(inner.as_ref())),
                 length: None,
             }),
+            ast::TypeSpecifierInner::Array(inner, len) => {
+                Self::array_of(Self::from(inner.as_ref()), *len as usize)
+            }
         }
     }
 }
```

当前对应实现：

```rust
pub(crate) fn compose_var_decl_dtype(base: Dtype, inner: &ast::VarDeclInner) -> Dtype {
    match inner {
        ast::VarDeclInner::Scalar => base,
        ast::VarDeclInner::Array(arr) => Dtype::array_of(base, arr.len),
    }
}

pub(crate) fn compose_var_def_dtype(base: Dtype, inner: &ast::VarDefInner) -> Dtype {
    match inner {
        ast::VarDefInner::Scalar(_) => base,
        ast::VarDefInner::Array(arr) => Dtype::array_of(base, arr.len),
    }
}
```

---

**`src/ir/gen/type_infer.rs`**

这里多维数组的关键不是“把类型压平”，而是“每次索引只剥掉最外层一层 `Array`”，这样 `[[i32; 4]; 3][i]` 会得到 `[i32; 4]`，再索引一次才得到 `i32`。

当前相关实现：

```rust
fn resolve_variable(&self, id: &str) -> Result<Dtype, Error> {
    let dtype = self.lookup_dtype(id)?;
    Ok(match dtype {
        Dtype::Array { element, .. } => element.as_ref().clone(),
        other => other,
    })
}

fn type_of_array_expr(&self, expr: &ast::ArrayExpr) -> Result<Dtype, Error> {
    let arr_type = self.type_of_left_val(&expr.arr)?;
    Ok(Self::element_type_of_indexing(&arr_type))
}

fn element_type_of_indexing(dtype: &Dtype) -> Dtype {
    match dtype {
        Dtype::Array { element, .. } => element.as_ref().clone(),
        Dtype::Pointer { pointee } => match pointee.as_ref() {
            Dtype::Array { element, .. } => element.as_ref().clone(),
            _ => dtype.clone(),
        },
        _ => dtype.clone(),
    }
}

fn type_of_reference(&self, id: &str) -> Result<Dtype, Error> {
    let var_type = self.lookup_dtype(id)?;
    let element_type = match var_type {
        Dtype::Array { element, .. } => element.as_ref().clone(),
        Dtype::Pointer { pointee } => match *pointee {
            Dtype::Array { element, .. } => element.as_ref().clone(),
            _ => {
                return Err(Error::InvalidReference {
                    symbol: id.to_string(),
                });
            }
        },
        _ => {
            return Err(Error::InvalidReference {
                symbol: id.to_string(),
            });
        }
    };
    Ok(Dtype::ptr_to(Dtype::Array {
        element: Box::new(element_type),
        length: None,
    }))
}

fn type_of_left_val_array(&self, expr: &ast::ArrayExpr) -> Result<Dtype, Error> {
    let arr_type = self.type_of_left_val(&expr.arr)?;
    Ok(Self::element_type_of_indexing(&arr_type))
}
```

---

**`src/ir/gen/function_gen.rs`**

这里多维数组的关键是 `handle_array_expr(...)`：GEP 每索引一次，只把结果类型推进到内层元素；如果内层元素本身还是 `Array { ... }`，结果就继续是“指向内层数组的指针”。

当前相关实现：

```rust
fn handle_array_expr(&mut self, expr: &ast::ArrayExpr) -> Result<Operand, Error> {
    let arr = self.handle_left_val(&expr.arr)?;

    let (arr, arr_dtype) = match arr.dtype() {
        Dtype::Pointer { pointee } if matches!(pointee.as_ref(), Dtype::Pointer { .. }) => {
            let loaded = Operand::from(self.fresh_local(pointee.as_ref().clone()));
            self.emit_load(loaded.clone(), arr);
            (loaded.clone(), loaded.dtype().clone())
        }
        _ => (arr.clone(), arr.dtype().clone()),
    };

    let target = match &arr_dtype {
        Dtype::Pointer { pointee } => match pointee.as_ref() {
            Dtype::Array { element, .. } => Ok(Operand::from(
                self.fresh_local(Dtype::ptr_to(element.as_ref().clone())),
            )),
            _ => Ok(Operand::from(
                self.fresh_local(Dtype::ptr_to(pointee.as_ref().clone())),
            )),
        },
        Dtype::Array { element, .. } => Ok(Operand::from(
            self.fresh_local(Dtype::ptr_to(element.as_ref().clone())),
        )),
        _ => Err(Error::InvalidArrayExpression),
    }?;

    let index = self.handle_index_expr(expr.idx.as_ref())?;
    self.emit_gep(target.clone(), arr, index);

    Ok(target)
}
```

补充：`handle_expr_unit(...)` 里对数组标识符直接当值使用会报错，这能避免把多维数组误当成标量加载：

```rust
let is_array = matches!(
    op.dtype(),
    Dtype::Pointer { pointee } if matches!(pointee.as_ref(), Dtype::Array { .. })
) || matches!(op.dtype(), Dtype::Array { .. });
if is_array {
    return Err(Error::ArrayUsedAsValue { symbol: id.clone() });
}
```
