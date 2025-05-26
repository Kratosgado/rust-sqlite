#[derive(Debug, PartialEq, Eq)]
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
    Identifier(String),
}

impl Token {
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Token::Identifier(ident) => Some(ident),
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
            c if c.is_whitespace() => continue,
            c if c.is_alphabetic() => {
                let mut ident = c.to_string().to_lowercase();
                while let Some(cc) = chars.next_if(|&cc| cc.is_alphanumeric() || cc == '_') {
                    ident.extend(cc.to_lowercase());
                }

                match ident.as_str() {
                    "create" => tokens.push(Token::Create),
                    "table" => tokens.push(Token::Table),
                    "select" => tokens.push(Token::Select),
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
