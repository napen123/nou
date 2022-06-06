use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::iter;

use crate::language::*;

const KNOWN_ZERO_CAPACITY: usize = 32;

pub struct Compiler {
    head: usize,
    next_allocation: usize,

    allocation_stack: Vec<HashMap<String, usize>>,
    pool_stack: Vec<HashMap<String, usize>>,

    marker: usize,
    zero_horizon: usize,
    known_zeros: HashSet<usize>,

    brainfuck: Vec<Brainfuck>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut known_zeros = HashSet::with_capacity(KNOWN_ZERO_CAPACITY);

        for i in 0..KNOWN_ZERO_CAPACITY {
            known_zeros.insert(i);
        }

        let mut allocation_stack = Vec::new();
        allocation_stack.push(HashMap::new());

        Self {
            head: 0,
            next_allocation: 0,

            allocation_stack,
            pool_stack: Vec::new(),

            marker: 0,
            zero_horizon: 0,
            known_zeros,

            brainfuck: Vec::new(),
        }
    }

    pub fn compile(
        mut self,
        top_level: Vec<Instruction>,
        macros: HashMap<String, Macro>,
        filename: &str,
    ) -> Result<(), ()> {
        match self.build_instructions(&top_level, &macros, None) {
            Ok(_) => match self.save(filename) {
                Ok(()) => Ok(()),
                Err(_) => panic!("WOO!"),
            },
            Err(_) => panic!("WOO!"),
        }
    }

    fn build_instructions(
        &mut self,
        instructions: &Vec<Instruction>,
        macros: &HashMap<String, Macro>,
        values: Option<&ValueList>,
    ) -> Result<(), ()> {
        for instruction in instructions {
            match instruction {
                Instruction::Builtin(builtin) => {
                    println!("{:?}", builtin);

                    if let Err(_) = self.build_builtin(builtin, values) {
                        panic!("WOO!")
                    }
                }
                Instruction::Macro(macro_name, macro_values) => {
                    if let Some(macro_data) = macros.get(macro_name) {
                        println!(
                            "{} {:?}",
                            macro_name,
                            &macro_values.values[0..macro_values.length]
                        );

                        // TODO: Values should be resolved here! (Instead of in builtins?)

                        if let Err(_) = self.build_instructions(
                            &macro_data.instructions,
                            macros,
                            Some(macro_values),
                        ) {
                            panic!("WOO!")
                        }
                    } else {
                        panic!("WOO!")
                    }
                }
            }
        }

        Ok(())
    }

    fn build_builtin(&mut self, builtin: &Builtin, values: Option<&ValueList>) -> Result<(), ()> {
        match builtin {
            Builtin::Allocate(value) => {
                match resolve_value(value, values) {
                    Ok(resolved) => {
                        if let Value::Variable(variable) = resolved {
                            match self.allocate(variable) {
                                Ok(_) => {}
                                Err(_) => panic!("WOO!"),
                            }
                        } else {
                            // SAFETY: The parser ensures that the value is correct.
                            unreachable!()
                        }
                    }
                    Err(_) => panic!("WOO!"),
                }
            }
            Builtin::Set(value) => {
                match resolve_value(value, values) {
                    Ok(resolved) => {
                        if let Value::Literal(literal) = resolved {
                            self.set(*literal);
                        } else {
                            // SAFETY: The parser ensures that the value is correct.
                            unreachable!()
                        }
                    }
                    Err(_) => panic!("WOO!"),
                }
            }
            Builtin::Move(value) => {
                match resolve_value(value, values) {
                    Ok(resolved) => {
                        match resolved {
                            Value::Literal(literal) => {
                                self.move_to(*literal);
                            }
                            Value::Variable(variable) => {
                                if let Some(location) = self.variable_location(variable) {
                                    self.move_to(location);
                                } else {
                                    panic!("WOO!")
                                }
                            }
                            Value::Parameter(_) => {
                                // SAFETY: All parameters have been resolved.
                                unreachable!()
                            }
                        }
                    }
                    Err(_) => panic!("WOO!"),
                }
            }
            Builtin::Mark => {
                self.mark();
            }
            Builtin::Restore => {
                self.restore();
            }
            Builtin::Hint(value) => {
                match resolve_value(value, values) {
                    Ok(resolved) => {
                        match resolved {
                            Value::Literal(literal) => {
                                self.hint(*literal);
                            }
                            Value::Variable(variable) => {
                                if let Some(location) = self.variable_location(variable) {
                                    self.hint(location);
                                } else {
                                    panic!("WOO!")
                                }
                            }
                            Value::Parameter(_) => {
                                // SAFETY: All parameters have been resolved.
                                unreachable!()
                            }
                        }
                    }
                    Err(_) => panic!("WOO!"),
                }
            }

            Builtin::Increment => {
                self.increment();
            }
            Builtin::Decrement => {
                self.decrement();
            }
            Builtin::Read => {
                self.read();
            }
            Builtin::Write => {
                self.write();
            }
            Builtin::IfZero => {
                self.if_zero();
            }
            Builtin::IfNotZero => {
                self.if_not_zero();
            }
        }

        Ok(())
    }

    fn allocate(&mut self, name: &str) -> Result<(), ()> {
        if self.variable_exists(name) {
            panic!("WOO!")
        } else {
            let location = self.next_allocation;
            self.next_allocation += 1;

            // SAFETY: There is always an allocation stack; the top-level stack always exists.
            unsafe {
                self.allocation_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .insert(name.to_owned(), location);
            }

            Ok(())
        }
    }

    fn set(&mut self, value: usize) {
        if !self.cell_is_definitely_zero() {
            self.brainfuck.extend_from_slice(&[
                Brainfuck::IfZero,
                Brainfuck::Decrement,
                Brainfuck::IfNotZero,
            ]);
        }

        if value == 0 {
            self.known_zeros.insert(self.head);
        } else {
            self.taint();
            self.brainfuck
                .extend(iter::repeat(Brainfuck::Increment).take(value % 0xFF));
        }
    }

    fn move_to(&mut self, location: usize) {
        if location > self.head {
            self.brainfuck
                .extend(iter::repeat(Brainfuck::Right).take(location - self.head));
        } else if location < self.head {
            self.brainfuck
                .extend(iter::repeat(Brainfuck::Left).take(self.head - location));
        }

        self.head = location;
    }

    fn mark(&mut self) {
        self.marker = self.next_allocation;

        let new_stack = self.pool_stack.pop().unwrap_or_default();
        self.allocation_stack.push(new_stack);
    }

    fn restore(&mut self) {
        self.next_allocation = self.marker;

        // SAFETY: There is always an allocation stack; the top-level stack always exists.
        let popped = unsafe { self.allocation_stack.pop().unwrap_unchecked() };
        self.pool_stack.push(popped);
    }

    #[inline]
    fn hint(&mut self, location: usize) {
        self.known_zeros.insert(location);
    }

    fn increment(&mut self) {
        self.taint();
        self.brainfuck.push(Brainfuck::Increment);
    }

    fn decrement(&mut self) {
        self.taint();
        self.brainfuck.push(Brainfuck::Decrement);
    }

    fn read(&mut self) {
        self.taint();
        self.brainfuck.push(Brainfuck::Read);
    }

    #[inline]
    fn write(&mut self) {
        self.brainfuck.push(Brainfuck::Write);
    }

    #[inline]
    fn if_zero(&mut self) {
        self.brainfuck.push(Brainfuck::IfZero);
    }

    #[inline]
    fn if_not_zero(&mut self) {
        self.brainfuck.push(Brainfuck::IfNotZero);
    }

    fn taint(&mut self) {
        self.zero_horizon = self.head + 1;
        self.known_zeros.remove(&self.head);
    }

    #[inline]
    fn cell_is_definitely_zero(&self) -> bool {
        self.head >= self.zero_horizon || self.known_zeros.contains(&self.head)
    }

    fn variable_exists(&self, name: &str) -> bool {
        for stack in self.allocation_stack.iter().rev() {
            if stack.contains_key(name) {
                return true;
            }
        }

        false
    }

    fn variable_location(&self, name: &str) -> Option<usize> {
        for stack in self.allocation_stack.iter().rev() {
            if let Some(location) = stack.get(name) {
                return Some(*location);
            }
        }

        None
    }

    fn save(self, filename: &str) -> Result<(), ()> {
        match File::create(filename) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);

                for instruction in self.brainfuck {
                    let result = match instruction {
                        Brainfuck::Increment => write!(writer, "+"),
                        Brainfuck::Decrement => write!(writer, "-"),
                        Brainfuck::Left => write!(writer, "<"),
                        Brainfuck::Right => write!(writer, ">"),
                        Brainfuck::Read => write!(writer, ","),
                        Brainfuck::Write => write!(writer, "."),
                        Brainfuck::IfZero => write!(writer, "["),
                        Brainfuck::IfNotZero => write!(writer, "]"),
                    };

                    match result {
                        Ok(()) => {}
                        Err(_) => panic!("WOO!"),
                    }
                }

                Ok(())
            }
            Err(_) => panic!("WOO!"),
        }
    }
}

fn resolve_value<'a>(value: &'a Value, values: Option<&'a ValueList>) -> Result<&'a Value, ()> {
    if let Value::Parameter(parameter) = value {
        if let Some(values) = values {
            if *parameter < values.length {
                // SAFETY: The values.length field is at most 10.
                unsafe { Ok(values.values.get_unchecked(*parameter)) }
            } else {
                panic!("WOO!")
            }
        } else {
            panic!("WOO!")
        }
    } else {
        Ok(value)
    }
}
