extern crate mirlvm;
use std::env;

use mirlvm::codegen::*;
use mirlvm::deadcode::*;
use mirlvm::dominators::*;
use mirlvm::lexer::*;
use mirlvm::lowir::*;
use mirlvm::mem2reg::*;
use mirlvm::parser::*;
use mirlvm::rega::*;
use mirlvm::rev_ssa::*;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let option = &args[1];
    let mut option2 = "";
    let mut option3 = "";

    if args.len() > 3 {
        option2 = &args[2];
    }

    if args.len() > 4 {
        option3 = &args[3];
    }

    // lexical analysis
    let mut tmass = lex();

    if option == "--out-lex" {
        println!("{:#?}", tmass);
        return;
    }

    // parsing
    let mut ssaprogram = parse(&mut tmass);

    if option == "--out-m2rinfo" {
        for func in ssaprogram.funcs {
            println!("{}:", func.name);
            for m2ralloc in func.m2rinfo.iter() {
                println!("\t{:?}", m2ralloc);
            }
        }
        return;
    }

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
    dominators(&mut ssaprogram);

    // information for each basic block
    if option == "--out-parsebb" {
        for func in &ssaprogram.funcs {
            println!("function: {}", func.name);
            for bb in &func.bls {
                println!(
                    "\tlabel: {}, id: {}, instrscount: {}, transition blocks: {:?}. idom: {}, domf: {:?}",
                    bb.lb,
                    bb.id,
                    bb.instrs.len(),
                    bb.transbbs,
                    bb.idom,
                    bb.domfros,
                );
            }
        }
        return;
    }

    if option == "--out-graph" {
        for func in &ssaprogram.funcs {
            println!("function: {}", func.name);
            println!("graph: {:?}", func.cfg.as_ref().unwrap().graph);
            println!("rgraph: {:?}", func.cfg.as_ref().unwrap().rgraph);
        }
        return;
    }

    removeuselessinstr(&mut ssaprogram);

    // SSA optical phase
    // remove useless instr
    if option2 == "-O1" {
        ezmem2reg(&mut ssaprogram);
        mem2reg(&mut ssaprogram);
    }

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

    rev_ssa(&mut ssaprogram);

    if option == "--out-norm_fmt" {
        for func in &ssaprogram.funcs {
            println!("func: {}", func.name);
            for b in &func.bls {
                println!("{}:", b.lb);
                for instr in &b.instrs {
                    if instr.living {
                        println!("{:?}", instr);
                    }
                }
            }
        }
        return;
    }

    // generate very low code
    let lirpg = genlowir(ssaprogram);

    if option == "--out-lowir" {
        for func in &lirpg.funcs {
            println!("fun: {}", func.lb);
            for bb in &func.rbbs {
                println!("{}:", bb.lb);
                for isr in &bb.instrs {
                    println!("{}", isr);
                }
            }
        }
        return;
    }

    if option == "--out-lowir_all" {
        for func in &lirpg.funcs {
            println!("fun: {}", func.lb);
            for bb in &func.rbbs {
                println!("{}:", bb.lb);
                for isr in &bb.instrs {
                    println!("{:?}", isr);
                }
            }
        }
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
                    println!("{:?}", instr);
                }
            }
        }
        return;
    }

    // generate x64 code
    gen_x64code(lirpg2, option3 == "-Sec");
}
