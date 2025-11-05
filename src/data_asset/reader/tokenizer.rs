use std::io::{Result, Error};
use std::num::Wrapping;

#[derive(Debug, Clone, Copy)]
pub struct TokenPosition {
    pub line: u32,
}

#[derive(Debug)]
pub enum TokenData {
    Eof(),
    String(String),
    Char(String),
    Number(u64),
    Ident(String),
    Punct(char),
    PreProcessor(String),
}

#[derive(Debug)]
pub struct Token {
    pub data: TokenData,
    pub pos: TokenPosition,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.data {
            TokenData::Eof() => write!(f, "EOF"),
            TokenData::String(s) => write!(f, "\"{}\"", s),
            TokenData::Char(c) => write!(f, "'{}'", c),
            TokenData::Number(n) => write!(f, "{}", n),
            TokenData::Ident(id) => write!(f, "{}", &id),
            TokenData::Punct(c) => write!(f, "{}", c),
            TokenData::PreProcessor(pre) => write!(f, "{}", &pre),
        }
    }
}

#[allow(dead_code)]
impl Token {
    fn eof(line: u32) -> Token {
        Token {
            data: TokenData::Eof(),
            pos: TokenPosition { line },
        }
    }
    
    fn punct(ch: char, line: u32) -> Token {
        Token {
            data: TokenData::Punct(ch),
            pos: TokenPosition { line },
        }
    }

    fn string(s: String, line: u32) -> Token {
        Token {
            data: TokenData::String(s),
            pos: TokenPosition { line },
        }
    }
    
    fn new_char(ch: String, line: u32) -> Token {
        Token {
            data: TokenData::Char(ch),
            pos: TokenPosition { line },
        }
    }

    fn number(n: u64, line: u32) -> Token {
        Token {
            data: TokenData::Number(n),
            pos: TokenPosition { line },
        }
    }
    
    fn ident(s: String, line: u32) -> Token {
        Token {
            data: TokenData::Ident(s),
            pos: TokenPosition { line },
        }
    }
    
    fn pre_processor(s: String, line: u32) -> Token {
        Token {
            data: TokenData::PreProcessor(s),
            pos: TokenPosition { line },
        }
    }
    
    pub fn is_eof(&self) -> bool {
        matches!(self.data, TokenData::Eof())
    }
    
    pub fn is_pre_processor(&self) -> bool {
        matches!(self.data, TokenData::PreProcessor(_))
    }

    pub fn is_any_ident(&self) -> bool {
        matches!(self.data, TokenData::Ident(_))
    }

    pub fn is_any_punct(&self) -> bool {
        matches!(self.data, TokenData::Punct(_))
    }

    pub fn is_any_number(&self) -> bool {
        matches!(self.data, TokenData::Number(_))
    }

    pub fn is_string(&self, s: &str) -> bool {
        match &self.data {
            TokenData::String(self_s) => self_s == s,
            _ => false,
        }
    }

    pub fn is_number(&self, n: u64) -> bool {
        match self.data {
            TokenData::Number(self_n) => self_n == n,
            _ => false,
        }
    }

    pub fn is_punct(&self, ch: char) -> bool {
        match self.data {
            TokenData::Punct(self_ch) => self_ch == ch,
            _ => false,
        }
    }

    pub fn is_ident(&self, id: &str) -> bool {
        match &self.data {
            TokenData::Ident(self_id) => self_id == id,
            _ => false,
        }
    }

    pub fn get_number(&self) -> Option<u64> {
        match self.data {
            TokenData::Number(n) => Some(n),
            _ => None,
        }
    }


    pub fn get_ident(&self) -> Option<&str> {
        match &self.data {
            TokenData::Ident(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn get_punct(&self) -> Option<char> {
        match self.data {
            TokenData::Punct(c) => Some(c),
            _ => None,
        }
    }
    
    pub fn get_pre_processor(&self) -> Option<&String> {
        match &self.data {
            TokenData::PreProcessor(s) => Some(s),
            _ => None,
        }
    }
    
}

#[allow(dead_code)]
pub struct Tokenizer<'a> {
    input: std::str::Chars<'a>,
    unget_data: Option<char>,
    line: u32,
}

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            input: data.chars(),
            unget_data: None,
            line: 1,
        }
    }

    fn error<S: AsRef<str>>(&self, msg: S, line: u32) -> Result<Token> {
        Result::Err(Error::other(format!("line {}: {}", line, msg.as_ref())))
    }

    fn next_char(&mut self) -> Option<char> {
        match self.unget_data {
            Some(c) => {
                self.unget_data = None;
                Some(c)
            },
            _ => self.input.next(),
        }
    }
    
    pub fn read(&mut self) -> Result<Token> {
        loop {
            let ch = match self.next_char() {
                Some(c) => c,
                None => return Ok(Token::eof(self.line)),
            };
            if ch == ' ' || ch == '\t' || ch == '\r' { continue; }
            if ch == '\n' { self.line += 1; continue; }

            // skip comments
            if ch == '/' {
                let next = match self.next_char() {
                    Some(c) => c,
                    None => return Ok(Token::punct(ch, self.line)),
                };
                
                // single-line comment
                if next == '/' {
                    loop {
                        match self.next_char() {
                            Some('\n') => { self.line += 1; break; },
                            Some(_) => {},
                            None => return Ok(Token::eof(self.line)),
                        };
                    }
                    continue;
                }

                // multi-line comment
                if next == '*' {
                    let start_line = self.line;
                    let mut got_star = false;
                    loop {
                        match self.next_char() {
                            Some('*') => { got_star = true; },
                            Some('/') => { if got_star { break; } got_star = false; },
                            Some('\n') => { got_star = false; self.line += 1; }
                            Some(_) => { got_star = false; },
                            None => { self.error("unterminated comment", start_line)?; },
                        };
                    }
                    continue;
                }

                // not a comment after all
                self.unget_data = Some(next);
            }

            // pre-processor line
            if ch == '#' {
                let mut value = String::new();
                value.push(ch);
                loop {
                    match self.next_char() {
                        Some('\n') => { self.line += 1; break; },
                        Some(c) => { value.push(c); },
                        None => break,
                    };
                }
                return Ok(Token::pre_processor(value, self.line));
            }
            
            // string or char
            if ch == '"' || ch == '\'' {
                let start_line = self.line;
                let mut value = String::new();
                loop {
                    match self.next_char() {
                        Some('\n') => { self.error("unterminated string", start_line)?; },
                        Some('\\') => {
                            match self.next_char() {
                                Some('\\') => value.push('\\'),
                                Some('n') => value.push('\n'),
                                Some('r') => value.push('\r'),
                                Some('t') => value.push('\t'),
                                Some('"') => value.push('\"'),
                                Some('\'') => value.push('\''),
                                Some('\r') => {
                                    match self.next_char() {
                                        Some('\n') => { self.line += 1; }
                                        Some(ch) => { self.error(format!("invalid character following '\\r': {}", ch), self.line)?; },
                                        None => { self.error("unterminated string", start_line)?; },
                                    }
                                },
                                Some('\n') => { self.line += 1; },
                                Some(c) => { value.push('\\'); value.push(c); },
                                None => { self.error("unterminated string", start_line)?; },
                            };
                        },
                        Some(c) => {
                            if ch == c { break; }
                            value.push(c);
                        },
                        None => { self.error("unterminated string", start_line)?; },
                    }
                }
                return if ch == '"' {
                    Ok(Token::string(value, start_line))
                } else {
                    Ok(Token::new_char(value, start_line))
                };
            }

            // number
            if ch.is_ascii_digit() {
                let base = if ch == '0' {
                    match self.next_char() {
                        Some('x') => Wrapping(16u64),
                        Some('b') => Wrapping(2u64),
                        Some(c) => {
                            self.unget_data = Some(c);
                            Wrapping(8u64)
                        },
                        None => return Ok(Token::number(0, self.line)),
                    }
                } else {
                    Wrapping(10u64)
                };
                const CHAR_0 : Wrapping<u64> = Wrapping('0' as u64);
                const CHAR_UA : Wrapping<u64> = Wrapping('A' as u64);
                const CHAR_LA : Wrapping<u64> = Wrapping('a' as u64);
                let mut num = Wrapping(ch as u64) - CHAR_0;
                loop {
                    match self.next_char() {
                        Some(c) if c.is_ascii_digit() => {
                            let digit = Wrapping(c as u64) - CHAR_0;
                            if digit >= base {
                                self.error(format!("invalid digit in number: '{}'", c), self.line)?;
                            }
                            num = num * base + digit;
                        },
                        Some(c) if c.is_ascii_uppercase() => {
                            let digit = Wrapping(c as u64) - CHAR_UA + Wrapping(10u64);
                            if digit >= base {
                                self.error(format!("invalid digit in number: '{}'", c), self.line)?;
                            }
                            num = num * base + digit;
                        },
                        Some(c) if c.is_ascii_lowercase() => {
                            let digit = Wrapping(c as u64) - CHAR_LA + Wrapping(10);
                            if digit >= base {
                                self.error(format!("invalid digit in number: '{}'", c), self.line)?;
                            }
                            num = num * base + digit;
                        },
                        Some(c) => {
                            self.unget_data = Some(c);
                            break;
                        },
                        None => { break; }
                    };
                }
                return Ok(Token::number(num.0, self.line));
            }

            // identifier
            if ch.is_ascii_alphabetic() || ch == '_' {
                let mut value = String::new();
                value.push(ch);
                loop {
                    match self.next_char() {
                        Some(c) if c.is_ascii_alphanumeric() || c == '_' => {
                            value.push(c);
                        },
                        Some(c) => {
                            self.unget_data = Some(c);
                            break;
                        },
                        None => { break; },
                    };
                }
                return Ok(Token::ident(value, self.line));
            }
            
            // anything else is considered punctuation
            return Ok(Token::punct(ch, self.line));
        }
    }
}
