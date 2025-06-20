use super::ast::Literal;

#[derive(Debug, PartialEq)]
pub enum Token {
    Create,
    Table,
    LPar,
    RPar,
    Select,
    As,
    From,
    Star,
    Comma,
    SemiColon,
    Where,
    Op(Ops),
    Identifier(String),

    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ops {
    Eq,
    Ne,
    Lt,
    Gt,
    Loe,
    Goe,
    And,
    Or,
}

impl Token {
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Token::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    pub fn as_op(&self) -> Option<&Ops> {
        match self {
            Token::Op(op) => Some(op),
            _ => None,
        }
    }

    pub fn as_literal(&self) -> Option<Literal> {
        match self {
            Token::Literal(i) => Some(i.clone()),
            Token::Identifier(v) => Some(Literal::Text(v.clone())),
            _ => None,
        }
    }
}

pub fn tokenize(input: &str) -> anyhow::Result<Vec<Token>> {
    let mut tokens = vec![];
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' => tokens.push(Token::Star),
            ',' => tokens.push(Token::Comma),
            ';' => tokens.push(Token::SemiColon),
            '(' => tokens.push(Token::LPar),
            ')' => tokens.push(Token::RPar),
            '=' | '<' | '>' | '!' => {
                let mut op = c.to_string();
                if let Some(cc) = chars.next_if(|&cc| cc == '=') {
                    op.push(cc);
                }
                match op.as_str() {
                    "=" => tokens.push(Token::Op(Ops::Eq)),
                    "!=" => tokens.push(Token::Op(Ops::Ne)),
                    "<" => tokens.push(Token::Op(Ops::Lt)),
                    ">" => tokens.push(Token::Op(Ops::Gt)),
                    ">=" => tokens.push(Token::Op(Ops::Goe)),
                    "<=" => tokens.push(Token::Op(Ops::Loe)),
                    _ => anyhow::bail!("unexpected character: {c}"),
                }
            }
            c if c.is_whitespace() => continue,
            c if c.is_numeric() => {
                let mut num = c.to_string();
                while let Some(cc) = chars.next_if(|&cc| cc.is_numeric() || cc == '.') {
                    num.extend(cc.to_lowercase());
                }
                tokens.push(if num.contains('.') {
                    Token::Literal(Literal::Real(num.parse()?))
                } else {
                    Token::Literal(Literal::Int(num.parse()?))
                });
            }
            c if c.is_alphabetic() => {
                let mut ident = c.to_string().to_lowercase();
                while let Some(cc) = chars.next_if(|&cc| cc.is_alphanumeric() || cc == '_') {
                    ident.extend(cc.to_lowercase());
                }

                match ident.as_str() {
                    "create" => tokens.push(Token::Create),
                    "table" => tokens.push(Token::Table),
                    "select" => tokens.push(Token::Select),
                    "where" => tokens.push(Token::Where),
                    "as" => tokens.push(Token::As),
                    "from" => tokens.push(Token::From),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            _ => anyhow::bail!("unexpected character: {}", c),
        }
    }
    Ok(tokens)
}
