mod ast;
mod codegen;
mod lexer;
mod parser;

use std::{env, fs};

use anyhow::{anyhow, bail};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use codegen::Codegen;
use inkwell::{context::Context, execution_engine::JitFunction, OptimizationLevel};
use parser::Parser;

type EntryFunc = unsafe extern "C" fn() -> f64;

fn main() -> anyhow::Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("optimization")
                .short("o")
                .long("opt")
                .value_name("LEVEL")
                .help("Sets the amount of optimization of the jit compiler")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("dump source")
                .short("s")
                .long("dump-source")
                .help("If set will dump wizarding source to stdout"),
        )
        .arg(
            Arg::with_name("dump ir")
                .short("i")
                .long("dump-ir")
                .help("If set will dump llvm ir to stdout"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file(s) to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let opt_amount = match matches.value_of("optimization").unwrap() {
        "0" => OptimizationLevel::None,
        "1" => OptimizationLevel::Less,
        "2" => OptimizationLevel::Aggressive,
        amount => bail!("unknown optimization amount: {}", amount),
    };

    let source = fs::read_to_string(matches.value_of("INPUT").unwrap())?;
    if matches.is_present("dump source") {
        println!("Source:");
        println!("{}", source);
        println!()
    }

    let parser = Parser::default();
    let ast = parser.parse_str(&source)?;
    let context = Context::create();

    let mut codegen = Codegen::new(&context);
    codegen.codegen(&ast)?;
    if matches.is_present("dump ir") {
        println!("IR:");
        println!("{}", codegen.module.print_to_string().to_str()?);
    }

    let ee = codegen
        .module
        .create_jit_execution_engine(opt_amount)
        .map_err(|e| anyhow!("{}", e.to_str().unwrap()))?;

    let entry: JitFunction<EntryFunc> = unsafe { ee.get_function("lambda") }?;

    println!("Result:");
    unsafe {
        println!("{}", entry.call());
    }

    Ok(())
}
