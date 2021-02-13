mod ast;
mod codegen;
mod lexer;
mod parser;

use std::env;

use codegen::Codegen;
use inkwell::context::Context;
use parser::Parser;

fn main() -> anyhow::Result<()> {
    let source: String = env::args().skip(1).collect::<Vec<String>>().join(" ");
    println!("{}", source);
    let parser = Parser::default();
    let ast = parser.parse_str(&source)?;
    let context = Context::create();
    let mut codegen = Codegen::new(&context);
    codegen.codegen(&ast)?;
    codegen.module.print_to_stderr();

    Ok(())
}
