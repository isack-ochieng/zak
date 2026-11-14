use std::collections::HashMap;

use crate::ast::{BinaryOperator, Expression, Program, Statement, TypeName, UnaryOperator};
use crate::diagnostic::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type { Int, Bool, String, Unit }

impl Type {
    fn from_name(name: TypeName) -> Self {
        match name {
            TypeName::Int => Self::Int,
            TypeName::Bool => Self::Bool,
            TypeName::String => Self::String,
            TypeName::Unit | TypeName::Unknown => Self::Unit,
        }
    }

    pub fn llvm_type(self) -> &'static str {
        match self {
            Self::Int => "i64",
            Self::Bool => "i1",
            Self::String => "ptr",
            Self::Unit => "void",
        }
    }
}

#[derive(Debug, Clone)]
struct Variable { ty: Type, mutable: bool }

pub fn check_program(program: &Program) -> Result<(), Diagnostic> {
    let mains: Vec<_> = program.functions.iter().filter(|function| function.name == "main").collect();
    if mains.len() != 1 {
        return Err(Diagnostic::new("type", "a ZAK program must declare exactly one 'main' function", program.functions[0].span));
    }
    if program.functions.len() != 1 {
        let extra = program.functions.iter().find(|function| function.name != "main");
        let span = extra.map(|function| function.span).unwrap_or(program.functions[0].span);
        return Err(Diagnostic::new("type", "additional user-defined functions are not supported in ZAK 0.1", span));
    }

    let main = mains[0];
    if !main.parameters.is_empty() {
        return Err(Diagnostic::new("type", "main() cannot accept parameters in ZAK 0.1", main.parameters[0].span));
    }
    if !matches!(main.return_type, TypeName::Int | TypeName::Unit) {
        return Err(Diagnostic::new("type", "main must return Int or Unit", main.span));
    }

    let mut variables = HashMap::new();
    for statement in &main.body {
        check_statement(statement, Type::from_name(main.return_type), &mut variables)?;
    }
    Ok(())
}

fn check_statement(statement: &Statement, return_type: Type, variables: &mut HashMap<String, Variable>) -> Result<(), Diagnostic> {
    match statement {
        Statement::Variable { name, declared_type, mutable, initializer, span } => {
            if variables.contains_key(name) {
                return Err(Diagnostic::new("type", format!("variable '{}' is already declared in this scope", name), *span));
            }
            let actual = check_expression(initializer, variables)?;
            let ty = declared_type.map(Type::from_name).unwrap_or(actual);
            if ty == Type::Unit {
                return Err(Diagnostic::new("type", "variables cannot have type Unit", *span));
            }
            if ty != actual {
                return Err(Diagnostic::new("type", format!("initializer has type {:?}, but '{}' is declared as {:?}", actual, name, ty), initializer.span()));
            }
            variables.insert(name.clone(), Variable { ty, mutable: *mutable });
            Ok(())
        }
        Statement::Assignment { name, value, span } => {
            let variable = variables.get(name).cloned().ok_or_else(|| Diagnostic::new("type", format!("unknown variable '{}'", name), *span))?;
            if !variable.mutable {
                return Err(Diagnostic::new("type", format!("cannot assign to immutable variable '{}'; declare it with 'var'", name), *span));
            }
            let value_type = check_expression(value, variables)?;
            if variable.ty != value_type {
                return Err(Diagnostic::new("type", format!("cannot assign {:?} to variable '{}' of type {:?}", value_type, name, variable.ty), value.span()));
            }
            Ok(())
        }
        Statement::Return { value, span } => match (return_type, value) {
            (Type::Unit, None) => Ok(()),
            (Type::Unit, Some(expression)) => Err(Diagnostic::new("type", "a Unit function cannot return a value", expression.span())),
            (Type::Int, Some(expression)) => {
                let actual = check_expression(expression, variables)?;
                if actual == Type::Int { Ok(()) }
                else { Err(Diagnostic::new("type", format!("main returns Int, but this expression has type {:?}", actual), expression.span())) }
            }
            (Type::Int, None) => Err(Diagnostic::new("type", "main returns Int, so return a value such as 'return 0;'", *span)),
            _ => Ok(()),
        },
        Statement::Expression { expression, .. } => match expression {
            Expression::Call { callee, .. } if callee == "print" => {
                let result = check_expression(expression, variables)?;
                if result == Type::Unit { Ok(()) }
                else { Err(Diagnostic::new("type", "print must be used as a statement", expression.span())) }
            }
            _ => Err(Diagnostic::new("type", "only the built-in print(...) call can be used as an expression statement in ZAK 0.1", expression.span())),
        },
    }
}

fn check_expression(expression: &Expression, variables: &HashMap<String, Variable>) -> Result<Type, Diagnostic> {
    match expression {
        Expression::Integer(_, _) => Ok(Type::Int),
        Expression::Boolean(_, _) => Ok(Type::Bool),
        Expression::String(_, _) => Ok(Type::String),
        Expression::Variable(name, span) => variables.get(name).map(|variable| variable.ty)
            .ok_or_else(|| Diagnostic::new("type", format!("unknown variable '{}'", name), *span)),
        Expression::Unary { operator, operand, span } => {
            let operand_type = check_expression(operand, variables)?;
            match (operator, operand_type) {
                (UnaryOperator::Negate, Type::Int) => Ok(Type::Int),
                (UnaryOperator::Not, Type::Bool) => Ok(Type::Bool),
                (UnaryOperator::Negate, other) => Err(Diagnostic::new("type", format!("unary '-' expects Int, found {:?}", other), *span)),
                (UnaryOperator::Not, other) => Err(Diagnostic::new("type", format!("'!' expects Bool, found {:?}", other), *span)),
            }
        }
        Expression::Binary { left, operator, right, span } => {
            let left_type = check_expression(left, variables)?;
            let right_type = check_expression(right, variables)?;
            use BinaryOperator::*;
            match operator {
                Add | Subtract | Multiply | Divide | Remainder => {
                    require_same(left_type, right_type, *span)?;
                    if left_type == Type::Int { Ok(Type::Int) }
                    else { Err(Diagnostic::new("type", "arithmetic operators currently require Int operands", *span)) }
                }
                Less | LessEqual | Greater | GreaterEqual => {
                    if left_type == Type::Int && right_type == Type::Int { Ok(Type::Bool) }
                    else { Err(Diagnostic::new("type", "comparison operators require Int operands", *span)) }
                }
                Equal | NotEqual => {
                    require_same(left_type, right_type, *span)?;
                    if matches!(left_type, Type::Int | Type::Bool) { Ok(Type::Bool) }
                    else { Err(Diagnostic::new("type", "equality is currently supported for Int and Bool values", *span)) }
                }
                And | Or => {
                    if left_type == Type::Bool && right_type == Type::Bool { Ok(Type::Bool) }
                    else { Err(Diagnostic::new("type", "logical operators require Bool operands", *span)) }
                }
            }
        }
        Expression::Call { callee, arguments, span } => {
            if callee != "print" {
                return Err(Diagnostic::new("type", format!("unknown function '{}'; only print(...) is available in ZAK 0.1", callee), *span));
            }
            if arguments.len() != 1 {
                return Err(Diagnostic::new("type", "print expects exactly one argument", *span));
            }
            let argument_type = check_expression(&arguments[0], variables)?;
            if !matches!(argument_type, Type::Int | Type::Bool | Type::String) {
                return Err(Diagnostic::new("type", "print supports Int, Bool, and String values", arguments[0].span()));
            }
            Ok(Type::Unit)
        }
    }
}

fn require_same(left: Type, right: Type, span: Span) -> Result<(), Diagnostic> {
    if left == right { Ok(()) }
    else {
        Err(Diagnostic::new("type", format!("type mismatch: left operand is {:?}, right operand is {:?}", left, right), span))
    }
}
