use iron::prelude::*;

fn main() {
    let now = std::time::Instant::now();

    let args: Vec<_> = std::env::args().collect();

    let mut ctx = ();
    let mut engine = Engine::new();

    engine.register_fn("print", |union: Union| println!("{}", union));

    engine.eval_file(&mut ctx, &args[1]).unwrap();

    println!("{:?}", std::time::Instant::now() - now);
}
