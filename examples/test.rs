use ferrous::*;

fn main() {
    let source = include_str!("test.fe");
    let program = ast::Program::new(source).unwrap();
    let mut runtime = runtime::Runtime::new();
    let mut scope = runtime::Scope::new();

    println!("{:#?}", program);

    program.run(&mut runtime, &mut scope).unwrap();

    println!("{:#?}", runtime);
    println!("{:#?}", scope);

    let x = {
        let x = 2;

        {
            let y = 3;
            
            {
                x + y
            }
        }
    };

    println!("{}", x);
}
