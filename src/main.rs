mod compiler;
mod language;
mod lexer;
mod parser;

fn main() {
    let tokens = lexer::lex_file("test.nou").unwrap();
    let (top_level, macros) = parser::parse(tokens).unwrap();
    compiler::Compiler::new()
        .compile(top_level, macros, "test.bf")
        .unwrap();
}
