extern crate mirlvm;

use mirlvm::lexer::*;
use mirlvm::parser::*;
use mirlvm::lowir::*;

fn main() {
    let mut tmass = lex();
    // println!("{:#?}", tmass);
    let parserprogram = parse(&mut tmass);
    // println!("{:#?}", parserprogram);
    let lirprogram = genlowir(parserprogram);
    println!("{:#?}", lirprogram);
}