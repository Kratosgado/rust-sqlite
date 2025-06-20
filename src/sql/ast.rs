use super::tokenizer::Ops;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    CreateTable(CreateTableStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub core: SelectCore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectCore {
    pub result_columns: Vec<ResultColumn>,
    pub from: SelectFrom,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultColumn {
    Star,
    Expr(ExprResultColumn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprResultColumn {
    pub expr: Expr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(String),
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Comparison(Box<Expr>, Ops, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectFrom {
    Table(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Real,
    Text,
    Blob,
}
