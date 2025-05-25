#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Select(SelectStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectStatement {
    pub core: SelectCore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectCore {
    pub result_columns: Vec<ResultColumn>,
    pub from: SelectFrom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultColumn {
    Star,
    Expr(ExprResultColumn),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprResultColumn {
    pub expr: Expr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Column(Column),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectFrom {
    Table(String),
}
