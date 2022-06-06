#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BuiltinReference {
    Allocate,
    Reserve,
    Set,
    Move,
    Mark,
    Restore,
    Hint,

    Add,
    Subtract,
    Left,
    Right,
    Read,
    Write,
    IfZero,
    IfNotZero,
}

impl TryFrom<&str> for BuiltinReference {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "allocate" => Self::Allocate,
            "reserve" => Self::Reserve,
            "set" => Self::Set,
            "move" => Self::Move,
            "mark" => Self::Mark,
            "restore" => Self::Restore,
            "hint" => Self::Hint,

            "add" => Self::Add,
            "sub" => Self::Subtract,
            "left" => Self::Left,
            "right" => Self::Right,
            "read" => Self::Read,
            "write" => Self::Write,
            "ifz" => Self::IfZero,
            "ifnz" => Self::IfNotZero,

            _ => return Err(()),
        })
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Literal(usize),
    Parameter(usize),
    Variable(String),
}

impl Default for Value {
    #[inline]
    fn default() -> Self {
        Self::Literal(0)
    }
}

#[derive(Debug)]
pub enum Builtin {
    Allocate(Value),
    Reserve(Value),
    Set(Value),
    Move(Value),
    Mark,
    Restore,
    Hint(Value),

    Add(Value),
    Subtract(Value),
    Left(Value),
    Right(Value),
    Read,
    Write,
    IfZero,
    IfNotZero,
}

#[derive(Debug)]
pub enum Directive {
    Parameter(usize),
    // TODO: Define(String, Value),
    Macro,
    Include,
}

impl TryFrom<&str> for Directive {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "0" => Self::Parameter(0),
            "1" => Self::Parameter(1),
            "2" => Self::Parameter(2),
            "3" => Self::Parameter(3),
            "4" => Self::Parameter(4),
            "5" => Self::Parameter(5),
            "6" => Self::Parameter(6),
            "7" => Self::Parameter(7),
            "8" => Self::Parameter(8),
            "9" => Self::Parameter(9),

            "macro" => Self::Macro,
            "include" => Self::Include,

            _ => return Err(()),
        })
    }
}

#[derive(Debug)]
pub enum Token {
    Identifier(String),
    Builtin(BuiltinReference),
    Directive(Directive),

    Integer(usize),
    String(String),

    LeftBrace,
    RightBrace,
}

#[derive(Clone, Debug)]
pub struct ValueList {
    pub length: usize,
    pub values: [Value; 10],
}

#[derive(Debug)]
pub enum Instruction {
    Macro(String, ValueList),
    Builtin(Builtin),
}

#[derive(Debug)]
pub struct Macro {
    pub parameter_count: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Brainfuck {
    Increment,
    Decrement,
    Left,
    Right,
    Read,
    Write,
    IfZero,
    IfNotZero,
}
