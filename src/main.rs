mod ast;
mod codegen;
mod lexer;
mod parser;

use std::env;

use anyhow::anyhow;
use codegen::Codegen;
use inkwell::{context::Context, execution_engine::JitFunction};
use parser::Parser;

type EntryFunc = unsafe extern "C" fn() -> f64;

fn main() -> anyhow::Result<()> {
    let source: String = env::args().skip(1).collect::<Vec<String>>().join(" ");
    println!("Source:");
    println!("{}\n", source);
    let parser = Parser::default();
    let ast = parser.parse_str(&source)?;
    let context = Context::create();

    let mut codegen = Codegen::new(&context);
    codegen.codegen(&ast)?;
    println!("IR:");
    println!("{}", codegen.module.print_to_string().to_str()?);

    let ee = codegen
        .module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| anyhow!("{}", e.to_str().unwrap()))?;

    let entry: JitFunction<EntryFunc> = unsafe { ee.get_function("lambda") }?;

    println!("Result:");
    unsafe {
        println!("{}", entry.call());
    }

    Ok(())
}
