use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::language::{BuiltinReference, Directive, Token};

pub fn lex_file(filename: &str) -> Result<Vec<Token>, ()> {
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(_) => panic!("WOO!"),
    };

    let mut comment_depth = 0;
    let mut tokens = Vec::new();
    let mut number_buffer = String::new();

    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => panic!("WOO!"),
        };

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let mut iterator = trimmed.chars();

        while let Some(c) = iterator.next() {
            if comment_depth == 0 {
                let token = match c {
                    '(' => {
                        comment_depth += 1;
                        continue;
                    }
                    ')' => panic!("WOO!"),

                    '{' => Token::LeftBrace,
                    '}' => Token::RightBrace,

                    '+' | '-' => {
                        number_buffer.push(c);

                        match lex_integer(&mut iterator, &mut number_buffer, 10) {
                            Ok(integer) => Token::Integer(integer),
                            Err(_) => panic!("WOO!"),
                        }
                    }
                    '#' | '%' | _ if c.is_ascii_digit() => {
                        let radix = match c {
                            '#' => 16,
                            '%' => 2,
                            _ => {
                                number_buffer.push(c);
                                10
                            }
                        };

                        match lex_integer(&mut iterator, &mut number_buffer, radix) {
                            Ok(integer) => Token::Integer(integer),
                            Err(_) => panic!("WOO!"),
                        }
                    }
                    '\'' => match lex_character(&mut iterator) {
                        Ok(character) => Token::Integer(character),
                        Err(_) => panic!("WOO!"),
                    },

                    '"' => match lex_string(&mut iterator) {
                        Ok(string) => Token::String(string),
                        Err(_) => panic!("WOO!"),
                    },

                    '@' => match lex_identifier(&mut iterator, None) {
                        Ok(identifier) if identifier.len() > 0 => {
                            match Directive::try_from(identifier.as_ref()) {
                                Ok(directive) => Token::Directive(directive),
                                Err(_) => panic!("WOO!"),
                            }
                        }
                        Ok(_) => panic!("WOO!"),
                        Err(_) => panic!("WOO!"),
                    },

                    _ if is_identifier_head(c) => match lex_identifier(&mut iterator, Some(c)) {
                        Ok(identifier) => {
                            if identifier.starts_with("__") {
                                match BuiltinReference::try_from(&identifier[2..]) {
                                    Ok(builtin) => Token::Builtin(builtin),
                                    Err(_) => Token::Identifier(identifier),
                                }
                            } else {
                                Token::Identifier(identifier)
                            }
                        }
                        Err(_) => panic!("WOO!"),
                    },

                    _ if c.is_whitespace() => continue,
                    _ => panic!("WOO!"),
                };

                tokens.push(token);
            } else {
                match c {
                    '(' => comment_depth += 1,
                    ')' => comment_depth -= 1,
                    _ => {}
                }
            }
        }
    }

    Ok(tokens)
}

fn lex_integer<I>(iterator: &mut I, buffer: &mut String, radix: u32) -> Result<usize, ()>
where
    I: Iterator<Item = char>,
{
    while let Some(next) = iterator.next() {
        match next {
            '_' => {}
            _ if next.is_digit(radix) => buffer.push(next),
            _ if is_token_break(next) => break,
            _ => panic!("WOO!"),
        }
    }

    match isize::from_str_radix(buffer, radix) {
        Ok(integer) => {
            buffer.clear();
            Ok(integer as usize)
        }
        Err(_) => panic!("WOO"),
    }
}

fn lex_character<I>(iterator: &mut I) -> Result<usize, ()>
where
    I: Iterator<Item = char>,
{
    if let Some(character) = iterator.next() {
        if let Some(terminator) = iterator.next() {
            if terminator == '\'' {
                let mut bytes = [0; 4];
                character.encode_utf8(&mut bytes);
                // TODO: Is this correct? Does encode_utf8 use native byte ordering?
                Ok(u32::from_ne_bytes(bytes) as usize)
            } else {
                panic!("WOO!");
            }
        } else {
            panic!("WOO!");
        }
    } else {
        panic!("WOO!");
    }
}

fn lex_string<I>(iterator: &mut I) -> Result<String, ()>
where
    I: Iterator<Item = char>,
{
    let mut string = String::new();

    while let Some(next) = iterator.next() {
        match next {
            '\"' => break,
            '\\' => {
                if let Some(escaped) = iterator.next() {
                    string.push(escaped);
                } else {
                    panic!("WOO!");
                }
            }
            _ => string.push(next),
        }
    }

    Ok(string)
}

fn lex_identifier<I>(iterator: &mut I, head: Option<char>) -> Result<String, ()>
where
    I: Iterator<Item = char>,
{
    let mut identifier = String::new();

    if let Some(c) = head {
        identifier.push(c);
    }

    while let Some(next) = iterator.next() {
        if is_identifier_body(next) {
            identifier.push(next);
        } else if is_token_break(next) {
            break;
        } else {
            panic!("WOO!");
        }
    }

    Ok(identifier)
}

#[inline]
fn is_identifier_head(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

#[inline]
fn is_identifier_body(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[inline]
fn is_token_break(c: char) -> bool {
    c.is_whitespace()
}
