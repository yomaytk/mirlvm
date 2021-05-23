extern crate mirlvm;

use mirlvm::parser::*;
use mirlvm::lexer::*;

fn main() {
    let mut tmass = lex();
    // println!("{:#?}", tmass);
    let program = parse(&mut tmass);
    println!("{:#?}", program);
}