**算术表达式 lowering：递归改迭代**

目标文件：[src/ir/gen/function_gen.rs](/d:/VS%20Code/teac/src/ir/gen/function_gen.rs)

这次真正和“递归改迭代”直接相关的改动，核心只有这一块：把 `handle_arith_expr` 从递归下降改成显式栈的后序遍历，并把“拿到左右操作数后如何生成 IR”的逻辑抽到 `lower_arith_operands(...)`。

---

**旧的递归写法**

```rust
/// Lowers an arithmetic expression (binary operation or a single unit).
fn handle_arith_expr(&mut self, expr: &ast::ArithExpr) -> Result<Operand, Error> {
    match &expr.inner {
        ast::ArithExprInner::ArithBiOpExpr(expr) => self.handle_arith_biop_expr(expr),
        ast::ArithExprInner::CastExpr(cast) => self.handle_cast_expr(cast),
    }
}

/// Lowers a binary arithmetic expression (`left op right`) to an `i32` temporary.
fn handle_arith_biop_expr(&mut self, expr: &ast::ArithBiOpExpr) -> Result<Operand, Error> {
    let left = self.handle_arith_expr(&expr.left)?;
    let right = self.handle_arith_expr(&expr.right)?;
    let dst = Operand::from(self.fresh_local(Dtype::I32));
    self.emit_biop(ArithBinOp::from(&expr.op), left, right, dst.clone());
    Ok(dst)
}
```

---

**新的迭代写法**

```rust
/// Lowers an arithmetic expression (binary operation or a single unit).
fn handle_arith_expr(&mut self, expr: &ast::ArithExpr) -> Result<Operand, Error> {
    self.handle_arith_expr_with_depth(expr, 0)
}

fn handle_arith_expr_with_depth(
    &mut self,
    expr: &ast::ArithExpr,
    depth: usize,
) -> Result<Operand, Error> {
    let mut nodes = vec![(expr, depth, false)];
    let mut values = Vec::new();

    while let Some((expr, depth, visited)) = nodes.pop() {
        if debug_longcode_enabled() && depth > 0 && depth % 200 == 0 && !visited {
            eprintln!("[dbg] irgen::arith depth={depth}");
        }

        match (&expr.inner, visited) {
            (ast::ArithExprInner::ExprUnit(unit), _) => {
                values.push(self.handle_expr_unit(unit)?);
            }
            (ast::ArithExprInner::ArithBiOpExpr(biop), false) => {
                nodes.push((expr, depth, true));
                nodes.push((biop.right.as_ref(), depth + 1, false));
                nodes.push((biop.left.as_ref(), depth + 1, false));
            }
            (ast::ArithExprInner::ArithBiOpExpr(biop), true) => {
                let right = values.pop().expect("missing rhs operand for arithmetic expression");
                let left = values.pop().expect("missing lhs operand for arithmetic expression");
                let value = self.lower_arith_operands(&biop.op, left, right)?;
                values.push(value);
            }
        }
    }

    Ok(values
        .pop()
        .expect("arithmetic expression traversal produced no value"))
}

fn handle_arith_biop_expr_with_depth(
    &mut self,
    expr: &ast::ArithBiOpExpr,
    depth: usize,
) -> Result<Operand, Error> {
    let left = self.handle_arith_expr_with_depth(&expr.left, depth)?;
    let right = self.handle_arith_expr_with_depth(&expr.right, depth)?;

    self.lower_arith_operands(&expr.op, left, right)
}

fn lower_arith_operands(
    &mut self,
    op: &ast::ArithBiOp,
    left: Operand,
    right: Operand,
) -> Result<Operand, Error> {
    let kind = ArithBinOp::from(op);

    match (left.dtype(), right.dtype()) {
        (Dtype::F32, _) | (_, Dtype::F32) => {
            let left = self.coerce_to_f32(left);
            let right = self.coerce_to_f32(right);
            let dst = Operand::from(self.fresh_local(Dtype::F32));
            self.emit_fbiop(kind, left, right, dst.clone());
            Ok(dst)
        }
        (Dtype::I32, Dtype::I32)
        | (Dtype::I1, Dtype::I32)
        | (Dtype::I32, Dtype::I1)
        | (Dtype::I1, Dtype::I1) => {
            let left = self.coerce_to_i32(left);
            let right = self.coerce_to_i32(right);
            let dst = Operand::from(self.fresh_local(Dtype::I32));
            self.emit_biop(kind, left, right, dst.clone());
            Ok(dst)
        }
        _ => Err(Error::TypeMismatch {
            symbol: "<arith-expr>".to_string(),
            expected: left.dtype().clone(),
            actual: right.dtype().clone(),
        }),
    }
}

/// Compatibility entry: old helper now delegates to the new lowering path.
fn handle_arith_biop_expr(&mut self, expr: &ast::ArithBiOpExpr) -> Result<Operand, Error> {
    self.handle_arith_biop_expr_with_depth(expr, 1)
}
```

---

**核心 diff 视图**

```diff
diff --git a/src/ir/gen/function_gen.rs b/src/ir/gen/function_gen.rs
@@
 fn handle_arith_expr(&mut self, expr: &ast::ArithExpr) -> Result<Operand, Error> {
-    match &expr.inner {
-        ast::ArithExprInner::ArithBiOpExpr(expr) => self.handle_arith_biop_expr(expr),
-        ast::ArithExprInner::CastExpr(cast) => self.handle_cast_expr(cast),
-    }
+    self.handle_arith_expr_with_depth(expr, 0)
 }

+fn handle_arith_expr_with_depth(
+    &mut self,
+    expr: &ast::ArithExpr,
+    depth: usize,
+) -> Result<Operand, Error> {
+    let mut nodes = vec![(expr, depth, false)];
+    let mut values = Vec::new();
+
+    while let Some((expr, depth, visited)) = nodes.pop() {
+        match (&expr.inner, visited) {
+            (ast::ArithExprInner::ExprUnit(unit), _) => {
+                values.push(self.handle_expr_unit(unit)?);
+            }
+            (ast::ArithExprInner::ArithBiOpExpr(biop), false) => {
+                nodes.push((expr, depth, true));
+                nodes.push((biop.right.as_ref(), depth + 1, false));
+                nodes.push((biop.left.as_ref(), depth + 1, false));
+            }
+            (ast::ArithExprInner::ArithBiOpExpr(biop), true) => {
+                let right = values.pop().expect("missing rhs operand for arithmetic expression");
+                let left = values.pop().expect("missing lhs operand for arithmetic expression");
+                let value = self.lower_arith_operands(&biop.op, left, right)?;
+                values.push(value);
+            }
+        }
+    }
+
+    Ok(values
+        .pop()
+        .expect("arithmetic expression traversal produced no value"))
+}
+
+fn lower_arith_operands(
+    &mut self,
+    op: &ast::ArithBiOp,
+    left: Operand,
+    right: Operand,
+) -> Result<Operand, Error> {
+    let kind = ArithBinOp::from(op);
+    ...
+}
@@
 fn handle_arith_biop_expr(&mut self, expr: &ast::ArithBiOpExpr) -> Result<Operand, Error> {
-    let left = self.handle_arith_expr(&expr.left)?;
-    let right = self.handle_arith_expr(&expr.right)?;
-    let dst = Operand::from(self.fresh_local(Dtype::I32));
-    self.emit_biop(ArithBinOp::from(&expr.op), left, right, dst.clone());
-    Ok(dst)
+    self.handle_arith_biop_expr_with_depth(expr, 1)
 }
```

---

**这次改动解决的问题**

- 原来 `a + b + c + ...` 这种超长左结合表达式，会形成非常深的 AST。
- 递归 lowering 时，每一层都要额外压一次 Rust 调用栈。
- `long_code2` 里的超长初始化表达式因此触发了 `stack overflow`。
- 改成显式栈 `nodes` 和结果栈 `values` 之后，深度转移到堆上的 `Vec`，不再吃函数调用栈。
