use std::{iter::Peekable, vec};

use codespan::Span;
use indexmap::IndexMap;

#[derive(Debug)]
pub enum Error {
    SourceFileTooBig(usize),
    InvalidCharacter(char, Span),
    UnterminatedStringLiteral(Span),
    UnexpectedToken(String, Token),
    TooNested(Span),
    UnexpectedEndOfFile,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Expr {
    Array(Vec<Expr>, Span),
    Object(IndexMap<String, Expr>, Span),
    String(String, Span),
    Ident(String, Span),
}

struct TokenStream {
    tokens: Peekable<vec::IntoIter<Token>>,
    nesting: usize,
}

impl TokenStream {
    fn peek(&mut self) -> Result<&Token> {
        self.tokens.peek().ok_or_else(|| Error::UnexpectedEndOfFile)
    }
    fn next(&mut self) -> Result<Token> {
        self.tokens.next().ok_or_else(|| Error::UnexpectedEndOfFile)
    }
    fn eat(&mut self, kind: TokenKind) -> Option<Token> {
        let next = self.peek().ok()?;
        if next.kind != kind {
            return None;
        }
        Some(self.next().unwrap())
    }
    fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        let next = self.next()?;
        if next.kind != kind {
            return Err(Error::UnexpectedToken(format!("expected {kind:?}"), next));
        }
        Ok(next)
    }
}

pub fn parse(input: &str) -> Result<Expr> {
    let tokens = lexer(input)?;
    dbg!(&tokens);
    let expr = parse_expr(&mut TokenStream {
        tokens: tokens.into_iter().peekable(),
        nesting: 0,
    })?;
    dbg!(&expr);
    Ok(expr)
}

fn parse_expr(t: &mut TokenStream) -> Result<Expr> {
    let token = t.next()?;
    if t.nesting > 50 {
        return Err(Error::TooNested(token.span));
    }
    let expr = match token.kind {
        TokenKind::BracketOpen => {
            let mut elems = vec![];
            let mut had_comma = true;
            loop {
                if let Ok(Token {
                    kind: TokenKind::BracketClose,
                    ..
                }) = t.peek()
                {
                    break;
                }

                if !had_comma {
                    return Err(Error::UnexpectedToken("comma".to_owned(), t.next()?));
                }

                t.nesting += 1;
                elems.push(parse_expr(t)?);
                t.nesting -= 1;

                had_comma = t.eat(TokenKind::Comma).is_some();
            }

            t.next()?;
            Expr::Array(elems, token.span)
        }
        TokenKind::BraceOpen => {
            let mut elems = IndexMap::new();
            let mut had_comma = true;
            loop {
                if let Ok(Token {
                    kind: TokenKind::BraceClose,
                    ..
                }) = t.peek()
                {
                    break;
                }

                if !had_comma {
                    return Err(Error::UnexpectedToken("comma".to_owned(), t.next()?));
                }

                let name_tok = t.next()?;
                let Token {
                    kind: TokenKind::Ident(name),
                    ..
                } = name_tok
                else {
                    return Err(Error::UnexpectedToken("expected name".into(), name_tok));
                };
                t.expect(TokenKind::Equal)?;
                t.nesting += 1;
                elems.insert(name, parse_expr(t)?);
                t.nesting -= 1;

                had_comma = t.eat(TokenKind::Comma).is_some();
            }

            t.next()?;
            Expr::Object(elems, token.span)
        }
        TokenKind::String(s) => Expr::String(s, token.span),
        TokenKind::Ident(s) => Expr::Ident(s, token.span),
        _ => return Err(Error::UnexpectedToken("expected expression".into(), token)),
    };
    Ok(expr)
}

#[derive(Debug, Clone)]
pub struct Token {
    span: Span,
    kind: TokenKind,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    Equal,
    Comma,
    Ident(String),
    String(String),
}

fn lexer(input_str: &str) -> Result<Vec<Token>> {
    if input_str.len() >= (u32::MAX as usize) {
        return Err(Error::SourceFileTooBig(input_str.len()));
    }
    let mut input = input_str.char_indices().peekable();

    let mut tokens = vec![];

    while let Some((idx, c)) = input.next() {
        let span = Span::new(idx as u32, (idx as u32) + 1);
        let mut simple = |kind| tokens.push(Token { span, kind });
        match c {
            c if c.is_whitespace() => {}
            '#' => loop {
                if let Some((_, '\n')) = input.next() {
                    break;
                }
            },
            '[' => simple(TokenKind::BracketOpen),
            ']' => simple(TokenKind::BracketClose),
            '{' => simple(TokenKind::BraceOpen),
            '}' => simple(TokenKind::BraceClose),
            '=' => simple(TokenKind::Equal),
            ',' => simple(TokenKind::Comma),
            '"' => loop {
                match input.next() {
                    None => return Err(Error::UnterminatedStringLiteral(span)),
                    Some((next_idx, '"')) => {
                        let s = &input_str[(idx + 1)..next_idx];
                        tokens.push(Token {
                            span: Span::new(idx as u32, next_idx as u32),
                            kind: TokenKind::String(s.to_owned()),
                        });
                        break;
                    }
                    Some(_) => {}
                }
            },
            c if c.is_alphabetic() => {
                while let Some((next_idx, c)) = input.peek() {
                    if !c.is_alphanumeric() {
                        let s = &input_str[idx..*next_idx];
                        tokens.push(Token {
                            span: Span::new(idx as u32, *next_idx as u32),
                            kind: TokenKind::Ident(s.to_owned()),
                        });
                        break;
                    }
                    input.next();
                }
            }
            c => return Err(Error::InvalidCharacter(c, span)),
        }
    }

    Ok(tokens)
}

impl Error {
    pub(crate) fn into_msg_and_span(self) -> (String, Span) {
        match self {
            Error::SourceFileTooBig(_) => ("source file bigger than 4GB".into(), Span::default()),
            Error::InvalidCharacter(c, sp) => (format!("invalid character: {c}"), sp),
            Error::UnterminatedStringLiteral(sp) => (format!("unterminated string literal"), sp),
            Error::UnexpectedToken(expected, tok) => {
                (format!("unexpected token, expected {expected}"), tok.span)
            }
            Error::TooNested(sp) => ("too nested".to_owned(), sp),
            Error::UnexpectedEndOfFile => ("unexpected end of file".to_owned(), Span::default()),
        }
    }
}
