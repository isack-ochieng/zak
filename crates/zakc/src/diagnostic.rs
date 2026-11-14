use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub phase: &'static str,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(phase: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self { phase, message: message.into(), span }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}] at {}:{}: {}", self.phase, self.span.line, self.span.column, self.message)
    }
}

impl std::error::Error for Diagnostic {}
