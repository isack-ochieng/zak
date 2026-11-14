use crate::diagnostic::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    StringLiteral(String),
    Fn, Let, Var, Return, True, False,
    LeftParen, RightParen, LeftBrace, RightBrace,
    Colon, Semicolon, Comma, Arrow,
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, Bang, BangEqual,
    Less, LessEqual, Greater, GreaterEqual,
    AndAnd, OrOr, Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

struct Scanner {
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    start_line: usize,
    start_column: usize,
    tokens: Vec<Token>,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut scanner = Scanner {
        chars: source.chars().collect(),
        current: 0,
        line: 1,
        column: 1,
        start_line: 1,
        start_column: 1,
        tokens: Vec::new(),
    };

    while !scanner.is_at_end() {
        scanner.start_line = scanner.line;
        scanner.start_column = scanner.column;
        scanner.scan_token()?;
    }

    scanner.tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(scanner.line, scanner.column),
    });
    Ok(scanner.tokens)
}

impl Scanner {
    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    fn peek(&self) -> char {
        self.chars.get(self.current).copied().unwrap_or('\0')
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.peek() != expected {
            return false;
        }
        self.advance();
        true
    }

    fn span(&self) -> Span {
        Span::new(self.start_line, self.start_column)
    }

    fn push(&mut self, kind: TokenKind) {
        self.tokens.push(Token { kind, span: self.span() });
    }

    fn scan_token(&mut self) -> Result<(), Diagnostic> {
        let ch = self.advance();
        match ch {
            ' ' | '\r' | '\t' | '\n' => {}
            '(' => self.push(TokenKind::LeftParen),
            ')' => self.push(TokenKind::RightParen),
            '{' => self.push(TokenKind::LeftBrace),
            '}' => self.push(TokenKind::RightBrace),
            ':' => self.push(TokenKind::Colon),
            ';' => self.push(TokenKind::Semicolon),
            ',' => self.push(TokenKind::Comma),
            '+' => self.push(TokenKind::Plus),
            '*' => self.push(TokenKind::Star),
            '%' => self.push(TokenKind::Percent),
            '-' => {
                if self.matches('>') { self.push(TokenKind::Arrow); }
                else { self.push(TokenKind::Minus); }
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() { self.advance(); }
                } else {
                    self.push(TokenKind::Slash);
                }
            }
            '=' => {
                if self.matches('=') { self.push(TokenKind::EqualEqual); }
                else { self.push(TokenKind::Equal); }
            }
            '!' => {
                if self.matches('=') { self.push(TokenKind::BangEqual); }
                else { self.push(TokenKind::Bang); }
            }
            '<' => {
                if self.matches('=') { self.push(TokenKind::LessEqual); }
                else { self.push(TokenKind::Less); }
            }
            '>' => {
                if self.matches('=') { self.push(TokenKind::GreaterEqual); }
                else { self.push(TokenKind::Greater); }
            }
            '&' => {
                if self.matches('&') { self.push(TokenKind::AndAnd); }
                else {
                    return Err(Diagnostic::new("lexer", "expected a second '&' to form '&&'", self.span()));
                }
            }
            '|' => {
                if self.matches('|') { self.push(TokenKind::OrOr); }
                else {
                    return Err(Diagnostic::new("lexer", "expected a second '|' to form '||'", self.span()));
                }
            }
            '"' => self.scan_string()?,
            c if c.is_ascii_digit() => self.scan_integer(c)?,
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier(c),
            _ => return Err(Diagnostic::new("lexer", format!("unexpected character '{}'", ch), self.span())),
        }
        Ok(())
    }

    fn scan_integer(&mut self, first: char) -> Result<(), Diagnostic> {
        let mut text = String::from(first);
        while self.peek().is_ascii_digit() {
            text.push(self.advance());
        }
        let value = text.parse::<i64>().map_err(|_| {
            Diagnostic::new("lexer", "integer literal is outside the supported 64-bit range", self.span())
        })?;
        self.push(TokenKind::Integer(value));
        Ok(())
    }

    fn scan_identifier(&mut self, first: char) {
        let mut text = String::from(first);
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            text.push(self.advance());
        }
        let kind = match text.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text),
        };
        self.push(kind);
    }

    fn scan_string(&mut self) -> Result<(), Diagnostic> {
        let mut value = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\n' {
                return Err(Diagnostic::new("lexer", "newline inside a string literal; use '\\n' instead", self.span()));
            }
            if ch == '\\' {
                if self.is_at_end() { break; }
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    other => {
                        return Err(Diagnostic::new(
                            "lexer",
                            format!("unsupported escape sequence '\\{}'", other),
                            self.span(),
                        ));
                    }
                }
            } else {
                value.push(ch);
            }
        }
        if self.is_at_end() {
            return Err(Diagnostic::new("lexer", "unterminated string literal", self.span()));
        }
        self.advance();
        self.push(TokenKind::StringLiteral(value));
        Ok(())
    }
}
