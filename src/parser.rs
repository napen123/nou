use std::collections::HashMap;

use crate::language::*;

pub fn parse(tokens: Vec<Token>) -> Result<(Vec<Instruction>, HashMap<String, Macro>), ()> {
    let mut current_macro = None;

    let mut top_level = Vec::new();
    let mut macros: HashMap<String, Macro> = HashMap::new();

    let mut iterator = tokens.into_iter();

    while let Some(token) = iterator.next() {
        let instruction = match token {
            Token::RightBrace => {
                if let Some((macro_name, macro_data)) = current_macro.take() {
                    macros.insert(macro_name, macro_data);
                    continue;
                } else {
                    panic!("WOO!")
                }
            }
            Token::Identifier(identifier) => {
                if let Some(macro_entry) = macros.get(&identifier) {
                    let mut values: [Value; 10] = Default::default();
                    debug_assert!(macro_entry.parameter_count <= 10);

                    for i in 0..macro_entry.parameter_count {
                        match parse_value(&mut iterator) {
                            Ok(value) => unsafe {
                                // SAFETY: The parameter_count is at most 10.
                                *values.get_unchecked_mut(i) = value;
                            },
                            Err(_) => panic!("WOO!"),
                        }
                    }

                    Instruction::Macro(
                        identifier,
                        ValueList {
                            length: macro_entry.parameter_count,
                            values,
                        },
                    )
                } else {
                    panic!("WOO: {}", identifier);
                }
            }
            Token::Builtin(builtin) => match parse_builtin(&mut iterator, builtin) {
                Ok(builtin) => Instruction::Builtin(builtin),
                Err(_) => panic!("WOO!"),
            },
            Token::Directive(directive) => {
                if current_macro.is_none() {
                    match directive {
                        Directive::Parameter(_) => panic!("WOO!"),
                        Directive::Macro => match parse_macro(&mut iterator, &mut macros) {
                            Ok(data) => {
                                current_macro = Some(data);
                                continue;
                            }
                            Err(_) => panic!("WOO!"),
                        },
                        Directive::Include => {
                            panic!("WOO!")
                        }
                    }
                } else {
                    panic!("WOO!");
                }
            }
            _ => panic!("WOO!"),
        };

        if let Some((_, macro_data)) = &mut current_macro {
            macro_data.instructions.push(instruction);
        } else {
            top_level.push(instruction);
        }
    }

    if let Some((macro_name, _)) = current_macro.take() {
        panic!("WOO: {}", macro_name)
    } else {
        Ok((top_level, macros))
    }
}

fn parse_macro<I>(iterator: &mut I, macros: &HashMap<String, Macro>) -> Result<(String, Macro), ()>
where
    I: Iterator<Item = Token>,
{
    let name = if let Some(token) = iterator.next() {
        if let Token::Identifier(identifier) = token {
            identifier
        } else {
            panic!("WOO: {:?}", token)
        }
    } else {
        panic!("WOO!")
    };

    if macros.contains_key(&name) {
        panic!("WOO!")
    }

    let parameter_count = if let Some(token) = iterator.next() {
        if let Token::Integer(integer) = token {
            integer
        } else {
            panic!("WOO!")
        }
    } else {
        panic!("WOO!")
    };

    if let Some(token) = iterator.next() {
        if let Token::LeftBrace = token {
            Ok((
                name,
                Macro {
                    parameter_count,
                    instructions: Vec::new(),
                },
            ))
        } else {
            panic!("WOO!")
        }
    } else {
        panic!("WOO!")
    }
}

fn parse_value<I>(iterator: &mut I) -> Result<Value, ()>
where
    I: Iterator<Item = Token>,
{
    if let Some(token) = iterator.next() {
        Ok(match token {
            Token::Integer(integer) => Value::Literal(integer),
            Token::Identifier(identifier) => Value::Variable(identifier),
            Token::Directive(Directive::Parameter(parameter)) => Value::Parameter(parameter),
            _ => panic!("WOO!"),
        })
    } else {
        panic!("WOO!")
    }
}

fn parse_builtin<I>(iterator: &mut I, builtin_ref: BuiltinReference) -> Result<Builtin, ()>
where
    I: Iterator<Item = Token>,
{
    Ok(match builtin_ref {
        BuiltinReference::Allocate => match parse_value(iterator) {
            Ok(value) => {
                if let Value::Literal(_) = value {
                    panic!("WOO!")
                } else {
                    Builtin::Allocate(value)
                }
            }
            Err(_) => panic!("WOO!"),
        },
        BuiltinReference::Set => match parse_value(iterator) {
            Ok(value) => {
                if let Value::Variable(_) = value {
                    panic!("WOO!")
                } else {
                    Builtin::Set(value)
                }
            }
            Err(_) => panic!("WOO!"),
        },
        BuiltinReference::Move => match parse_value(iterator) {
            Ok(value) => Builtin::Move(value),
            Err(_) => panic!("WOO!"),
        },
        BuiltinReference::Mark => Builtin::Mark,
        BuiltinReference::Restore => Builtin::Restore,
        BuiltinReference::Hint => match parse_value(iterator) {
            Ok(value) => Builtin::Hint(value),
            Err(_) => panic!("WOO!"),
        },

        BuiltinReference::Add => match parse_value(iterator) {
            Ok(value) => {
                if let Value::Variable(_) = value {
                    panic!("WOO!")
                } else {
                    Builtin::Add(value)
                }
            }
            Err(_) => panic!("WOO!"),
        },
        BuiltinReference::Subtract => match parse_value(iterator) {
            Ok(value) => {
                if let Value::Variable(_) = value {
                    panic!("WOO!")
                } else {
                    Builtin::Subtract(value)
                }
            }
            Err(_) => panic!("WOO!"),
        },
        BuiltinReference::Read => Builtin::Read,
        BuiltinReference::Write => Builtin::Write,
        BuiltinReference::IfZero => Builtin::IfZero,
        BuiltinReference::IfNotZero => Builtin::IfNotZero,
    })
}
