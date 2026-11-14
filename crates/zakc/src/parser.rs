use std::mem::discriminant;

use crate::ast::{
    BinaryOperator, Expression, Function, Parameter, Program, Statement, TypeName, UnaryOperator,
};
use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut functions = Vec::new();
        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }
        if functions.is_empty() {
            return Err(Diagnostic::new("parser", "expected at least one function declaration", self.peek().span));
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, Diagnostic> {
        let start = self.peek().span;
        self.expect_simple(TokenKind::Fn, "expected 'fn'")?;
        let name = self.expect_identifier("expected a function name")?;
        self.expect_simple(TokenKind::LeftParen, "expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check_simple(&TokenKind::RightParen) {
            loop {
                let span = self.peek().span;
                let parameter_name = self.expect_identifier("expected parameter name")?;
                self.expect_simple(TokenKind::Colon, "expected ':' after parameter name")?;
                let type_name = self.parse_type()?;
                parameters.push(Parameter { name: parameter_name, type_name, span });
                if !self.matches_simple(&TokenKind::Comma) { break; }
            }
        }
        self.expect_simple(TokenKind::RightParen, "expected ')' after parameters")?;

        let return_type = if self.matches_simple(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            TypeName::Unit
        };
        let body = self.parse_block()?;
        Ok(Function { name, parameters, return_type, body, span: start })
    }

    fn parse_type(&mut self) -> Result<TypeName, Diagnostic> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => {
                let ty = TypeName::from_name(&name);
                if ty == TypeName::Unknown {
                    Err(Diagnostic::new("parser", format!("unknown type '{}'; expected Int, Bool, String, or Unit", name), token.span))
                } else {
                    Ok(ty)
                }
            }
            _ => Err(Diagnostic::new("parser", "expected a type name", token.span)),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        self.expect_simple(TokenKind::LeftBrace, "expected '{' to begin function body")?;
        let mut statements = Vec::new();
        while !self.check_simple(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        self.expect_simple(TokenKind::RightBrace, "expected '}' after function body")?;
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, Diagnostic> {
        if self.matches_simple(&TokenKind::Let) { return self.parse_variable(false); }
        if self.matches_simple(&TokenKind::Var) { return self.parse_variable(true); }

        if self.matches_simple(&TokenKind::Return) {
            let span = self.previous().span;
            let value = if self.check_simple(&TokenKind::Semicolon) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            self.expect_simple(TokenKind::Semicolon, "expected ';' after return")?;
            return Ok(Statement::Return { value, span });
        }

        let span = self.peek().span;
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if self.check_next_simple(&TokenKind::Equal) {
                let name = name.clone();
                self.advance();
                self.advance();
                let value = self.parse_expression()?;
                self.expect_simple(TokenKind::Semicolon, "expected ';' after assignment")?;
                return Ok(Statement::Assignment { name, value, span });
            }
        }

        let expression = self.parse_expression()?;
        self.expect_simple(TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Statement::Expression { expression, span })
    }

    fn parse_variable(&mut self, mutable: bool) -> Result<Statement, Diagnostic> {
        let span = self.previous().span;
        let name = self.expect_identifier("expected variable name")?;
        let declared_type = if self.matches_simple(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_simple(TokenKind::Equal, "expected '=' before variable initializer")?;
        let initializer = self.parse_expression()?;
        self.expect_simple(TokenKind::Semicolon, "expected ';' after variable declaration")?;
        Ok(Statement::Variable { name, declared_type, mutable, initializer, span })
    }

    fn parse_expression(&mut self) -> Result<Expression, Diagnostic> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_and()?;
        while self.matches_simple(&TokenKind::OrOr) {
            let span = expression.span();
            let right = self.parse_and()?;
            expression = Expression::Binary { left: Box::new(expression), operator: BinaryOperator::Or, right: Box::new(right), span };
        }
        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_equality()?;
        while self.matches_simple(&TokenKind::AndAnd) {
            let span = expression.span();
            let right = self.parse_equality()?;
            expression = Expression::Binary { left: Box::new(expression), operator: BinaryOperator::And, right: Box::new(right), span };
        }
        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_comparison()?;
        loop {
            let operator = if self.matches_simple(&TokenKind::EqualEqual) {
                Some(BinaryOperator::Equal)
            } else if self.matches_simple(&TokenKind::BangEqual) {
                Some(BinaryOperator::NotEqual)
            } else { None };
            let Some(operator) = operator else { break };
            let span = expression.span();
            let right = self.parse_comparison()?;
            expression = Expression::Binary { left: Box::new(expression), operator, right: Box::new(right), span };
        }
        Ok(expression)
    }

    fn parse_comparison(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_term()?;
        loop {
            let operator = if self.matches_simple(&TokenKind::Less) {
                Some(BinaryOperator::Less)
            } else if self.matches_simple(&TokenKind::LessEqual) {
                Some(BinaryOperator::LessEqual)
            } else if self.matches_simple(&TokenKind::Greater) {
                Some(BinaryOperator::Greater)
            } else if self.matches_simple(&TokenKind::GreaterEqual) {
                Some(BinaryOperator::GreaterEqual)
            } else { None };
            let Some(operator) = operator else { break };
            let span = expression.span();
            let right = self.parse_term()?;
            expression = Expression::Binary { left: Box::new(expression), operator, right: Box::new(right), span };
        }
        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_factor()?;
        loop {
            let operator = if self.matches_simple(&TokenKind::Plus) {
                Some(BinaryOperator::Add)
            } else if self.matches_simple(&TokenKind::Minus) {
                Some(BinaryOperator::Subtract)
            } else { None };
            let Some(operator) = operator else { break };
            let span = expression.span();
            let right = self.parse_factor()?;
            expression = Expression::Binary { left: Box::new(expression), operator, right: Box::new(right), span };
        }
        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expression, Diagnostic> {
        let mut expression = self.parse_unary()?;
        loop {
            let operator = if self.matches_simple(&TokenKind::Star) {
                Some(BinaryOperator::Multiply)
            } else if self.matches_simple(&TokenKind::Slash) {
                Some(BinaryOperator::Divide)
            } else if self.matches_simple(&TokenKind::Percent) {
                Some(BinaryOperator::Remainder)
            } else { None };
            let Some(operator) = operator else { break };
            let span = expression.span();
            let right = self.parse_unary()?;
            expression = Expression::Binary { left: Box::new(expression), operator, right: Box::new(right), span };
        }
        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, Diagnostic> {
        if self.matches_simple(&TokenKind::Minus) {
            let span = self.previous().span;
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary { operator: UnaryOperator::Negate, operand: Box::new(operand), span });
        }
        if self.matches_simple(&TokenKind::Bang) {
            let span = self.previous().span;
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary { operator: UnaryOperator::Not, operand: Box::new(operand), span });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, Diagnostic> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Integer(value) => Ok(Expression::Integer(value, token.span)),
            TokenKind::StringLiteral(value) => Ok(Expression::String(value, token.span)),
            TokenKind::True => Ok(Expression::Boolean(true, token.span)),
            TokenKind::False => Ok(Expression::Boolean(false, token.span)),
            TokenKind::Identifier(name) => {
                if self.matches_simple(&TokenKind::LeftParen) {
                    let mut arguments = Vec::new();
                    if !self.check_simple(&TokenKind::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.matches_simple(&TokenKind::Comma) { break; }
                        }
                    }
                    self.expect_simple(TokenKind::RightParen, "expected ')' after arguments")?;
                    Ok(Expression::Call { callee: name, arguments, span: token.span })
                } else {
                    Ok(Expression::Variable(name, token.span))
                }
            }
            TokenKind::LeftParen => {
                let expression = self.parse_expression()?;
                self.expect_simple(TokenKind::RightParen, "expected ')' after expression")?;
                Ok(expression)
            }
            _ => Err(Diagnostic::new("parser", "expected an expression", token.span)),
        }
    }

    fn expect_identifier(&mut self, message: &str) -> Result<String, Diagnostic> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => Ok(name),
            _ => Err(Diagnostic::new("parser", message, token.span)),
        }
    }

    fn expect_simple(&mut self, expected: TokenKind, message: &str) -> Result<(), Diagnostic> {
        if self.matches_simple(&expected) { Ok(()) }
        else { Err(Diagnostic::new("parser", message, self.peek().span)) }
    }

    fn matches_simple(&mut self, expected: &TokenKind) -> bool {
        if self.check_simple(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_simple(&self, expected: &TokenKind) -> bool {
        discriminant(&self.peek().kind) == discriminant(expected)
    }

    fn check_next_simple(&self, expected: &TokenKind) -> bool {
        self.tokens.get(self.current + 1)
            .map(|token| discriminant(&token.kind) == discriminant(expected))
            .unwrap_or(false)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current.min(self.tokens.len() - 1)]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous()
    }
}
