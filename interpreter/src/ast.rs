use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Boolean,
    Array,
    Object,
    Regex,
    List(Box<Type>),
    Pairs(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Nil,
    Regex(String),
    Array(Vec<Expression>),
    Object(HashMap<String, Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    EphemeralVar(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
    BracketAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    TernaryThen {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
    TernaryQuestion {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
    StringInterpolation {
        parts: Vec<StringPart>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Is,
    IsNot,
    Match,
    NotMatch,

    // Logical
    And,
    Or,

    // Assignment
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        type_annotation: Option<Type>,
        is_const: bool,
        is_global: bool,
        name: String,
        value: Expression,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_ifs: Vec<(Expression, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    ForLoop {
        key_var: Option<String>,
        value_var: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    WhileLoop {
        condition: Expression,
        body: Vec<Statement>,
    },
    Loop {
        condition: Expression,
        body: Vec<Statement>,
    },
    TryCatch {
        try_body: Vec<Statement>,
        error_var: String,
        catch_body: Vec<Statement>,
    },
    Block {
        expression: Expression,
    },
    LibExport {
        name: String,
        exports: Vec<String>,
    },
    Import {
        path: String,
        alias: Option<String>,
    },
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
