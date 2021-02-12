#[derive(Debug, PartialEq, Clone)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(f64),
    Variable(String),
    Binary(String, Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub prototype: Prototype,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Extern(Prototype),
    Function(Function),
}
