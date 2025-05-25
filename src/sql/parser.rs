use anyhow::{bail, Context};

use super::tokenizer::Token;

#[derive(Debug)]
struct ParserState {
    tokens: Vec<Token>,
    pos: usize,
}

impl ParserState {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn next_token_is(&self, expected: Token) -> bool {
        self.tokens.get(self.pos) == Some(&expected)
    }

    fn expected_identifier(&mut self) -> anyhow::Result<&str> {
        self.expect_matching(|t| matches!(t, Token::Identifier(_)))
            .map(|t| t.as_identifier().unwrap())
    }

    fn expect_eq(&mut self, expected: Token) -> anyhow::Result<&Token> {
        self.expect_matching(|t| *t == expected)
    }

    fn expect_matching(&mut self, f: impl Fn(&Token) -> bool) -> anyhow::Result<&Token> {
        match self.next_token() {
            Some(token) if f(token) => Ok(token),
            Some(token) => anyhow::bail!("unexpected token: {:?}", token),
            None => anyhow::bail!("unexpected end of input"),
        }
    }

    fn peak_next_token(&self) -> anyhow::Result<&Token> {
        self.tokens.get(self.pos).context("unexpected end of input")
    }

    fn next_token(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
            // self.advance();
        }
        token
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}
