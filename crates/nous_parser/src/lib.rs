use nous_lexer::{Diagnostic, Keyword, Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Return(Option<Expr>),
    Expr(Expr),
    If {
        branches: Vec<IfBranch>,
        else_body: Vec<Stmt>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub text: String,
    pub span: Span,
}

impl Expr {
    fn new(parts: Vec<String>, span: Span) -> Self {
        Self {
            text: parts.join(" "),
            span,
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Program, Vec<Diagnostic>> {
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();
    if parser.diagnostics.is_empty() {
        Ok(program)
    } else {
        Err(parser.diagnostics)
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();
        self.skip_newlines();

        while !self.at(TokenKindRef::Eof) {
            if self.eat_keyword(Keyword::Fn).is_some() {
                if let Some(function) = self.parse_function() {
                    functions.push(function);
                }
            } else {
                let token = self.peek();
                self.error(
                    "N0201",
                    "expected top-level function declaration",
                    token.span,
                );
                self.advance();
            }
            self.skip_newlines();
        }

        Program { functions }
    }

    fn parse_function(&mut self) -> Option<Function> {
        let fn_span = self.previous().span;
        let name = self.expect_identifier("expected function name after `fn`")?;
        let mut params = Vec::new();

        while !self.at(TokenKindRef::Arrow)
            && !self.at(TokenKindRef::Newline)
            && !self.at(TokenKindRef::Eof)
        {
            let param_name = self.expect_identifier("expected parameter name")?;
            let ty = self.expect_type("expected parameter type")?;
            params.push(Param {
                name: param_name,
                ty,
            });
        }

        if !self.eat(TokenKindRef::Arrow) {
            self.error(
                "N0202",
                "expected `->` before function return type",
                self.peek().span,
            );
            return None;
        }

        let return_type = self.expect_type("expected function return type after `->`")?;
        self.expect_newline("expected newline after function signature");
        self.expect(TokenKindRef::Indent, "expected indented function body")?;
        let body = self.parse_block(&[BlockEnd::Dedent]);
        self.expect(TokenKindRef::Dedent, "expected function body dedent")?;

        Some(Function {
            name,
            params,
            return_type,
            body,
            span: fn_span,
        })
    }

    fn parse_block(&mut self, ends: &[BlockEnd]) -> Vec<Stmt> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.at(TokenKindRef::Eof) && !self.is_block_end(ends) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.advance();
            }
            self.skip_newlines();
        }

        statements
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        if self.eat_keyword(Keyword::Return).is_some() {
            let span = self.previous().span;
            let expr = if self.at(TokenKindRef::Newline) {
                None
            } else {
                Some(self.parse_expr_until_newline(span))
            };
            self.expect_newline("expected newline after return statement");
            return Some(Stmt::Return(expr));
        }

        if self.eat_keyword(Keyword::If).is_some() {
            return self.parse_if();
        }

        let span = self.peek().span;
        let expr = self.parse_expr_until_newline(span);
        self.expect_newline("expected newline after expression");
        Some(Stmt::Expr(expr))
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        let span = self.previous().span;
        let first_condition = self.parse_expr_until_newline(span);
        self.expect_newline("expected newline after if condition");
        self.expect(TokenKindRef::Indent, "expected indented if body")?;
        let first_body = self.parse_block(&[BlockEnd::Dedent]);
        self.expect(TokenKindRef::Dedent, "expected if body dedent")?;

        let mut branches = vec![IfBranch {
            condition: first_condition,
            body: first_body,
        }];
        let mut else_body = Vec::new();

        loop {
            self.skip_newlines();
            if self.eat_keyword(Keyword::Elif).is_some() {
                let branch_span = self.previous().span;
                let condition = self.parse_expr_until_newline(branch_span);
                self.expect_newline("expected newline after elif condition");
                self.expect(TokenKindRef::Indent, "expected indented elif body")?;
                let body = self.parse_block(&[BlockEnd::Dedent]);
                self.expect(TokenKindRef::Dedent, "expected elif body dedent")?;
                branches.push(IfBranch { condition, body });
                continue;
            }

            if self.eat_keyword(Keyword::Else).is_some() {
                self.expect_newline("expected newline after else");
                self.expect(TokenKindRef::Indent, "expected indented else body")?;
                else_body = self.parse_block(&[BlockEnd::Dedent]);
                self.expect(TokenKindRef::Dedent, "expected else body dedent")?;
            }
            break;
        }

        Some(Stmt::If {
            branches,
            else_body,
            span,
        })
    }

    fn parse_expr_until_newline(&mut self, span: Span) -> Expr {
        let mut parts = Vec::new();
        while !self.at(TokenKindRef::Newline)
            && !self.at(TokenKindRef::Indent)
            && !self.at(TokenKindRef::Dedent)
            && !self.at(TokenKindRef::Eof)
        {
            parts.push(token_text(&self.peek().kind));
            self.advance();
        }
        Expr::new(parts, span)
    }

    fn expect_type(&mut self, message: &'static str) -> Option<TypeRef> {
        match &self.peek().kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Some(TypeRef { name })
            }
            TokenKind::Keyword(Keyword::Void) => {
                self.advance();
                Some(TypeRef {
                    name: "void".to_string(),
                })
            }
            _ => {
                self.error("N0203", message, self.peek().span);
                None
            }
        }
    }

    fn expect_identifier(&mut self, message: &'static str) -> Option<String> {
        match &self.peek().kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Some(name)
            }
            _ => {
                self.error("N0204", message, self.peek().span);
                None
            }
        }
    }

    fn expect_newline(&mut self, message: &'static str) -> bool {
        self.expect(TokenKindRef::Newline, message).is_some()
    }

    fn expect(&mut self, kind: TokenKindRef, message: &'static str) -> Option<Token> {
        if self.eat(kind) {
            Some(self.previous().clone())
        } else {
            self.error("N0205", message, self.peek().span);
            None
        }
    }

    fn eat(&mut self, kind: TokenKindRef) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        if matches!(&self.peek().kind, TokenKind::Keyword(actual) if *actual == keyword) {
            self.advance();
            Some(self.previous().clone())
        } else {
            None
        }
    }

    fn skip_newlines(&mut self) {
        while self.at(TokenKindRef::Newline) {
            self.advance();
        }
    }

    fn at(&self, kind: TokenKindRef) -> bool {
        matches!(
            (kind, &self.peek().kind),
            (TokenKindRef::Arrow, TokenKind::Arrow)
                | (TokenKindRef::Newline, TokenKind::Newline)
                | (TokenKindRef::Indent, TokenKind::Indent)
                | (TokenKindRef::Dedent, TokenKind::Dedent)
                | (TokenKindRef::Eof, TokenKind::Eof)
        )
    }

    fn is_block_end(&self, ends: &[BlockEnd]) -> bool {
        ends.iter().any(|end| match end {
            BlockEnd::Dedent => self.at(TokenKindRef::Dedent),
        })
    }

    fn advance(&mut self) {
        if self.cursor < self.tokens.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    fn peek(&self) -> &'a Token {
        &self.tokens[self.cursor]
    }

    fn previous(&self) -> &'a Token {
        &self.tokens[self.cursor.saturating_sub(1)]
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.diagnostics.push(Diagnostic::new(code, message, span));
    }
}

#[derive(Debug, Clone, Copy)]
enum TokenKindRef {
    Arrow,
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone, Copy)]
enum BlockEnd {
    Dedent,
}

fn token_text(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(value)
        | TokenKind::Number(value)
        | TokenKind::String(value)
        | TokenKind::Symbol(value) => value.clone(),
        TokenKind::Keyword(keyword) => keyword_text(*keyword).to_string(),
        TokenKind::Arrow => "->".to_string(),
        TokenKind::Newline => "\\n".to_string(),
        TokenKind::Indent => "<indent>".to_string(),
        TokenKind::Dedent => "<dedent>".to_string(),
        TokenKind::Eof => "<eof>".to_string(),
    }
}

fn keyword_text(keyword: Keyword) -> &'static str {
    match keyword {
        Keyword::Fn => "fn",
        Keyword::Return => "return",
        Keyword::If => "if",
        Keyword::Elif => "elif",
        Keyword::Else => "else",
        Keyword::For => "for",
        Keyword::While => "while",
        Keyword::Loop => "loop",
        Keyword::Break => "break",
        Keyword::Continue => "continue",
        Keyword::True => "true",
        Keyword::False => "false",
        Keyword::Void => "void",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nous_lexer::lex;

    #[test]
    fn parses_function_with_expression_return() {
        let tokens = lex("fn add x i64 y i64 -> i64\n    x + y\n").expect("lex");
        let program = parse(&tokens).expect("parse");
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "add");
        assert_eq!(program.functions[0].params.len(), 2);
        assert_eq!(program.functions[0].return_type.name, "i64");
    }

    #[test]
    fn parses_void_function() {
        let tokens = lex("fn main -> void\n    return\n").expect("lex");
        let program = parse(&tokens).expect("parse");
        assert_eq!(program.functions[0].return_type.name, "void");
    }

    #[test]
    fn requires_indented_function_body() {
        let tokens = lex("fn main -> void\nreturn\n").expect("lex");
        let diagnostics = parse(&tokens).expect_err("parse should fail");
        assert_eq!(diagnostics[0].code, "N0205");
    }
}
