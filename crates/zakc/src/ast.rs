use crate::diagnostic::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeName,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub type_name: TypeName,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeName {
    Int, Bool, String, Unit, Unknown,
}

impl TypeName {
    pub fn from_name(name: &str) -> Self {
        match name {
            "Int" => Self::Int,
            "Bool" => Self::Bool,
            "String" => Self::String,
            "Unit" => Self::Unit,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Variable {
        name: String,
        declared_type: Option<TypeName>,
        mutable: bool,
        initializer: Expression,
        span: Span,
    },
    Assignment { name: String, value: Expression, span: Span },
    Return { value: Option<Expression>, span: Span },
    Expression { expression: Expression, span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Variable(String, Span),
    Unary { operator: UnaryOperator, operand: Box<Expression>, span: Span },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    Call { callee: String, arguments: Vec<Expression>, span: Span },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Self::Integer(_, span)
            | Self::Boolean(_, span)
            | Self::String(_, span)
            | Self::Variable(_, span) => *span,
            Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::Call { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add, Subtract, Multiply, Divide, Remainder,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or,
}
