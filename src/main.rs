extern crate mirlvm;
use std::env;

use mirlvm::codegen::*;
use mirlvm::lexer::*;
use mirlvm::lowir::*;
use mirlvm::parser::*;
use mirlvm::rega::*;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let option = &args[1];

    let mut tmass = lex();
    if option == "--out-lex" {
        println!("{:#?}", tmass);
        return;
    }
    let parserprogram = parse(&mut tmass);
    if option == "--out-parse" {
        println!("{:#?}", parserprogram);
        return;
    }
    let lirpg = genlowir(parserprogram);
    if option == "--out-lowir" {
        println!("{:#?}", lirpg);
        return;
    }
    let lirpg2 = registeralloc(lirpg);
    if option == "--out-lowir_rega" {
        println!("{:#?}", lirpg2);
        return;
    }
    if option == "--out-lowir-ISA" {
        for func in lirpg2.funcs {
            println!("Function {}:", func.lb);
            for bb in func.rbbs {
                println!("{}:", bb.lb);
                for instr in bb.instrs {
                    println!("{}", instr);
                }
            }
        }
        return;
    }
    gen_x64code(lirpg2);
}
