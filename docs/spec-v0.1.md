# ZAK Language Specification — 0.1

**Status:** Initial implementation specification  
**Version:** 0.1  
**Stability:** Experimental; source compatibility is not guaranteed.

This document describes the language subset implemented by the first compiler milestone. It is intentionally narrower than the long-term ZAK language.

## 1. Design goals

ZAK is a statically typed, native-first language intended for application development and general-purpose programming.

The initial language design prioritizes:

- A small grammar that can be implemented and tested rigorously.
- Explicit mutability.
- Static type checking.
- Predictable compilation to LLVM IR.
- Useful source-level diagnostics.
- A future path to ownership, borrowing, concurrency and native platform APIs.

## 2. Source files and programs

The source-file extension is .zak. A ZAK 0.1 program contains exactly one main function. Additional user-defined functions are reserved for a later version.

~~~zak
fn main() -> Int {
    print("Hello from ZAK");
    return 0;
}
~~~

The return type may be omitted, in which case it defaults to Unit.

~~~zak
fn main() {
    print("Hello from ZAK");
}
~~~

main may return Int or Unit. An omitted return at the end of main produces process exit code zero.

## 3. Lexical structure

### 3.1 Identifiers

Identifiers begin with an ASCII letter or underscore and may contain ASCII letters, digits and underscores.

Examples: count, _ready, user_name2.

### 3.2 Comments

Line comments begin with // and continue to the end of the line.

### 3.3 Literals

ZAK 0.1 supports:

- Signed 64-bit integer literals, written as decimal digit sequences.
- Boolean literals: true and false.
- String literals delimited by double quotes.

String escape sequences: \\n, \\r, \\t, \\" and \\\\.

### 3.4 Keywords

fn, let, var, return, true, false.

### 3.5 Operators

Arithmetic: +, -, *, /, %  
Comparison: <, <=, >, >=  
Equality: ==, !=  
Logical: &&, ||, !  
Assignment: =

Operator precedence, from highest to lowest:

1. Unary - and !
2. *, /, %
3. +, -
4. <, <=, >, >=
5. ==, !=
6. &&
7. ||

Binary operators are left-associative.

## 4. Types

| ZAK type | Meaning | LLVM representation |
|---|---|---|
| Int | Signed 64-bit integer | i64 |
| Bool | Boolean value | i1 |
| String | UTF-8 string literal or string pointer | ptr |
| Unit | Absence of a value | void |

The compiler infers a local variable's type when no annotation is supplied.

~~~zak
let attempts: Int = 3;
let ready: Bool = true;
let message: String = "Ready";
~~~

A variable cannot have type Unit.

## 5. Variables and mutability

let declares an immutable local variable. var declares a mutable local variable.

~~~zak
let initial: Int = 10;
var current: Int = initial;
current = current + 1;
~~~

Assignment to a let variable is a compile-time error. Variables must be declared before they are read or assigned. Variable redeclaration in the same function body is an error.

## 6. Statements

### 6.1 Variable declaration

~~~ebnf
variable-declaration = ("let" | "var"), identifier,
                       [":", type], "=", expression, ";";
~~~

### 6.2 Assignment

~~~ebnf
assignment = identifier, "=", expression, ";";
~~~

Only mutable variables may be assigned.

### 6.3 Print

print is a built-in function that accepts exactly one Int, Bool or String argument.

~~~zak
print("Hello");
print(42);
print(true);
~~~

It writes a newline after the value.

### 6.4 Return

~~~zak
return 0;
return;
~~~

An Int-returning main must return an integer expression when it uses an explicit return. A Unit-returning main uses return;.

## 7. Expressions

Supported expressions include:

- Integer, boolean and string literals.
- Local variable references.
- Parenthesized expressions.
- Unary negation and logical not.
- Integer arithmetic.
- Integer comparisons.
- Integer and boolean equality.
- Boolean conjunction and disjunction.
- The built-in print(...) call as a statement.

Examples:

~~~zak
let total = 20 + 22;
let valid = total >= 42 && true;
let negative = -total;
~~~

Arithmetic operations require Int operands. Relational operators require Int operands. Equality supports matching Int or Bool operands. Logical operators require Bool operands.

## 8. Functions

The parser recognizes function declarations and typed parameters, but ZAK 0.1's semantic checker accepts only a parameterless main function. Additional function declarations and user-defined function calls are reserved for a later release.

## 9. Compilation model

The initial compilation pipeline is:

1. Lex source into tokens.
2. Parse tokens into an abstract syntax tree.
3. Resolve and validate the supported program shape.
4. Type-check expressions and statements.
5. Lower the accepted program to textual LLVM IR.
6. Invoke Clang to compile and link the LLVM IR for the host platform.

The generated module uses LLVM opaque pointers. LLVM pointer values do not carry ZAK source-level type information; the ZAK frontend retains and checks those types before code generation.

## 10. Explicit non-goals for 0.1

The following are not implemented in this milestone:

- User-defined function calls.
- Conditional and loop statements.
- Modules and package imports.
- Structs, enums, generics and traits.
- Ownership and borrowing.
- Async/await and concurrency.
- Floating-point types.
- Dynamic strings and a general memory-managed object model.
- Native UI or device APIs.
- Android, iOS, macOS, Windows and Linux cross-compilation.

These are planned language and runtime features, not current compiler capabilities.

## 11. Compatibility

ZAK 0.1 is experimental. Grammar, type rules and compiler output may change. The project will use RFCs and explicit language-version notes before declaring a stable language release.
