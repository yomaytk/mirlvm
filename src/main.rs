extern crate mirlvm;
use std::env;

use mirlvm::codegen::*;
use mirlvm::deadcode::*;
use mirlvm::lexer::*;
use mirlvm::lowir::*;
use mirlvm::parser::*;
use mirlvm::rega::*;
use mirlvm::dominators::*;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let option = &args[1];

    // lexical analysis
    let mut tmass = lex();

    if option == "--out-lex" {
        println!("{:#?}", tmass);
        return;
    }

    // parsing
    let mut ssaprogram = parse(&mut tmass);

    if option == "--out-parse" {
        println!("{:#?}", ssaprogram);
        return;
    }

    
    if option == "--out-ssair" {
        for func in &ssaprogram.funcs {
            println!("function {}", func.name);
            for b in &func.bls {
                println!("{}:", b.lb);
                for instr in &b.instrs {
                    println!("{:?}", instr.op);
                }
            }
        }
        return;
    }
    
    if option == "--out-gdata" {
        for gv in &ssaprogram.gvs {
            println!("{:?}", gv);
        }
        return;
    }
    
    // compute dominators tree
    makedomt(&mut ssaprogram);
    
    // information for each basic block
    if option == "--out-parsebb" {
        for func in &ssaprogram.funcs {
            println!("function: {}", func.name);
            for bb in &func.bls {
                println!(
                    "\tlabel: {}, id: {}, instrscount: {}, transition blocks: {:?}. idom: {}",
                    bb.lb,
                    bb.id,
                    bb.instrs.len(),
                    bb.transbbs,
                    bb.idom,
                );
            }
        }
        return;
    }

    // SSA optical phase
    // remove useless instr
    removeuselessinstr(&mut ssaprogram);
    
    if option == "--out-ssair_1" {
        for func in &ssaprogram.funcs {
            println!("function {}", func.name);
            for b in &func.bls {
                println!("{}:", b.lb);
                for instr in &b.instrs {
                    if instr.living {
                        println!("{:?}", instr.op);
                    }
                }
            }
        }
        return;
    }

    // generate very low code
    let lirpg = genlowir(ssaprogram);
    if option == "--out-lowir" {
        println!("{:#?}", lirpg);
        return;
    }

    // register allocate
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

    // generate x64 code
    gen_x64code(lirpg2);
}
