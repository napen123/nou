/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod compiler;
mod language;
mod lexer;
mod logger;
mod parser;
mod verifier;

fn main() {
    let tokens = lexer::lex_file("test.nou").unwrap();
    let (top_level, macros) = parser::parse(tokens).unwrap();
    compiler::Compiler::new()
        .compile(top_level, macros, "test.bf")
        .unwrap();
}
