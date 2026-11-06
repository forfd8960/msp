use msp::{
    Lexer,
    ast::{Literal, Program, Stmt},
    interpreter::{Env, Interpreter, Value},
    parser,
};

fn parse(source: &str) -> Program {
    let tokens = Lexer::new(source.chars().collect()).lex().unwrap();
    parser::new(tokens).parse().unwrap()
}

fn number(n: i64) -> Value {
    Value::Literal(Literal::Number(n))
}

fn string(s: &str) -> Value {
    Value::Literal(Literal::Str(s.into()))
}

fn call(expression: &str, expected: Value) {
    let mut setup = parse(include_str!("../docs/test.msp"));
    // Load the sample's definitions without running its demonstration calls.
    setup.nodes.retain(|node| matches!(&node.stmt, Stmt::Func(_)));
    let env = Env::new(None);
    let mut interpreter = Interpreter::new(env.clone());
    interpreter.execute(&setup).unwrap();
    interpreter
        .execute(&parse(&format!("test_result = {expression};")))
        .unwrap();
    assert_eq!(env.borrow().get("test_result"), Some(expected));
}

#[test]
fn executes_complete_test_msp() {
    let program = parse(include_str!("../docs/test.msp"));
    Interpreter::new(Env::new(None)).execute(&program).unwrap();
}

#[test]
fn recursive_fibonacci_returns_five() {
    call("fib(5)", number(5));
}

#[test]
fn large_returns_the_larger_argument() {
    call("large(10, 100)", number(100));
    call("large(100, 10)", number(100));
}

#[test]
fn is_even_returns_booleans() {
    call("is_even(4)", Value::Literal(Literal::Boolean(true)));
    call("is_even(5)", Value::Literal(Literal::Boolean(false)));
}

#[test]
fn filter_returns_multiples_of_three() {
    call("filter([1, 3, 9, 8, 100])", Value::Array(vec![number(3), number(9)]));
}

#[test]
fn names_len_returns_each_string_length() {
    call(
        "names_len([\"alice\", \"bob\", \"jane\"])",
        Value::Array(vec![number(5), number(3), number(4)]),
    );
}

#[test]
fn split_whitespace_skips_repeated_spaces() {
    call(
        "split_whitespace(\"  hello   world  \")",
        Value::Array(vec![string("hello"), string("world")]),
    );
}

#[test]
fn split_whitespace_handles_empty_input() {
    call("split_whitespace(\"\")", Value::Array(vec![]));
}

#[test]
fn append_updates_the_enclosing_array_across_iterations() {
    let env = Env::new(None);
    let mut interpreter = Interpreter::new(env.clone());
    interpreter
        .execute(&parse("items = []; for i = 0; i < 3; i++ { append(items, i); }"))
        .unwrap();
    assert_eq!(
        env.borrow().get("items"),
        Some(Value::Array(vec![number(0), number(1), number(2)]))
    );
}

#[test]
fn append_propagates_argument_errors_without_appending() {
    let env = Env::new(None);
    let mut interpreter = Interpreter::new(env.clone());
    interpreter.execute(&parse("items = [1];")).unwrap();
    assert!(interpreter.execute(&parse("append(items, 2, missing);")).is_err());
    assert_eq!(env.borrow().get("items"), Some(Value::Array(vec![number(1)])));
}

#[test]
fn character_comparisons_support_equality_and_ordering() {
    for (expression, expected) in [
        ("\"abc\"[0] == \"a\"", true),
        ("\"a\" == \"abc\"[0]", true),
        ("\"abc\"[0] == \"another\"[0]", true),
        ("\"abc\"[0] != \"abc\"[1]", true),
        ("\"abc\"[0] != \"a\"", false),
        ("\"a\" != \"abc\"[0]", false),
        ("\"abc\"[0] == \"\"", false),
        ("\"abc\"[0] == \"ab\"", false),
        ("\"abc\"[0] < \"abc\"[1]", true),
        ("\"abc\"[0] <= \"a\"", true),
        ("\"b\" > \"abc\"[0]", true),
        ("\"abc\"[1] >= \"b\"", true),
        ("\" \"[0] == \" \"", true),
    ] {
        call(expression, Value::Literal(Literal::Boolean(expected)));
    }
}
