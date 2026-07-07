//! WebAssembly backend — first increment (the scalar subset).
//!
//! This module compiles the typed IR (`IrModule`) directly to a binary `.wasm`
//! module using only the Rust standard library: it implements the WASM binary
//! encoding (magic, version, Type/Function/Export/Code sections, LEB128, and the
//! stack-machine opcodes it needs) from scratch.
//!
//! Scope: only functions whose parameter and return types are all scalars in
//! {`i64`, `f64`, `bool`, `char`, `byte`} compile to WASM. `i64` maps to wasm
//! `i64`, `f64` to `f64`, and `bool`/`char`/`byte` to `i32`. `void` return means
//! no result. Supported bodies: integer/float/bool literals, variables (params +
//! `let` locals), arithmetic (`+ - * /`; integer `/` uses `i64.div_s` which traps
//! on 0), comparisons, `and`/`or`/`not`, `if`/`elif`/`else`, `while`, `loop` with
//! `break`/`continue`, range `for` (lowered to a loop), `return`, and calls to
//! other compiled scalar functions (including recursion). A function that uses any
//! non-scalar type, heap value, `match`, or a builtin is SKIPPED (it still runs on
//! the interpreters). This first increment does not touch linear memory.

use std::collections::HashMap;

use lullaby_parser::{BinaryOp, TypeRef, UnaryOp};

use crate::{IrExpr, IrExprKind, IrFunction, IrModule, IrStmt};

/// A compiled `.wasm` module plus the record of which functions compiled and
/// which were skipped (with a reason).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmArtifact {
    /// The binary `.wasm` module (starts with `\0asm` + version 1).
    pub bytes: Vec<u8>,
    /// Names of functions that compiled to WASM, in module order.
    pub compiled: Vec<String>,
    /// Functions skipped for WASM, each with a human-readable reason.
    pub skipped: Vec<SkippedFunction>,
}

/// A function that was not eligible for the WASM scalar subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedFunction {
    pub name: String,
    pub reason: String,
}

/// A failure while emitting the WASM module. Currently the only hard error is
/// "no functions were eligible", which the CLI surfaces as diagnostic `L0338`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmError {
    pub code: &'static str,
    pub message: String,
    /// Functions skipped, so the CLI can still report why nothing compiled.
    pub skipped: Vec<SkippedFunction>,
}

/// WASM value types used by the scalar subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WasmValType {
    I32,
    I64,
    F64,
}

impl WasmValType {
    /// The binary encoding byte for this value type.
    fn byte(self) -> u8 {
        match self {
            WasmValType::I32 => 0x7f,
            WasmValType::I64 => 0x7e,
            WasmValType::F64 => 0x7c,
        }
    }
}

/// Map a Lullaby scalar `TypeRef` to a WASM value type, or `None` if the type is
/// not in the scalar subset.
fn scalar_val_type(ty: &TypeRef) -> Option<WasmValType> {
    match ty.name.as_str() {
        "i64" => Some(WasmValType::I64),
        "f64" => Some(WasmValType::F64),
        "bool" | "char" | "byte" => Some(WasmValType::I32),
        _ => None,
    }
}

/// The result type for a function: empty for `void`, else one value type.
/// `Err(())` means the return type is a non-scalar (function ineligible).
fn return_val_type(ty: &TypeRef) -> Result<Option<WasmValType>, ()> {
    if ty.is_void() {
        return Ok(None);
    }
    scalar_val_type(ty).map(Some).ok_or(())
}

/// A resolved local: its WASM index and value type.
#[derive(Debug, Clone, Copy)]
struct Local {
    index: u32,
    ty: WasmValType,
}

// -- Public entry point ------------------------------------------------------

/// Emit a binary `.wasm` module for the scalar-subset functions of `module`.
///
/// Every top-level function is examined: an eligible one is lowered and exported
/// by its Lullaby name; an ineligible one is recorded in `skipped` with a reason.
/// If no function is eligible, this returns `Err(WasmError)` with code `L0338`.
pub fn emit_wasm_module(module: &IrModule) -> Result<WasmArtifact, WasmError> {
    // First pass: decide signature eligibility and assign WASM function indices
    // to the functions we will compile. Calls between compiled functions resolve
    // against this index map.
    let mut compiled_names: Vec<String> = Vec::new();
    let mut skipped: Vec<SkippedFunction> = Vec::new();
    let mut func_index: HashMap<String, u32> = HashMap::new();
    let mut eligible: Vec<&IrFunction> = Vec::new();

    for function in &module.functions {
        match eligibility(function) {
            Ok(()) => {
                func_index.insert(function.name.clone(), eligible.len() as u32);
                eligible.push(function);
                compiled_names.push(function.name.clone());
            }
            Err(reason) => skipped.push(SkippedFunction {
                name: function.name.clone(),
                reason,
            }),
        }
    }

    if eligible.is_empty() {
        return Err(WasmError {
            code: "L0338",
            message: "no functions were eligible for the WebAssembly scalar subset".to_string(),
            skipped,
        });
    }

    // Second pass: lower each eligible function's body. A lowering failure (a
    // construct we cannot compile) demotes that function to skipped. Because that
    // changes index assignment, we re-run the whole emission over the reduced
    // set. This converges quickly (each pass removes at least one function).
    let mut lowered: Vec<LoweredFunction> = Vec::new();
    for function in &eligible {
        match lower_function(function, &func_index) {
            Ok(l) => lowered.push(l),
            Err(reason) => {
                let demoted = SkippedFunction {
                    name: function.name.clone(),
                    reason,
                };
                let mut reduced = module.clone();
                reduced.functions.retain(|f| f.name != demoted.name);
                return match emit_wasm_module(&reduced) {
                    Ok(mut artifact) => {
                        artifact.compiled.retain(|n| n != &demoted.name);
                        merge_skip(&mut artifact.skipped, demoted);
                        for s in &skipped {
                            merge_skip(&mut artifact.skipped, s.clone());
                        }
                        Ok(artifact)
                    }
                    Err(mut err) => {
                        merge_skip(&mut err.skipped, demoted);
                        for s in &skipped {
                            merge_skip(&mut err.skipped, s.clone());
                        }
                        Err(err)
                    }
                };
            }
        }
    }

    let bytes = encode_module(&lowered);
    Ok(WasmArtifact {
        bytes,
        compiled: compiled_names,
        skipped,
    })
}

/// Append a skip record unless one with that name is already present.
fn merge_skip(skips: &mut Vec<SkippedFunction>, skip: SkippedFunction) {
    if !skips.iter().any(|s| s.name == skip.name) {
        skips.push(skip);
    }
}

// -- Eligibility -------------------------------------------------------------

/// Check whether a function's signature is entirely in the scalar subset.
fn eligibility(function: &IrFunction) -> Result<(), String> {
    for param in &function.params {
        if scalar_val_type(&param.ty).is_none() {
            return Err(format!(
                "parameter `{}` has non-scalar type `{}`",
                param.name, param.ty.name
            ));
        }
    }
    if return_val_type(&function.return_type).is_err() {
        return Err(format!(
            "return type `{}` is not a scalar",
            function.return_type.name
        ));
    }
    Ok(())
}

// -- Lowering ----------------------------------------------------------------

/// A function lowered to WASM: its signature value types, extra (non-parameter)
/// local declarations, and the encoded body instruction bytes (without the final
/// `end`).
struct LoweredFunction {
    name: String,
    params: Vec<WasmValType>,
    result: Option<WasmValType>,
    /// Locals beyond the parameters, in index order.
    extra_locals: Vec<WasmValType>,
    body: Vec<u8>,
}

/// Per-function lowering state.
struct LowerCtx<'a> {
    /// name -> (index, type) for every param and `let`/`for` local.
    locals: HashMap<String, Local>,
    /// Value types of the extra (non-param) locals, in index order.
    extra_locals: Vec<WasmValType>,
    /// Number of parameters (locals 0..param_count are params).
    param_count: u32,
    /// Function-name -> WASM function index, for calls.
    func_index: &'a HashMap<String, u32>,
}

impl<'a> LowerCtx<'a> {
    fn new(function: &IrFunction, func_index: &'a HashMap<String, u32>) -> Result<Self, String> {
        let mut locals = HashMap::new();
        for (i, param) in function.params.iter().enumerate() {
            let ty = scalar_val_type(&param.ty)
                .ok_or_else(|| format!("parameter `{}` is not a scalar", param.name))?;
            locals.insert(
                param.name.clone(),
                Local {
                    index: i as u32,
                    ty,
                },
            );
        }
        Ok(Self {
            locals,
            extra_locals: Vec::new(),
            param_count: function.params.len() as u32,
            func_index,
        })
    }

    /// Allocate a fresh non-parameter local of the given type; return its index.
    fn add_local(&mut self, ty: WasmValType) -> u32 {
        let index = self.param_count + self.extra_locals.len() as u32;
        self.extra_locals.push(ty);
        index
    }
}

fn lower_function(
    function: &IrFunction,
    func_index: &HashMap<String, u32>,
) -> Result<LoweredFunction, String> {
    let result = match return_val_type(&function.return_type) {
        Ok(result) => result,
        Err(()) => return Err("return type is not a scalar".to_string()),
    };
    let mut ctx = LowerCtx::new(function, func_index)?;

    let mut body = Vec::new();
    lower_stmts(&mut ctx, &function.body, &mut body, &LoopCtx::none())?;

    // A non-void function must leave a value on every path. A trailing `Return`
    // or a value-producing tail `Expr` guarantees this; otherwise reject (the
    // interpreter still runs it) so WASM validation cannot fail.
    if result.is_some() && !body_guarantees_value(&function.body) {
        return Err(
            "non-void function may fall through without a return value (unsupported in WASM)"
                .to_string(),
        );
    }

    let params = function
        .params
        .iter()
        .map(|p| scalar_val_type(&p.ty).expect("checked eligible"))
        .collect();

    Ok(LoweredFunction {
        name: function.name.clone(),
        params,
        result,
        extra_locals: ctx.extra_locals,
        body,
    })
}

/// Loop context: branch depths (relative to the current point) for `break` and
/// `continue`. WASM `br N` targets the N-th enclosing structured block.
#[derive(Clone, Copy)]
struct LoopCtx {
    /// Relative depth of the enclosing `block` whose `end` is past the loop
    /// (target of `break`), or `None` if not in a loop.
    break_depth: Option<u32>,
    /// Relative depth of the enclosing `loop` (target of `continue`).
    continue_depth: Option<u32>,
}

impl LoopCtx {
    fn none() -> Self {
        Self {
            break_depth: None,
            continue_depth: None,
        }
    }

    /// Enter a structured block that does not change the loop targets but adds a
    /// level of nesting (e.g. an `if` block). Increments existing depths by 1.
    fn nest(self) -> Self {
        Self {
            break_depth: self.break_depth.map(|d| d + 1),
            continue_depth: self.continue_depth.map(|d| d + 1),
        }
    }
}

fn lower_stmts(
    ctx: &mut LowerCtx,
    stmts: &[IrStmt],
    out: &mut Vec<u8>,
    loops: &LoopCtx,
) -> Result<(), String> {
    for stmt in stmts {
        lower_stmt(ctx, stmt, out, loops)?;
    }
    Ok(())
}

fn lower_stmt(
    ctx: &mut LowerCtx,
    stmt: &IrStmt,
    out: &mut Vec<u8>,
    loops: &LoopCtx,
) -> Result<(), String> {
    match stmt {
        IrStmt::Let {
            name, ty, value, ..
        } => {
            let vty = scalar_val_type(ty)
                .ok_or_else(|| format!("`let {name}` has non-scalar type `{}`", ty.name))?;
            lower_expr(ctx, value, out)?;
            let index = ctx.add_local(vty);
            ctx.locals.insert(name.clone(), Local { index, ty: vty });
            set_local(out, index);
            Ok(())
        }
        IrStmt::Assign {
            name,
            path,
            op,
            value,
            ..
        } => {
            if !path.is_empty() {
                return Err("field/index assignment is not in the WASM scalar subset".to_string());
            }
            let local = *ctx
                .locals
                .get(name)
                .ok_or_else(|| format!("assignment to unknown local `{name}`"))?;
            match op {
                lullaby_parser::AssignOp::Replace => {
                    lower_expr(ctx, value, out)?;
                }
                other => {
                    // Compound assignment: local = local <op> value.
                    get_local(out, local.index);
                    lower_expr(ctx, value, out)?;
                    let bin = match other {
                        lullaby_parser::AssignOp::Add => BinaryOp::Add,
                        lullaby_parser::AssignOp::Subtract => BinaryOp::Subtract,
                        lullaby_parser::AssignOp::Multiply => BinaryOp::Multiply,
                        lullaby_parser::AssignOp::Divide => BinaryOp::Divide,
                        lullaby_parser::AssignOp::Replace => unreachable!(),
                    };
                    emit_binary_op_typed(bin, local.ty, out)?;
                }
            }
            set_local(out, local.index);
            Ok(())
        }
        IrStmt::Return(value) => {
            if let Some(expr) = value {
                lower_expr(ctx, expr, out)?;
            }
            out.push(0x0f); // return
            Ok(())
        }
        IrStmt::Break(_) => {
            let depth = loops
                .break_depth
                .ok_or_else(|| "`break` outside a loop".to_string())?;
            out.push(0x0c); // br
            write_uleb(out, depth as u64);
            Ok(())
        }
        IrStmt::Continue(_) => {
            let depth = loops
                .continue_depth
                .ok_or_else(|| "`continue` outside a loop".to_string())?;
            out.push(0x0c); // br
            write_uleb(out, depth as u64);
            Ok(())
        }
        IrStmt::Expr(expr) => {
            // In the supported subset a value-producing expression only appears
            // as the tail of a non-void function (handled by the implicit `end`).
            // A void expression (e.g. a call returning void) pushes nothing.
            // Anything else (a value-producing statement not in tail position) is
            // rejected so the stack stays balanced.
            if expr_val_type(ctx, expr)?.is_some() {
                // Tail value: leave it on the stack for the function `end`.
                lower_expr(ctx, expr, out)?;
                Ok(())
            } else {
                lower_expr(ctx, expr, out)?;
                Ok(())
            }
        }
        IrStmt::If {
            branches,
            else_body,
            ..
        } => lower_if(ctx, branches, else_body, out, loops),
        IrStmt::While {
            condition, body, ..
        } => lower_while(ctx, condition, body, out),
        IrStmt::Loop { body, .. } => lower_loop(ctx, body, out),
        IrStmt::For {
            name,
            start,
            end,
            step,
            body,
            ..
        } => lower_for(ctx, name, start, end, step.as_ref(), body, out),
        IrStmt::Throw { .. } | IrStmt::Try { .. } => {
            Err("throw/try is not in the WASM scalar subset".to_string())
        }
        IrStmt::Match { .. } => Err("match is not in the WASM scalar subset".to_string()),
    }
}

/// Lower an `if`/`elif`/`else` chain to nested WASM `if`/`else` blocks (void
/// result type — the branches are statement blocks).
fn lower_if(
    ctx: &mut LowerCtx,
    branches: &[crate::IrIfBranch],
    else_body: &[IrStmt],
    out: &mut Vec<u8>,
    loops: &LoopCtx,
) -> Result<(), String> {
    lower_if_from(ctx, branches, 0, else_body, out, loops)
}

fn lower_if_from(
    ctx: &mut LowerCtx,
    branches: &[crate::IrIfBranch],
    idx: usize,
    else_body: &[IrStmt],
    out: &mut Vec<u8>,
    loops: &LoopCtx,
) -> Result<(), String> {
    let branch = &branches[idx];
    lower_expr(ctx, &branch.condition, out)?; // condition (i32)
    out.push(0x04); // if
    out.push(0x40); // void block type
    let inner = loops.nest();
    lower_stmts(ctx, &branch.body, out, &inner)?;
    out.push(0x05); // else
    if idx + 1 < branches.len() {
        lower_if_from(ctx, branches, idx + 1, else_body, out, &inner)?;
    } else {
        lower_stmts(ctx, else_body, out, &inner)?;
    }
    out.push(0x0b); // end
    Ok(())
}

/// Lower a `while`: `block { loop { br_if(!cond) end; body; br loop } }`.
fn lower_while(
    ctx: &mut LowerCtx,
    condition: &IrExpr,
    body: &[IrStmt],
    out: &mut Vec<u8>,
) -> Result<(), String> {
    out.push(0x02); // block
    out.push(0x40);
    out.push(0x03); // loop
    out.push(0x40);
    // depth 0 = loop (continue), depth 1 = block (break).
    let inner = LoopCtx {
        break_depth: Some(1),
        continue_depth: Some(0),
    };
    lower_expr(ctx, condition, out)?;
    out.push(0x45); // i32.eqz
    out.push(0x0d); // br_if 1 (break when condition is false)
    write_uleb(out, 1);
    lower_stmts(ctx, body, out, &inner)?;
    out.push(0x0c); // br 0 (repeat)
    write_uleb(out, 0);
    out.push(0x0b); // end loop
    out.push(0x0b); // end block
    Ok(())
}

/// Lower an infinite `loop` with `break`/`continue`:
/// `block { loop { body; br loop } }`.
fn lower_loop(ctx: &mut LowerCtx, body: &[IrStmt], out: &mut Vec<u8>) -> Result<(), String> {
    out.push(0x02); // block
    out.push(0x40);
    out.push(0x03); // loop
    out.push(0x40);
    let inner = LoopCtx {
        break_depth: Some(1),
        continue_depth: Some(0),
    };
    lower_stmts(ctx, body, out, &inner)?;
    out.push(0x0c); // br 0 (repeat)
    write_uleb(out, 0);
    out.push(0x0b); // end loop
    out.push(0x0b); // end block
    Ok(())
}

/// Lower a range `for` to a loop over an `i64` induction variable, mirroring the
/// interpreter's inclusive range with an optional step: ascending stops when
/// `i > end`, descending when `i < end`.
#[allow(clippy::too_many_arguments)]
fn lower_for(
    ctx: &mut LowerCtx,
    name: &str,
    start: &IrExpr,
    end: &IrExpr,
    step: Option<&IrExpr>,
    body: &[IrStmt],
    out: &mut Vec<u8>,
) -> Result<(), String> {
    let i_index = ctx.add_local(WasmValType::I64);
    ctx.locals.insert(
        name.to_string(),
        Local {
            index: i_index,
            ty: WasmValType::I64,
        },
    );
    let end_index = ctx.add_local(WasmValType::I64);
    let step_index = ctx.add_local(WasmValType::I64);

    // i = start
    lower_expr(ctx, start, out)?;
    set_local(out, i_index);
    // end_local = end
    lower_expr(ctx, end, out)?;
    set_local(out, end_index);
    // step_local = step (default 1)
    match step {
        Some(step_expr) => lower_expr(ctx, step_expr, out)?,
        None => {
            out.push(0x42); // i64.const
            write_sleb(out, 1);
        }
    }
    set_local(out, step_index);

    out.push(0x02); // block
    out.push(0x40);
    out.push(0x03); // loop
    out.push(0x40);
    let inner = LoopCtx {
        break_depth: Some(1),
        continue_depth: Some(0),
    };

    // cond = (step >= 0) ? (i <= end) : (i >= end)
    get_local(out, step_index);
    out.push(0x42); // i64.const 0
    write_sleb(out, 0);
    out.push(0x59); // i64.ge_s
    out.push(0x04); // if
    out.push(0x7f); // result i32
    get_local(out, i_index);
    get_local(out, end_index);
    out.push(0x57); // i64.le_s
    out.push(0x05); // else
    get_local(out, i_index);
    get_local(out, end_index);
    out.push(0x59); // i64.ge_s
    out.push(0x0b); // end if -> i32 cond on stack
    out.push(0x45); // i32.eqz
    out.push(0x0d); // br_if 1 (break when cond false)
    write_uleb(out, 1);

    lower_stmts(ctx, body, out, &inner)?;

    // i += step
    get_local(out, i_index);
    get_local(out, step_index);
    out.push(0x7c); // i64.add
    set_local(out, i_index);

    out.push(0x0c); // br 0
    write_uleb(out, 0);
    out.push(0x0b); // end loop
    out.push(0x0b); // end block
    Ok(())
}

// -- Expression lowering -----------------------------------------------------

fn lower_expr(ctx: &mut LowerCtx, expr: &IrExpr, out: &mut Vec<u8>) -> Result<(), String> {
    match &expr.kind {
        IrExprKind::Integer(value) => {
            out.push(0x42); // i64.const
            write_sleb(out, *value);
            Ok(())
        }
        IrExprKind::Float(value) => {
            out.push(0x44); // f64.const
            out.extend_from_slice(&value.to_le_bytes());
            Ok(())
        }
        IrExprKind::Bool(value) => {
            out.push(0x41); // i32.const
            write_sleb(out, if *value { 1 } else { 0 });
            Ok(())
        }
        IrExprKind::Char(value) => {
            out.push(0x41); // i32.const
            write_sleb(out, *value as i64);
            Ok(())
        }
        IrExprKind::Variable(name) => {
            let local = ctx
                .locals
                .get(name)
                .ok_or_else(|| format!("unknown variable `{name}`"))?;
            get_local(out, local.index);
            Ok(())
        }
        IrExprKind::Unary { op, expr: inner } => match op {
            UnaryOp::Not => {
                lower_expr(ctx, inner, out)?;
                out.push(0x45); // i32.eqz (bool not)
                Ok(())
            }
        },
        IrExprKind::Binary { left, op, right } => lower_binary(ctx, left, *op, right, out),
        IrExprKind::Call { name, args } => {
            let index = *ctx
                .func_index
                .get(name)
                .ok_or_else(|| format!("call to non-scalar or unknown function `{name}`"))?;
            for arg in args {
                lower_expr(ctx, arg, out)?;
            }
            out.push(0x10); // call
            write_uleb(out, index as u64);
            Ok(())
        }
        IrExprKind::String(_)
        | IrExprKind::Array(_)
        | IrExprKind::Index { .. }
        | IrExprKind::Field { .. } => {
            Err("expression uses a non-scalar value (unsupported in WASM)".to_string())
        }
    }
}

fn lower_binary(
    ctx: &mut LowerCtx,
    left: &IrExpr,
    op: BinaryOp,
    right: &IrExpr,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    // Short-circuit `and`/`or` via WASM `if`/`else` producing i32.
    match op {
        BinaryOp::And => {
            lower_expr(ctx, left, out)?;
            out.push(0x04); // if
            out.push(0x7f); // result i32
            lower_expr(ctx, right, out)?;
            out.push(0x05); // else
            out.push(0x41); // i32.const 0
            write_sleb(out, 0);
            out.push(0x0b); // end
            return Ok(());
        }
        BinaryOp::Or => {
            lower_expr(ctx, left, out)?;
            out.push(0x04); // if
            out.push(0x7f); // result i32
            out.push(0x41); // i32.const 1
            write_sleb(out, 1);
            out.push(0x05); // else
            lower_expr(ctx, right, out)?;
            out.push(0x0b); // end
            return Ok(());
        }
        _ => {}
    }

    // The operand value type is that of the left operand.
    let operand_ty = expr_val_type(ctx, left)?
        .ok_or_else(|| "binary operand has no scalar value".to_string())?;
    lower_expr(ctx, left, out)?;
    lower_expr(ctx, right, out)?;
    emit_binary_op_typed(op, operand_ty, out)
}

/// Emit the opcode(s) for a binary op given the operand WASM type.
fn emit_binary_op_typed(op: BinaryOp, ty: WasmValType, out: &mut Vec<u8>) -> Result<(), String> {
    match ty {
        WasmValType::I64 => emit_i64_binop(op, out),
        WasmValType::F64 => emit_f64_binop(op, out),
        WasmValType::I32 => emit_i32_binop(op, out),
    }
}

fn emit_i64_binop(op: BinaryOp, out: &mut Vec<u8>) -> Result<(), String> {
    let opcode = match op {
        BinaryOp::Add => 0x7c,
        BinaryOp::Subtract => 0x7d,
        BinaryOp::Multiply => 0x7e,
        BinaryOp::Divide => 0x7f, // i64.div_s (traps on 0)
        BinaryOp::Equal => 0x51,
        BinaryOp::NotEqual => 0x52,
        BinaryOp::Less => 0x53,         // lt_s
        BinaryOp::LessEqual => 0x57,    // le_s
        BinaryOp::Greater => 0x55,      // gt_s
        BinaryOp::GreaterEqual => 0x59, // ge_s
        BinaryOp::And | BinaryOp::Or => unreachable!("handled by caller"),
    };
    out.push(opcode);
    Ok(())
}

fn emit_f64_binop(op: BinaryOp, out: &mut Vec<u8>) -> Result<(), String> {
    let opcode = match op {
        BinaryOp::Add => 0xa0,
        BinaryOp::Subtract => 0xa1,
        BinaryOp::Multiply => 0xa2,
        BinaryOp::Divide => 0xa3,
        BinaryOp::Equal => 0x61,
        BinaryOp::NotEqual => 0x62,
        BinaryOp::Less => 0x63,
        BinaryOp::LessEqual => 0x65,
        BinaryOp::Greater => 0x64,
        BinaryOp::GreaterEqual => 0x66,
        BinaryOp::And | BinaryOp::Or => unreachable!("handled by caller"),
    };
    out.push(opcode);
    Ok(())
}

/// `i32` operands are `bool`/`char`/`byte`. Comparisons use the signed opcodes;
/// arithmetic is supported defensively.
fn emit_i32_binop(op: BinaryOp, out: &mut Vec<u8>) -> Result<(), String> {
    let opcode = match op {
        BinaryOp::Add => 0x6a,
        BinaryOp::Subtract => 0x6b,
        BinaryOp::Multiply => 0x6c,
        BinaryOp::Divide => 0x6d, // i32.div_s
        BinaryOp::Equal => 0x46,
        BinaryOp::NotEqual => 0x47,
        BinaryOp::Less => 0x48,         // lt_s
        BinaryOp::LessEqual => 0x4c,    // le_s
        BinaryOp::Greater => 0x4a,      // gt_s
        BinaryOp::GreaterEqual => 0x4e, // ge_s
        BinaryOp::And | BinaryOp::Or => unreachable!("handled by caller"),
    };
    out.push(opcode);
    Ok(())
}

/// The WASM value type an expression leaves on the stack, using the IR's type
/// annotation. `None` for a `void` expression.
fn expr_val_type(ctx: &LowerCtx, expr: &IrExpr) -> Result<Option<WasmValType>, String> {
    let _ = ctx;
    if expr.ty.is_void() {
        return Ok(None);
    }
    if let Some(vt) = scalar_val_type(&expr.ty) {
        return Ok(Some(vt));
    }
    Err(format!("expression has non-scalar type `{}`", expr.ty.name))
}

/// Whether a non-void function body always leaves a value / returns on every
/// path. Conservative: accept a trailing `Return(Some)`, a value-producing tail
/// `Expr`, an exhaustive `If` whose branches all guarantee a value, or a `loop`
/// whose body contains a `Return`.
fn body_guarantees_value(body: &[IrStmt]) -> bool {
    match body.last() {
        Some(IrStmt::Return(Some(_))) => true,
        Some(IrStmt::Expr(expr)) => !expr.ty.is_void(),
        Some(IrStmt::If {
            branches,
            else_body,
            ..
        }) => {
            !else_body.is_empty()
                && body_guarantees_value(else_body)
                && branches.iter().all(|b| body_guarantees_value(&b.body))
        }
        Some(IrStmt::Loop { body, .. }) => stmts_contain_return(body),
        _ => false,
    }
}

fn stmts_contain_return(stmts: &[IrStmt]) -> bool {
    stmts.iter().any(|s| match s {
        IrStmt::Return(_) => true,
        IrStmt::If {
            branches,
            else_body,
            ..
        } => {
            branches.iter().any(|b| stmts_contain_return(&b.body))
                || stmts_contain_return(else_body)
        }
        IrStmt::While { body, .. } | IrStmt::Loop { body, .. } | IrStmt::For { body, .. } => {
            stmts_contain_return(body)
        }
        _ => false,
    })
}

// -- Local get/set helpers ---------------------------------------------------

fn get_local(out: &mut Vec<u8>, index: u32) {
    out.push(0x20);
    write_uleb(out, index as u64);
}

fn set_local(out: &mut Vec<u8>, index: u32) {
    out.push(0x21);
    write_uleb(out, index as u64);
}

// -- Binary encoder ----------------------------------------------------------

/// Unsigned LEB128.
fn write_uleb(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Signed LEB128.
fn write_sleb(out: &mut Vec<u8>, mut value: i64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7; // arithmetic shift
        let sign_bit = byte & 0x40;
        let done = (value == 0 && sign_bit == 0) || (value == -1 && sign_bit != 0);
        if !done {
            byte |= 0x80;
        }
        out.push(byte);
        if done {
            break;
        }
    }
}

/// A distinct signature (parameters + optional result). Functions with the same
/// signature share a type-section entry.
#[derive(Clone, PartialEq, Eq)]
struct FuncType {
    params: Vec<WasmValType>,
    result: Option<WasmValType>,
}

/// Encode the whole module: header + Type, Function, Export, Code sections.
fn encode_module(functions: &[LoweredFunction]) -> Vec<u8> {
    // Deduplicate signatures into a type table.
    let mut types: Vec<FuncType> = Vec::new();
    let mut type_of_func: Vec<u32> = Vec::with_capacity(functions.len());
    for f in functions {
        let sig = FuncType {
            params: f.params.clone(),
            result: f.result,
        };
        let idx = match types.iter().position(|t| *t == sig) {
            Some(i) => i as u32,
            None => {
                types.push(sig);
                (types.len() - 1) as u32
            }
        };
        type_of_func.push(idx);
    }

    let mut module = Vec::new();
    // Magic + version.
    module.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

    // Type section (id 1).
    {
        let mut section = Vec::new();
        write_uleb(&mut section, types.len() as u64);
        for t in &types {
            section.push(0x60); // func type
            write_uleb(&mut section, t.params.len() as u64);
            for p in &t.params {
                section.push(p.byte());
            }
            match t.result {
                Some(vt) => {
                    write_uleb(&mut section, 1);
                    section.push(vt.byte());
                }
                None => write_uleb(&mut section, 0),
            }
        }
        push_section(&mut module, 1, &section);
    }

    // Function section (id 3): type index per function.
    {
        let mut section = Vec::new();
        write_uleb(&mut section, functions.len() as u64);
        for &t in &type_of_func {
            write_uleb(&mut section, t as u64);
        }
        push_section(&mut module, 3, &section);
    }

    // Export section (id 7): export each function by its Lullaby name.
    {
        let mut section = Vec::new();
        write_uleb(&mut section, functions.len() as u64);
        for (i, f) in functions.iter().enumerate() {
            let name = f.name.as_bytes();
            write_uleb(&mut section, name.len() as u64);
            section.extend_from_slice(name);
            section.push(0x00); // export kind: func
            write_uleb(&mut section, i as u64);
        }
        push_section(&mut module, 7, &section);
    }

    // Code section (id 10).
    {
        let mut section = Vec::new();
        write_uleb(&mut section, functions.len() as u64);
        for f in functions {
            let mut code = Vec::new();
            // Locals: run-length compressed consecutive same-type runs.
            let runs = compress_locals(&f.extra_locals);
            write_uleb(&mut code, runs.len() as u64);
            for (count, ty) in runs {
                write_uleb(&mut code, count as u64);
                code.push(ty.byte());
            }
            code.extend_from_slice(&f.body);
            code.push(0x0b); // end
            write_uleb(&mut section, code.len() as u64);
            section.extend_from_slice(&code);
        }
        push_section(&mut module, 10, &section);
    }

    module
}

/// Run-length compress a local declaration list into `(count, type)` runs.
fn compress_locals(locals: &[WasmValType]) -> Vec<(u32, WasmValType)> {
    let mut runs: Vec<(u32, WasmValType)> = Vec::new();
    for &ty in locals {
        match runs.last_mut() {
            Some((count, last)) if *last == ty => *count += 1,
            _ => runs.push((1, ty)),
        }
    }
    runs
}

/// Append a section: `id`, byte length, then the section contents.
fn push_section(module: &mut Vec<u8>, id: u8, contents: &[u8]) {
    module.push(id);
    write_uleb(module, contents.len() as u64);
    module.extend_from_slice(contents);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower;
    use lullaby_lexer::lex;
    use lullaby_parser::parse;
    use lullaby_semantics::validate;

    fn module_for(source: &str) -> IrModule {
        let tokens = lex(source).expect("lex");
        let program = parse(&tokens).expect("parse");
        let checked = validate(&program).expect("semantic");
        lower(&checked).expect("lower")
    }

    #[test]
    fn header_is_wasm_magic_and_version() {
        let source = "fn add a i64 b i64 -> i64\n    a + b\n";
        let artifact = emit_wasm_module(&module_for(source)).expect("emit");
        assert_eq!(
            &artifact.bytes[0..8],
            &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
        );
        assert_eq!(artifact.compiled, vec!["add".to_string()]);
        assert!(artifact.skipped.is_empty());
    }

    #[test]
    fn expected_sections_are_present() {
        let source = "fn add a i64 b i64 -> i64\n    a + b\n";
        let artifact = emit_wasm_module(&module_for(source)).expect("emit");
        let ids = section_ids(&artifact.bytes);
        assert_eq!(ids, vec![1, 3, 7, 10], "type/function/export/code sections");
    }

    #[test]
    fn scalar_and_nonscalar_split() {
        let source = "fn add a i64 b i64 -> i64\n    a + b\n\nfn greet s string -> string\n    s\n";
        let artifact = emit_wasm_module(&module_for(source)).expect("emit");
        assert_eq!(artifact.compiled, vec!["add".to_string()]);
        assert_eq!(artifact.skipped.len(), 1);
        assert_eq!(artifact.skipped[0].name, "greet");
        assert!(artifact.skipped[0].reason.contains("non-scalar"));
    }

    #[test]
    fn recursive_function_compiles() {
        let source = "fn fib n i64 -> i64\n    if n < 2\n        return n\n    return fib(n - 1) + fib(n - 2)\n";
        let artifact = emit_wasm_module(&module_for(source)).expect("emit");
        assert_eq!(artifact.compiled, vec!["fib".to_string()]);
    }

    #[test]
    fn bool_returning_comparison_compiles() {
        let source = "fn is_pos n i64 -> bool\n    n > 0\n";
        let artifact = emit_wasm_module(&module_for(source)).expect("emit");
        assert_eq!(artifact.compiled, vec!["is_pos".to_string()]);
    }

    #[test]
    fn no_eligible_functions_errors() {
        let source = "fn greet s string -> string\n    s\n";
        let err = emit_wasm_module(&module_for(source)).expect_err("no eligible");
        assert_eq!(err.code, "L0338");
        assert_eq!(err.skipped.len(), 1);
    }

    #[test]
    fn uleb_and_sleb_roundtrip() {
        let mut out = Vec::new();
        write_uleb(&mut out, 0);
        assert_eq!(out, vec![0x00]);
        out.clear();
        write_uleb(&mut out, 624485);
        assert_eq!(out, vec![0xe5, 0x8e, 0x26]);
        out.clear();
        write_sleb(&mut out, -123456);
        assert_eq!(out, vec![0xc0, 0xbb, 0x78]);
        out.clear();
        write_sleb(&mut out, 0);
        assert_eq!(out, vec![0x00]);
    }

    /// Parse the section ids present in a module (skipping the 8-byte header).
    fn section_ids(bytes: &[u8]) -> Vec<u8> {
        let mut ids = Vec::new();
        let mut i = 8;
        while i < bytes.len() {
            let id = bytes[i];
            i += 1;
            let (len, consumed) = read_uleb(&bytes[i..]);
            i += consumed;
            i += len as usize;
            ids.push(id);
        }
        ids
    }

    fn read_uleb(bytes: &[u8]) -> (u64, usize) {
        let mut result = 0u64;
        let mut shift = 0;
        let mut i = 0;
        loop {
            let byte = bytes[i];
            result |= ((byte & 0x7f) as u64) << shift;
            i += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        (result, i)
    }
}
