extern crate mirlvm;
use std::env;

use mirlvm::lexer::*;
use mirlvm::parser::*;
use mirlvm::lowir::*;
use mirlvm::rega::*;

fn main() {
    
    let args = env::args().collect::<Vec<String>>();
    let option = &args[1];

    let mut tmass = lex();
    if option == "--out-lex" {
        println!("{:#?}", tmass);
    }
    let parserprogram = parse(&mut tmass);
    if option == "--out-parse" {
        println!("{:#?}", parserprogram);
    }
    let lirprogram = genlowir(parserprogram);
    if option == "--out-lowir" {
        println!("{:#?}", lirprogram);
    }
    let lirprogram2 = registeralloc(lirprogram);
    if option == "--out-lowir_rega" {
        println!("{:#?}", lirprogram2);
    }
    if option == "--out-lowir-ISA" {
        for func in lirprogram2.funcs {
            println!("Function {}:", func.lb);
            for bb in func.rbbs {
                println!("{}:", bb.lb);
                for instr in bb.instrs {
                    println!("{}", instr);
                }
            }
        }
    }
}