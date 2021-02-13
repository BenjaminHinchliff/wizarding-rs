use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue},
};

use crate::ast::{ASTNode, Expression, Function, Prototype};

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("unknown variable referenced {0}")]
    UnknownVariable(String),
    #[error("unknown operator {0}")]
    UnknownOperator(String),
    #[error("unknown function {0}")]
    UnknownFunction(String),
    #[error("invalid number of args in call {0} expected {1} found {2}")]
    InvalidCall(String, usize, usize),
    #[error("failed to verify function {0}")]
    InvalidFunction(String),
}

pub struct Codegen<'a> {
    pub context: &'a Context,
    pub module: Module<'a>,
    pub builder: Builder<'a>,
    pub named_values: HashMap<String, BasicValueEnum<'a>>,
}

impl<'a> Codegen<'a> {
    pub fn new(context: &'a Context) -> Codegen {
        let module = context.create_module("wizarding");
        let builder = context.create_builder();

        Codegen {
            context,
            module,
            builder,
            named_values: HashMap::new(),
        }
    }

    fn codegen_expr(&mut self, expr: &Expression) -> Result<FloatValue<'a>, CodegenError> {
        match expr {
            Expression::Literal(value) => Ok(self.context.f64_type().const_float(*value)),
            Expression::Variable(name) => match self.named_values.get(name) {
                Some(var) => Ok(var.into_float_value()),
                None => Err(CodegenError::UnknownVariable(name.clone())),
            },
            Expression::Binary(op, left, right) => {
                let lhs = self.codegen_expr(left)?;
                let rhs = self.codegen_expr(right)?;

                match op.as_str() {
                    "+" => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    "-" => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    "*" => Ok(self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    "/" => Ok(self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    _ => Err(CodegenError::UnknownOperator(op.clone())),
                }
            }
            Expression::Call(callee, args) => match self.module.get_function(callee) {
                Some(func) => {
                    if func.get_params().len() != args.len() {
                        return Err(CodegenError::InvalidCall(
                            callee.clone(),
                            func.get_params().len(),
                            args.len(),
                        ));
                    }

                    let mut gened_args = Vec::with_capacity(args.len());

                    for arg in args {
                        gened_args.push(self.codegen_expr(arg)?);
                    }

                    let argsv: Vec<BasicValueEnum> =
                        gened_args.iter().by_ref().map(|&val| val.into()).collect();

                    match self
                        .builder
                        .build_call(func, argsv.as_slice(), "tmp")
                        .try_as_basic_value()
                        .left()
                    {
                        Some(value) => Ok(value.into_float_value()),
                        None => panic!("recieved instruction from build call somehow"),
                    }
                }
                None => Err(CodegenError::UnknownFunction(callee.clone())),
            },
        }
    }

    fn compile_proto(&self, proto: &Prototype) -> Result<FunctionValue<'a>, CodegenError> {
        let args_types = std::iter::repeat(self.context.f64_type())
            .take(proto.args.len())
            .map(|f| f.into())
            .collect::<Vec<BasicTypeEnum>>();
        let args_types = args_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(args_types, false);
        let fn_val = self.module.add_function(proto.name.as_str(), fn_type, None);

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_float_value().set_name(proto.args[i].as_str());
        }

        Ok(fn_val)
    }

    fn compile_fn(&mut self, function: &Function) -> Result<FunctionValue<'a>, CodegenError> {
        let Function {
            prototype: proto,
            body,
        } = function;
        let llvm_func = self.compile_proto(proto)?;

        let entry = self.context.append_basic_block(llvm_func, "entry");

        self.builder.position_at_end(entry);

        self.named_values.reserve(proto.args.len());

        for (i, arg) in llvm_func.get_param_iter().enumerate() {
            self.named_values.insert(proto.args[i].clone(), arg);
        }

        let body = self.codegen_expr(body)?;

        self.builder.build_return(Some(&body));

        if llvm_func.verify(true) {
            Ok(llvm_func)
        } else {
            unsafe {
                llvm_func.delete();
            }

            Err(CodegenError::InvalidFunction(proto.name.clone()))
        }
    }

    pub fn codegen(&mut self, ast_nodes: &Vec<ASTNode>) -> Result<(), CodegenError> {
        for node in ast_nodes {
            match node {
                ASTNode::Function(func) => self.compile_fn(func),
                ASTNode::Extern(func) => self.compile_proto(func),
            }?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use inkwell::context::Context;
    use parser::Parser;

    use crate::parser;

    use super::Codegen;

    #[test]
    fn codegen_works() {
        let parser = Parser::default();
        let mut ast = parser
            .parse_str("extern sin(x); def thing(x) sin(x) * x;")
            .unwrap();
        let context = Context::create();
        let mut codegen = Codegen::new(&context);
        codegen.codegen(&mut ast).unwrap();
        println!("{}", codegen.module.print_to_string().to_str().unwrap());
        panic!();
    }
}
