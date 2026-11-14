pub mod ast;
pub mod checker;
pub mod codegen;
pub mod diagnostic;
pub mod lexer;
pub mod parser;

use ast::Program;
use diagnostic::Diagnostic;
use lexer::lex;
use parser::Parser;

pub fn parse_source(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let tokens = lex(source).map_err(|error| vec![error])?;
    Parser::new(tokens).parse_program().map_err(|error| vec![error])
}

pub fn check_source(source: &str) -> Result<(), Vec<Diagnostic>> {
    let program = parse_source(source)?;
    checker::check_program(&program).map_err(|error| vec![error])
}

pub fn compile_source(source: &str) -> Result<String, Vec<Diagnostic>> {
    let program = parse_source(source)?;
    checker::check_program(&program).map_err(|error| vec![error])?;
    codegen::generate(&program).map_err(|error| vec![error])
}

#[cfg(test)]
mod tests {
    use super::{check_source, compile_source};

    #[test]
    fn compiles_a_minimal_program_to_llvm_ir() {
        let source = r#"
            fn main() -> Int {
                print("Hello from ZAK");
                let answer: Int = 40 + 2;
                print(answer);
                return 0;
            }
        "#;
        let ir = compile_source(source).expect("program should compile");
        assert!(ir.contains("define i32 @main()"));
        assert!(ir.contains("add i64 40, 2"));
        assert!(ir.contains("@puts"));
        assert!(ir.contains("@printf"));
    }

    #[test]
    fn rejects_assignment_to_let() {
        let source = r#"
            fn main() {
                let answer = 42;
                answer = 43;
            }
        "#;
        let errors = check_source(source).expect_err("immutable assignment must fail");
        assert!(errors[0].message.contains("immutable"));
    }

    #[test]
    fn rejects_mismatched_types() {
        let source = r#"
            fn main() {
                let answer: Int = true;
            }
        "#;
        let errors = check_source(source).expect_err("type mismatch must fail");
        assert!(errors[0].message.contains("initializer has type"));
    }
}
