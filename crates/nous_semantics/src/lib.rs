use nous_parser::{Function, Program, Stmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub function: Option<String>,
}

impl SemanticDiagnostic {
    fn new(code: &'static str, message: impl Into<String>, function: Option<String>) -> Self {
        Self {
            code,
            message: message.into(),
            function,
        }
    }
}

pub fn validate(program: &Program) -> Result<(), Vec<SemanticDiagnostic>> {
    let mut diagnostics = Vec::new();

    for function in &program.functions {
        validate_function(function, &mut diagnostics);
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_function(function: &Function, diagnostics: &mut Vec<SemanticDiagnostic>) {
    if function.return_type.name == "void" {
        return;
    }

    if !block_returns_value(&function.body) {
        diagnostics.push(SemanticDiagnostic::new(
            "N0301",
            format!(
                "function `{}` declares `{}` but has no final return value",
                function.name, function.return_type.name
            ),
            Some(function.name.clone()),
        ));
    }
}

fn block_returns_value(statements: &[Stmt]) -> bool {
    let Some(last) = statements.last() else {
        return false;
    };

    match last {
        Stmt::Return(Some(_)) | Stmt::Expr(_) => true,
        Stmt::Return(None) => false,
        Stmt::If {
            branches,
            else_body,
            ..
        } => {
            !else_body.is_empty()
                && branches
                    .iter()
                    .all(|branch| block_returns_value(&branch.body))
                && block_returns_value(else_body)
        }
    }
}

#[cfg(test)]
mod tests {
    use nous_lexer::lex;
    use nous_parser::parse;

    use super::*;

    #[test]
    fn non_void_function_may_return_last_expression() {
        let tokens = lex("fn add x i64 y i64 -> i64\n    x + y\n").expect("lex");
        let program = parse(&tokens).expect("parse");
        assert!(validate(&program).is_ok());
    }

    #[test]
    fn non_void_function_rejects_empty_return() {
        let tokens = lex("fn bad -> i64\n    return\n").expect("lex");
        let program = parse(&tokens).expect("parse");
        let diagnostics = validate(&program).expect_err("semantic error");
        assert_eq!(diagnostics[0].code, "N0301");
    }
}
