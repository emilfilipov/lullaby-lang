use std::collections::HashMap;
use std::fmt;

use nous_parser::{BinaryOp, Expr, ExprKind, Function, Program, Stmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    I64(i64),
    Bool(bool),
    String(String),
    Ptr(usize),
    Void,
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I64(value) => write!(formatter, "{value}"),
            Self::Bool(value) => write!(formatter, "{value}"),
            Self::String(value) => write!(formatter, "{value}"),
            Self::Ptr(slot) => write!(formatter, "ptr({slot})"),
            Self::Void => write!(formatter, "void"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub code: &'static str,
    pub message: String,
}

impl RuntimeError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn run_main(program: &Program) -> Result<Value, RuntimeError> {
    let mut runtime = Runtime::new(program)?;
    runtime.call_function("main", Vec::new())
}

struct Runtime<'a> {
    functions: HashMap<&'a str, &'a Function>,
    heap: Vec<Option<Value>>,
}

impl<'a> Runtime<'a> {
    fn new(program: &'a Program) -> Result<Self, RuntimeError> {
        let functions = program
            .functions
            .iter()
            .map(|function| (function.name.as_str(), function))
            .collect::<HashMap<_, _>>();

        if !functions.contains_key("main") {
            return Err(RuntimeError::new("N0400", "missing `main` function"));
        }

        Ok(Self {
            functions,
            heap: Vec::new(),
        })
    }

    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match name {
            "alloc" => self.builtin_alloc(args),
            "load" => self.builtin_load(args),
            "dealloc" => self.builtin_dealloc(args),
            _ => {
                let function = *self.functions.get(name).ok_or_else(|| {
                    RuntimeError::new("N0401", format!("unknown function `{name}`"))
                })?;

                if function.params.len() != args.len() {
                    return Err(RuntimeError::new(
                        "N0402",
                        format!(
                            "function `{name}` expects {} arguments but got {}",
                            function.params.len(),
                            args.len()
                        ),
                    ));
                }

                let mut env = HashMap::new();
                for (param, value) in function.params.iter().zip(args) {
                    env.insert(param.name.clone(), value);
                }

                match self.eval_block(&function.body, &mut env)? {
                    Control::Return(value) | Control::Value(value) => Ok(value),
                }
            }
        }
    }

    fn eval_block(
        &mut self,
        statements: &[Stmt],
        env: &mut HashMap<String, Value>,
    ) -> Result<Control, RuntimeError> {
        let mut last = Value::Void;

        for statement in statements {
            match self.eval_statement(statement, env)? {
                Control::Return(value) => return Ok(Control::Return(value)),
                Control::Value(value) => last = value,
            }
        }

        Ok(Control::Value(last))
    }

    fn eval_statement(
        &mut self,
        statement: &Stmt,
        env: &mut HashMap<String, Value>,
    ) -> Result<Control, RuntimeError> {
        match statement {
            Stmt::Let { name, value, .. } => {
                let value = self.eval_expr(value, env)?;
                env.insert(name.clone(), value);
                Ok(Control::Value(Value::Void))
            }
            Stmt::Return(expr) => {
                let value = expr
                    .as_ref()
                    .map(|expr| self.eval_expr(expr, env))
                    .unwrap_or(Ok(Value::Void))?;
                Ok(Control::Return(value))
            }
            Stmt::Expr(expr) => self.eval_expr(expr, env).map(Control::Value),
            Stmt::If {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    let condition = self.eval_expr(&branch.condition, env)?;
                    if condition.as_bool()? {
                        let mut branch_env = env.clone();
                        return self.eval_block(&branch.body, &mut branch_env);
                    }
                }

                let mut else_env = env.clone();
                self.eval_block(else_body, &mut else_env)
            }
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        match &expr.kind {
            ExprKind::Integer(value) => Ok(Value::I64(*value)),
            ExprKind::Bool(value) => Ok(Value::Bool(*value)),
            ExprKind::String(value) => Ok(Value::String(value.clone())),
            ExprKind::Variable(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::new("N0403", format!("unknown variable `{name}`"))),
            ExprKind::Binary { left, op, right } => {
                let left = self.eval_expr(left, env)?;
                let right = self.eval_expr(right, env)?;
                self.eval_binary(left, *op, right)
            }
            ExprKind::Call { name, args } => {
                let values = args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call_function(name, values)
            }
        }
    }

    fn eval_binary(&self, left: Value, op: BinaryOp, right: Value) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Add => Ok(Value::I64(left.as_i64()? + right.as_i64()?)),
            BinaryOp::Subtract => Ok(Value::I64(left.as_i64()? - right.as_i64()?)),
            BinaryOp::Multiply => Ok(Value::I64(left.as_i64()? * right.as_i64()?)),
            BinaryOp::Divide => {
                let divisor = right.as_i64()?;
                if divisor == 0 {
                    Err(RuntimeError::new("N0404", "division by zero"))
                } else {
                    Ok(Value::I64(left.as_i64()? / divisor))
                }
            }
            BinaryOp::Equal => Ok(Value::Bool(left == right)),
            BinaryOp::NotEqual => Ok(Value::Bool(left != right)),
            BinaryOp::Less => Ok(Value::Bool(left.as_i64()? < right.as_i64()?)),
            BinaryOp::LessEqual => Ok(Value::Bool(left.as_i64()? <= right.as_i64()?)),
            BinaryOp::Greater => Ok(Value::Bool(left.as_i64()? > right.as_i64()?)),
            BinaryOp::GreaterEqual => Ok(Value::Bool(left.as_i64()? >= right.as_i64()?)),
        }
    }

    fn builtin_alloc(&mut self, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let [value]: [Value; 1] = args
            .try_into()
            .map_err(|args: Vec<Value>| Self::wrong_arity("alloc", 1, args.len()))?;
        self.heap.push(Some(value));
        Ok(Value::Ptr(self.heap.len() - 1))
    }

    fn builtin_load(&self, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let [ptr]: [Value; 1] = args
            .try_into()
            .map_err(|args: Vec<Value>| Self::wrong_arity("load", 1, args.len()))?;
        let slot = ptr.as_ptr()?;
        self.heap
            .get(slot)
            .and_then(|value| value.clone())
            .ok_or_else(|| RuntimeError::new("N0406", format!("invalid pointer `{slot}`")))
    }

    fn builtin_dealloc(&mut self, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let [ptr]: [Value; 1] = args
            .try_into()
            .map_err(|args: Vec<Value>| Self::wrong_arity("dealloc", 1, args.len()))?;
        let slot = ptr.as_ptr()?;
        let Some(value) = self.heap.get_mut(slot) else {
            return Err(RuntimeError::new(
                "N0406",
                format!("invalid pointer `{slot}`"),
            ));
        };
        if value.take().is_none() {
            return Err(RuntimeError::new(
                "N0406",
                format!("invalid pointer `{slot}`"),
            ));
        }
        Ok(Value::Void)
    }

    fn wrong_arity(name: &str, expected: usize, actual: usize) -> RuntimeError {
        RuntimeError::new(
            "N0405",
            format!("function `{name}` expects {expected} arguments but got {actual}"),
        )
    }
}

enum Control {
    Return(Value),
    Value(Value),
}

impl Value {
    fn as_i64(&self) -> Result<i64, RuntimeError> {
        match self {
            Self::I64(value) => Ok(*value),
            _ => Err(RuntimeError::new("N0407", "expected i64 value")),
        }
    }

    fn as_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err(RuntimeError::new("N0408", "expected bool value")),
        }
    }

    fn as_ptr(&self) -> Result<usize, RuntimeError> {
        match self {
            Self::Ptr(value) => Ok(*value),
            _ => Err(RuntimeError::new("N0409", "expected pointer value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use nous_lexer::lex;
    use nous_parser::parse;
    use nous_semantics::validate;

    use super::*;

    fn run_source(source: &str) -> Result<Value, RuntimeError> {
        let tokens = lex(source).expect("lex");
        let program = parse(&tokens).expect("parse");
        validate(&program).expect("semantic");
        run_main(&program)
    }

    #[test]
    fn runs_function_calls_and_arithmetic() {
        let source = "fn add x i64 y i64 -> i64\n    x + y\n\nfn main -> i64\n    let value i64 = add(40, 2)\n    value\n";
        assert_eq!(run_source(source).expect("run"), Value::I64(42));
    }

    #[test]
    fn runs_memory_builtins() {
        let source = "fn main -> i64\n    let ptr ptr_i64 = alloc(41)\n    let value i64 = load(ptr)\n    dealloc(ptr)\n    value + 1\n";
        assert_eq!(run_source(source).expect("run"), Value::I64(42));
    }

    #[test]
    fn runs_if_expression_result() {
        let source = "fn main -> i64\n    if true\n        42\n    else\n        0\n";
        assert_eq!(run_source(source).expect("run"), Value::I64(42));
    }

    #[test]
    fn rejects_double_dealloc() {
        let source =
            "fn main -> void\n    let ptr ptr_i64 = alloc(1)\n    dealloc(ptr)\n    dealloc(ptr)\n";
        let error = run_source(source).expect_err("runtime error");
        assert_eq!(error.code, "N0406");
    }
}
