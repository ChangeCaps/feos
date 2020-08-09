use ferrous::*;

fn main() {
    let mut runtime = runtime::Runtime::new();
    let mut scope = runtime::Scope::new();
    let parser = grammar::StmtParser::new();
    let source = include_str!("test.fe");

    let stmt = parser.parse(source).unwrap();

    stmt.evaluate(&mut runtime, &mut scope);
}